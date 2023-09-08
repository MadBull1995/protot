// Copyright 2023 The ProtoT Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{
    collections::{hash_map::Keys, HashMap},
    fmt,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc, Condvar, Mutex,
    },
    thread, error::Error
};

use async_trait::async_trait;
use tonic::Status;

use super::{
    job::Job,
    worker::{LocalWorker, Worker, WorkerType},
};

// Lib modules
#[allow(unused_imports)]
use crate::{internal::protot::scheduler::v1::{ExecuteRequest, TaskCompletion}, logger, SchedulerError};

#[cfg(feature = "stats")]
use crate::server::metrics::{
    self, increment_task, set_worker_pool_metric, WorkerPoolMetricType, WorkerPoolTaskType,
};

// Trait for task execution
pub trait TaskExecutor: Send + Sync + 'static {
    fn execute(&self, args: ExecuteRequest);
}

// Trait for task execution
#[async_trait(?Send)]
pub trait AsyncTaskExecutor: Send + Sync + 'static {
    async fn execute(&self, args: ExecuteRequest) -> Result<TaskCompletion, Box<dyn Error>>;
}

// Struct to hold task executions and their argument implementations
pub struct TaskRegistry {
    registry: HashMap<String, Box<dyn TaskExecutor>>,
}

// Struct to hold task executions and their argument implementations
pub struct GrpcWorkersRegistry {
    registry: HashMap<String, Box<dyn AsyncTaskExecutor>>,
}


impl GrpcWorkersRegistry {
    pub fn new() -> Self {
        GrpcWorkersRegistry {
            registry: HashMap::new()
        }
    }

    pub fn register_task<E>(&mut self, task_name: &str, executor: E)
    where
        E: AsyncTaskExecutor,
    {
        self.registry
            .insert(task_name.to_string(), Box::new(executor));
    }

    pub fn get_executor(
        &self,
        excutor_name: &str,
    ) -> Result<&Box<dyn AsyncTaskExecutor>, SchedulerError> {
        if let Some(executor) = self.registry.get(excutor_name) {
            Ok(executor)
        } else {
            Err(SchedulerError::TaskExecutionError(format!(
                "Executor not found for task: {}",
                excutor_name
            )))
        }
    }
}

impl Default for TaskRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskRegistry {
    pub fn new() -> Self {
        TaskRegistry {
            registry: HashMap::new(),
        }
    }

    pub fn register_task<E>(&mut self, task_name: &str, executor: E)
    where
        E: TaskExecutor,
    {
        self.registry
            .insert(task_name.to_string(), Box::new(executor));
    }

    pub fn iter_tasks(&self) -> Keys<'_, String, Box<dyn TaskExecutor>> {
        self.registry.keys()
    }

    pub fn get_executor(
        &self,
        excutor_name: &str,
    ) -> Result<&Box<dyn TaskExecutor>, SchedulerError> {
        if let Some(executor) = self.registry.get(excutor_name) {
            Ok(executor)
        } else {
            Err(SchedulerError::TaskExecutionError(format!(
                "Executor not found for task: {}",
                excutor_name
            )))
        }
    }
}

pub struct Sentinel<'a> {
    worker_id: usize,
    shared_data: &'a Arc<WorkerPoolSharedData>,
    active: bool,
}

impl<'a> Sentinel<'a> {
    pub fn new(worker_id: usize, shared_data: &'a Arc<WorkerPoolSharedData>) -> Sentinel<'a> {
        Sentinel {
            shared_data,
            worker_id,
            active: true,
        }
    }

    /// Cancel and destroy this sentinel.
    pub fn cancel(mut self) {
        self.active = false;
    }
}

impl<'a> Drop for Sentinel<'a> {
    fn drop(&mut self) {
        if self.active {
            self.shared_data.active_count.fetch_sub(1, Ordering::SeqCst);
            #[cfg(feature = "stats")]
            {
                metrics::set_worker_pool_metric(
                    metrics::WorkerPoolMetricType::Active,
                    self.shared_data.active_count.load(Ordering::SeqCst),
                );
            }
            if thread::panicking() {
                self.shared_data.panic_count.fetch_add(1, Ordering::SeqCst);
                #[cfg(feature = "stats")]
                {
                    metrics::increment_task(metrics::WorkerPoolTaskType::Panic);
                }
            }
            self.shared_data.no_work_notify_all();
            log::error!("sentinel droped, spawnning a new worker",);
            spawn_in_pool(self.worker_id, self.shared_data.clone());
        }
    }
}

#[derive(Clone, Default)]
pub struct Builder {
    name: Option<String>,
    num_workers: Option<usize>,
    thread_stack_size: Option<usize>,
    workers_type: WorkerType,
    force_shutdown: Option<bool>,
    executors: Option<Arc<Mutex<TaskRegistry>>>,
}

fn init_registry(executors: Option<Arc<Mutex<TaskRegistry>>>) -> Arc<Mutex<TaskRegistry>> {
    let registry = match executors.is_none() {
        true => Arc::new(Mutex::new(TaskRegistry::new())),
        _ => executors.unwrap(),
    };
    let cloned = registry.clone();
    let binding = registry.as_ref().lock().unwrap();

    let tasks_iterator = binding.iter_tasks();
    for task in tasks_iterator {
        log::debug!("registerd: {}", task);
    }

    cloned
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            name: None,
            num_workers: None,
            thread_stack_size: None,
            force_shutdown: None,
            executors: None,
            ..Default::default()
        }
    }

    /// If the scheduler system work with remote workers over gRPC
    pub fn grpc_workers(mut self) -> Builder {
        self.workers_type = WorkerType::Remote;
        self
    }

    // Num of workers
    pub fn num_workers(mut self, num_workers: usize) -> Builder {
        self.num_workers = Some(num_workers);
        self
    }

    /// Custom thread name for main process
    pub fn name(mut self, name: String) -> Builder {
        self.name = Some(name);
        self
    }

    /// The RUST_MIN_STACK environment variable can override the default stack size of 2 MB for created threads
    pub fn thread_stack_size(mut self, thread_stack_size: usize) -> Builder {
        self.thread_stack_size = Some(thread_stack_size);
        self
    }

    /// Will force the scheduler server to shutdown even if there is still working tasks
    pub fn with_force_shutdown(mut self) -> Builder {
        self.force_shutdown = Some(true);
        self
    }

    pub fn executors(mut self, executors: Arc<Mutex<TaskRegistry>>) -> Builder {
        self.executors = Some(executors);
        self
    }

    pub fn build(self) -> Result<WorkerPool, SchedulerError> {
        let num_workers = self.num_workers.unwrap_or_else(num_cpus::get);
        let force_shutdown = self.force_shutdown.unwrap_or(false);
        let (tx, rx) = channel::<Job<'static, ExecuteRequest>>();

        let (shared_data, workers) = match self.workers_type {
            WorkerType::Local => initialize_thread_pool(num_workers, rx, force_shutdown, self.name),
            WorkerType::Remote => Err(SchedulerError::PoolCreationError(
                "Cant serve gRPC based workers currently".to_string(),
            ))?,
        };

        #[cfg(feature = "stats")]
        {
            // Increment the counter
            metrics::set_worker_pool_metric(
                metrics::WorkerPoolMetricType::TotalWorkers,
                num_workers,
            );
        }

        let registry = init_registry(self.executors);
        Ok(WorkerPool {
            jobs: Some(tx),
            shared_data,
            executors: registry,
            workers,
        })
    }

}


pub struct GrpcWorkerPool {
    execute_channel: Option<Sender<ExecuteRequest>>,
}

pub struct WorkerPool {
    jobs: Option<Sender<Job<'static, ExecuteRequest>>>,
    shared_data: Arc<WorkerPoolSharedData>,
    pub executors: Arc<Mutex<TaskRegistry>>,
    workers: Vec<Arc<dyn Worker>>,
}

impl WorkerPool {
    /// The most basic WorkerPool configuration
    pub fn new(num_workers: usize) -> Result<WorkerPool, SchedulerError> {
        Builder::new().num_workers(num_workers).build()
    }

    /// Add some name to main process
    pub fn with_name(name: String, num_workers: usize) -> Result<WorkerPool, SchedulerError> {
        Builder::new().num_workers(num_workers).name(name).build()
    }

    pub fn execute<F>(&self, job: F, args: ExecuteRequest) -> Result<(), SchedulerError>
    where
        F: FnOnce(ExecuteRequest) + Send + 'static,
    {
        let job_count = self.shared_data.job_counter.fetch_add(1, Ordering::SeqCst);
        self.shared_data.queued_count.fetch_add(1, Ordering::SeqCst);
        let mut task = match args.clone().task {
            Some(t) => t,
            None => Err(SchedulerError::TaskExecutionError(
                "task execution must include valid data".to_string(),
            ))?,
        };
        task.id = job_count.to_string();
        let request = ExecuteRequest { task: Some(task) };

        #[cfg(feature = "stats")]
        increment_task(WorkerPoolTaskType::Queued);

        if self.jobs.is_some() {
            self.jobs
                .as_ref()
                .unwrap()
                .send(Job {
                    id: job_count,
                    data: request,
                    job: Box::new(job),
                })
                .expect("WorkerPool::execute unable to send job into queue.");
        } else {
            Err(SchedulerError::TaskExecutionError(
                "Couldn't excute the job as the WorkerPool is not initalized properly".to_string(),
            ))?
        }

        Ok(())
    }

    pub fn queued_count(&self) -> usize {
        self.shared_data.queued_count.load(Ordering::Relaxed)
    }

    pub fn active_count(&self) -> usize {
        self.shared_data.active_count.load(Ordering::SeqCst)
    }

    pub fn max_count(&self) -> usize {
        self.shared_data.max_thread_count.load(Ordering::Relaxed)
    }

    pub fn panic_count(&self) -> usize {
        self.shared_data.panic_count.load(Ordering::Relaxed)
    }

    pub fn join(&self) {
        // fast path requires no mutex
        if !self.shared_data.has_work() {
            return;
        }

        let generation = self.shared_data.join_generation.load(Ordering::SeqCst);
        let mut lock = self.shared_data.empty_trigger.lock().unwrap();

        while generation == self.shared_data.join_generation.load(Ordering::Relaxed)
            && self.shared_data.has_work()
        {
            if self.shared_data.force_shutdown.load(Ordering::SeqCst) {
                log::debug!("Force shutdown activated, exiting join",);
                return;
            }
            lock = self.shared_data.empty_condvar.wait(lock).unwrap();
        }

        // increase generation if we are the first thread to come out of the loop
        if self
            .shared_data
            .join_generation
            .compare_exchange(
                generation,
                generation.wrapping_add(1),
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .is_err()
        {
            // Log the failure or take other actions.
            log::warn!("Failed to update join_generation");
        }
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        drop(self.jobs.take());

        self.join();
    }
}

impl Clone for WorkerPool {
    /// Cloning a pool will create a new handle to the pool.
    /// The behavior is similar to [Arc](https://doc.rust-lang.org/stable/std/sync/struct.Arc.html).
    ///
    /// We could for example submit jobs from multiple threads concurrently.
    ///
    /// ```rust,no_run
    /// use protot::core::worker_pool::WorkerPool;
    /// use protot::internal::sylklabs::scheduler::v1::ExecuteRequest;
    /// use std::thread;
    /// use std::sync::mpsc::channel;
    ///
    /// let pool = WorkerPool::with_name("clone example".into(), 2).unwrap();
    ///
    /// let results = (0..2)
    ///     .map(|i| {
    ///         let pool= pool.clone();
    ///         thread::spawn(move || {
    ///             let (tx, rx) = channel();
    ///             for i in 1..12 {
    ///                 let tx = tx.clone();
    ///                 pool.execute(move |args| {
    ///                     tx.send(i).expect("channel will be waiting");
    ///                 }, ExecuteRequest {..Default::default()});
    ///             }
    ///             drop(tx);
    ///             if i == 0 {
    ///                 rx.iter().fold(0, |accumulator, element| accumulator + element)
    ///             } else {
    ///                 rx.iter().fold(1, |accumulator, element| accumulator * element)
    ///             }
    ///         })
    ///     })
    ///     .map(|join_handle| join_handle.join().expect("collect results from threads"))
    ///     .collect::<Vec<usize>>();
    ///
    /// assert_eq!(vec![66, 39916800], results);
    /// ```
    fn clone(&self) -> WorkerPool {
        WorkerPool {
            jobs: self.jobs.clone(),
            shared_data: self.shared_data.clone(),
            executors: self.executors.clone(),
            workers: self.workers.clone(),
        }
    }
}

/// Create a thread pool with one thread per CPU.
/// On machines with hyperthreading,
/// this will create one thread per hyperthread.
impl Default for WorkerPool {
    fn default() -> Self {
        WorkerPool::new(num_cpus::get()).expect("failed to create defualt thread pool!")
    }
}

impl fmt::Debug for WorkerPool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("WorkerPool")
            .field("name", &self.shared_data.name)
            .field("queued_count", &self.queued_count())
            .field("active_count", &self.active_count())
            .field("max_count", &self.max_count())
            .finish()
    }
}

pub struct WorkerPoolSharedData {
    name: Option<String>,
    pub job_receiver: Arc<Mutex<Receiver<Job<'static, ExecuteRequest>>>>,
    empty_trigger: Mutex<()>,
    empty_condvar: Condvar,
    join_generation: AtomicUsize,
    queued_count: AtomicUsize,
    active_count: AtomicUsize,
    max_thread_count: AtomicUsize,
    panic_count: AtomicUsize,
    stack_size: Option<usize>,
    job_counter: AtomicUsize,
    // workers: Vec<Arc<dyn Worker>>,
    force_shutdown: AtomicBool,
}

impl WorkerPoolSharedData {
    pub fn new(
        num_threads: usize,
        thread_stack_size: Option<usize>,
        receiver: Receiver<Job<'static, ExecuteRequest>>,
        force_shutdown: bool,
        pool_name: Option<String>,
    ) -> Arc<Self> {
        Arc::new(Self {
            name: pool_name,
            job_receiver: Arc::new(Mutex::new(receiver)),
            empty_condvar: Condvar::new(),
            empty_trigger: Mutex::new(()),
            join_generation: AtomicUsize::new(0),
            queued_count: AtomicUsize::new(0),
            active_count: AtomicUsize::new(0),
            max_thread_count: AtomicUsize::new(num_threads),
            panic_count: AtomicUsize::new(0),
            job_counter: AtomicUsize::new(0),
            force_shutdown: AtomicBool::new(force_shutdown),
            stack_size: thread_stack_size,
        })
    }

    pub fn has_work(&self) -> bool {
        self.queued_count.load(Ordering::SeqCst) > 0 || self.active_count.load(Ordering::SeqCst) > 0
    }

    /// Notify all observers joining this pool if there is no more work to do.
    pub fn no_work_notify_all(&self) {
        if !self.has_work() {
            let p = self
                .empty_trigger
                .lock()
                .expect("Unable to notify all joining threads");
            drop(p);
            self.empty_condvar.notify_all();
        }
    }

    pub fn get_name(&self) -> Option<String> {
        self.name.clone()
    }

    pub fn get_stack_size(&self) -> Option<usize> {
        self.stack_size
    }

    pub fn load_thread_metrics(&self) -> (usize, usize) {
        (
            self.active_count.load(Ordering::Acquire),
            self.max_thread_count.load(Ordering::Relaxed),
        )
    }

    pub fn decrement_thread_active(&self) {
        self.active_count.fetch_sub(1, Ordering::SeqCst);
        #[cfg(feature = "stats")]
        {
            metrics::set_worker_pool_metric(
                WorkerPoolMetricType::Active,
                self.active_count.load(Ordering::SeqCst),
            );
        }
    }

    pub fn process_new_excution_metrics(&self) {
        self.active_count.fetch_add(1, Ordering::SeqCst);
        self.queued_count.fetch_sub(1, Ordering::SeqCst);

        #[cfg(feature = "stats")]
        {
            let active_workers = self.active_count.load(Ordering::Acquire);
            metrics::set_worker_pool_metric(metrics::WorkerPoolMetricType::Active, active_workers);
        }
    }
}

// Function to initialize the pool
pub fn initialize_thread_pool(
    num_threads: usize,
    receiver: Receiver<Job<'static, ExecuteRequest>>,
    force_shutdown: bool,
    pool_name: Option<String>,
) -> (Arc<WorkerPoolSharedData>, Vec<Arc<dyn Worker>>) {
    let shared_data =
        WorkerPoolSharedData::new(num_threads, None, receiver, force_shutdown, pool_name);
    let shared_data_clone = Arc::clone(&shared_data);

    let mut workers = Vec::new();
    for id in 0..num_threads {
        let worker =
            Arc::new(LocalWorker::new(id, Arc::clone(&shared_data_clone))) as Arc<dyn Worker>;
        worker.spawn();
        workers.push(worker);
    }

    (shared_data, workers)
}

fn spawn_in_pool(worker_id: usize, shared_data: Arc<WorkerPoolSharedData>) {
    let binding = Arc::clone(&shared_data);
    let worker = Arc::new(LocalWorker::new(worker_id, Arc::clone(&binding))) as Arc<dyn Worker>;

    worker.spawn();
}
