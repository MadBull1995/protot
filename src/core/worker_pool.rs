use std::{
    collections::HashMap,
    fmt,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc, Condvar, Mutex,
    },
    thread,
};

use super::{
    job::Job,
    worker::{LocalWorker, Worker, WorkerType},
};

// Lib modules
use crate::{internal::sylklabs::scheduler::v1::ExecuteRequest, logger, SchedulerError};

// Trait for task execution
pub trait TaskExecutor: Send + Sync + 'static {
    fn execute(&self, args: ExecuteRequest);
}

// Struct to hold task executions and their argument implementations
pub struct TaskRegistry {
    registry: HashMap<String, Box<dyn TaskExecutor>>,
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

    // pub fn execute_task(
    //     &self,
    //     task_name: &str,
    //     args: ExecuteRequest,
    // ) -> Result<(), &'static str> {
    //     if let Some(executor) = self.registry.get(task_name) {
    //         executor.execute(args);
    //         Ok(())
    //     } else {
    //         Err("Task not found")
    //     }
    // }

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
    shared_data: &'a Arc<WorkerPoolSharedData>,
    active: bool,
}

impl<'a> Sentinel<'a> {
    pub fn new(shared_data: &'a Arc<WorkerPoolSharedData>) -> Sentinel<'a> {
        Sentinel {
            shared_data,
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
            if thread::panicking() {
                self.shared_data.panic_count.fetch_add(1, Ordering::SeqCst);
            }
            self.shared_data.no_work_notify_all();
            logger::log(
                logger::LogLevel::ERROR,
                "sentinel droped but havent spawned any new worker",
            )
            // TODO add sentinel spwan
            // spawn_in_pool(self.shared_data.clone())
        }
    }
}

#[derive(Clone, Default)]
pub struct Builder {
    num_threads: Option<usize>,
    thread_name: Option<String>,
    thread_stack_size: Option<usize>,
    workers_type: WorkerType,
    force_shutdown: Option<bool>,
    executers: Option<bool>,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            num_threads: None,
            thread_name: None,
            thread_stack_size: None,
            force_shutdown: None,
            executers: None,
            ..Default::default()
        }
    }

    pub fn grpc_workers(mut self) -> Builder {
        self.workers_type = WorkerType::Remote;
        self
    }

    pub fn num_threads(mut self, num_threads: usize) -> Builder {
        self.num_threads = Some(num_threads);
        self
    }

    pub fn thread_name(mut self, thread_name: String) -> Builder {
        self.thread_name = Some(thread_name);
        self
    }

    pub fn thread_stack_size(mut self, thread_stack_size: usize) -> Builder {
        self.thread_stack_size = Some(thread_stack_size);
        self
    }

    pub fn with_force_shutdown(mut self) -> Builder {
        self.force_shutdown = Some(true);
        self
    }

    pub fn executors(mut self) -> Builder {
        self.executers = Some(true);
        self
    }

    pub fn build(self) -> Result<WorkerPool, SchedulerError> {
        let num_threads = self.num_threads.unwrap_or_else(num_cpus::get);
        let force_shutdown = self.force_shutdown.unwrap_or_else(|| false);
        let (tx, rx) = channel::<Job<'static, ExecuteRequest>>();

        let shared_data = match self.workers_type {
            WorkerType::Local => {
                initialize_thread_pool(num_threads, rx, force_shutdown, self.thread_name)
            }
            WorkerType::Remote => Err(SchedulerError::PoolCreationError(
                "Cant serve gRPC based workers currently".to_string(),
            ))?,
        };
        let registry = Arc::new(Mutex::new(TaskRegistry::new()));

        Ok(WorkerPool {
            jobs: Some(tx),
            shared_data,
            executors: registry,
        })
    }
}

pub struct WorkerPool {
    jobs: Option<Sender<Job<'static, ExecuteRequest>>>,
    shared_data: Arc<WorkerPoolSharedData>,
    pub executors: Arc<Mutex<TaskRegistry>>,
}

impl WorkerPool {
    pub fn new(num_threads: usize) -> Result<WorkerPool, SchedulerError> {
        Builder::new().num_threads(num_threads).build()
    }

    pub fn with_name(name: String, num_threads: usize) -> Result<WorkerPool, SchedulerError> {
        Builder::new()
            .num_threads(num_threads)
            .thread_name(name)
            .build()
    }

    pub fn execute<F>(&self, job: F, args: ExecuteRequest) -> Result<(), SchedulerError>
    where
        F: FnOnce(ExecuteRequest) + Send + 'static,
    {
        let job_count = self.shared_data.job_counter.fetch_add(1, Ordering::SeqCst);
        self.shared_data.queued_count.fetch_add(1, Ordering::SeqCst);
        let mut task = match args.clone().task {
            Some(t) => { t },
            None => Err(SchedulerError::TaskExecutionError("task execution must include valid data".to_string()))?
        };
        task.id = job_count.to_string();
        let request = ExecuteRequest {
            task: Some(task)
        };
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
            Err(SchedulerError::TaskExecutionError("Couldn't excute the job as the WorkerPool is not initalized properly".to_string()))?
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
        if self.shared_data.has_work() == false {
            return;
        }

        let generation = self.shared_data.join_generation.load(Ordering::SeqCst);
        let mut lock = self.shared_data.empty_trigger.lock().unwrap();

        while generation == self.shared_data.join_generation.load(Ordering::Relaxed)
            && self.shared_data.has_work()
        {
            if self.shared_data.force_shutdown.load(Ordering::SeqCst) {
                logger::log(
                    logger::LogLevel::DEBUG,
                    "Force shutdown activated, exiting join",
                );
                return;
            }
            lock = self.shared_data.empty_condvar.wait(lock).unwrap();
        }

        // increase generation if we are the first thread to come out of the loop
        if self.shared_data.join_generation.compare_exchange(
            generation,
            generation.wrapping_add(1),
            Ordering::SeqCst,
            Ordering::SeqCst,
        ).is_err() {
            // Log the failure or take other actions.
            logger::log(logger::LogLevel::WARN, "Failed to update join_generation");
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
    /// use proto_tasker::core::worker_pool::WorkerPool;
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
    ///                 pool.execute(move || {
    ///                     tx.send(i).expect("channel will be waiting");
    ///                 });
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
    workers: Vec<Arc<dyn Worker>>,
    force_shutdown: AtomicBool,
}

impl WorkerPoolSharedData {
    pub fn new_partial(
        num_threads: usize,
        thread_stack_size: Option<usize>,
        receiver: Receiver<Job<'static, ExecuteRequest>>,
        force_shutdown: bool,
        pool_name: Option<String>,
    ) -> Arc<Self> {
        Arc::new(Self {
            name: pool_name,
            workers: Vec::new(), // empty for now
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

    pub fn populate_workers(&mut self, workers: Vec<Arc<dyn Worker>>) {
        self.workers = workers;
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
    }

    pub fn process_new_excution_metrics(&self) {
        self.active_count.fetch_add(1, Ordering::SeqCst);
        self.queued_count.fetch_sub(1, Ordering::SeqCst);
    }
}

// Function to initialize the pool
pub fn initialize_thread_pool(
    num_threads: usize,
    receiver: Receiver<Job<'static, ExecuteRequest>>,
    force_shutdown: bool,
    pool_name: Option<String>,
) -> Arc<WorkerPoolSharedData> {
    let shared_data =
        WorkerPoolSharedData::new_partial(num_threads, None, receiver, force_shutdown, pool_name);
    let mut shared_data_clone = Arc::clone(&shared_data);

    let mut workers = Vec::new();
    for id in 0..num_threads {
        let worker =
            Arc::new(LocalWorker::new(id, Arc::clone(&shared_data_clone))) as Arc<dyn Worker>;
        worker.spawn();
        workers.push(worker);
    }

    // Safely populate the `workers` field
    if let Some(data) = Arc::get_mut(&mut shared_data_clone) {
        data.populate_workers(workers);
    }

    shared_data
}
