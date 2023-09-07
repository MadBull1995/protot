#[allow(unused_imports)]
use crate::{logger, server::metrics};
#[allow(unused_imports)]
use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        mpsc::RecvTimeoutError,
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use super::worker_pool::{Sentinel, WorkerPoolSharedData};

#[derive(Default, Debug, Clone, Copy)]
pub enum WorkerType {
    #[default]
    Local,
    Remote,
}

pub fn create_worker(
    worker_type: WorkerType,
    id: usize,
    shared_data: Arc<WorkerPoolSharedData>,
) -> Box<dyn Worker> {
    match worker_type {
        WorkerType::Local => Box::new(LocalWorker::new(id, shared_data.clone())),
        // WorkerType::Remote => Box::new(RemoteWorker::new(id, shared_data.clone())),
        _ => panic!("worker type not supported"),
    }
}

pub trait Worker: Send + Sync {
    /// Initialize a new worker.
    fn new(id: usize, shared_data: Arc<WorkerPoolSharedData>) -> Self
    where
        Self: Sized;

    /// Spawn the worker to start processing jobs.
    fn spawn(&self);
}

#[cfg(not(feature = "stats"))]
#[allow(unused_variables, unused_mut)]
fn process_task(
    binding: Arc<WorkerPoolSharedData>,
    sentinel: Sentinel<'_>,
    mut total_time: Duration,
    mut active_time: Duration,
    worker_id: usize,
) {
    loop {
        // Shutdown this thread if the pool has become smaller.
        let (thread_counter_val, max_thread_count_val) = binding.load_thread_metrics();
        if thread_counter_val >= max_thread_count_val {
            break;
        }

        // Job retrieval logic would go here, possibly involving more shared state.
        let message_result = {
            // Only lock jobs for the time it takes
            // to get a job, not run it.
            let lock = binding
                .job_receiver
                .lock()
                .expect("Worker thread unable to lock job_receiver");

            lock.recv()
        };

        let job = match message_result {
            Err(e) => {
                println!("disconnecting: {:?}", thread::current().name().unwrap());
                break;
            }
            Ok(j) => j,
        };

        binding.process_new_excution_metrics();

        log::info!("worker executing job -> {}", job.get_id());

        job.execute_job();

        binding.decrement_thread_active();

        // Notify condition variable, or any other logic for signaling that work is done.
        binding.no_work_notify_all();
    }
    sentinel.cancel();
}

#[cfg(feature = "stats")]
fn process_task(
    binding: Arc<WorkerPoolSharedData>,
    sentinel: Sentinel<'_>,
    mut total_time: Duration,
    mut active_time: Duration,
    worker_id: usize,
) {
    loop {
        // Shutdown this thread if the pool has become smaller.
        let (thread_counter_val, max_thread_count_val) = binding.load_thread_metrics();
        if thread_counter_val >= max_thread_count_val {
            break;
        }
        let timeout = Duration::from_millis(1); // 20 ms timeout, adjust as needed

        // Record the loop start time
        let loop_start_time = Instant::now();

        // Job retrieval logic would go here, possibly involving more shared state.
        let message_result = {
            // Only lock jobs for the time it takes
            // to get a job, not run it.
            let lock = binding
                .job_receiver
                .lock()
                .expect("Worker thread unable to lock job_receiver");

            lock.recv_timeout(timeout)
        };

        match message_result {
            Ok(job) => {
                // Process the job message
                binding.process_new_excution_metrics();
                let task_start_time = Instant::now();

                log::info!("worker executing job -> {}", job.get_id());

                metrics::decrement_task_queue();

                // Execute the job and update counters.
                job.execute_job();

                let task_duration = task_start_time.elapsed();
                active_time += task_duration;

                metrics::increment_task(metrics::WorkerPoolTaskType::Executed);

                binding.decrement_thread_active();

                // Notify condition variable, or any other logic for signaling that work is done.
                binding.no_work_notify_all();
            }
            Err(RecvTimeoutError::Timeout) => {
                // Update idle time metric here
                let idle_duration = loop_start_time.elapsed();
                total_time += idle_duration;
            }
            Err(RecvTimeoutError::Disconnected) => {
                // The ThreadPool was dropped.
                println!("disconnecting: {:?}.", thread::current().name().unwrap());
                break;
            }
        }

        let loop_duration = loop_start_time.elapsed();
        total_time += loop_duration;

        metrics::update_worker_utilization(worker_id, active_time, total_time);
    }
    sentinel.cancel();
}

#[allow(dead_code)]
pub struct LocalWorker {
    id: usize,
    is_active: AtomicBool,
    jobs_processed: AtomicUsize,
    shared_data: Arc<WorkerPoolSharedData>,
    active_time: AtomicUsize,
    idle_time: AtomicUsize,
}

impl Worker for LocalWorker {
    /// Creates a new `Worker` with a given `id` and `shared_data`.
    ///
    /// # Parameters
    ///
    /// * `id`: The identifier for the new worker.
    /// * `shared_data`: An `Arc` wrapped `WorkerPoolSharedData` instance shared among all workers.
    ///
    /// # Returns
    ///
    /// Returns a new `Worker` instance.
    fn new(id: usize, shared_data: Arc<WorkerPoolSharedData>) -> LocalWorker {
        LocalWorker {
            is_active: AtomicBool::new(false),
            jobs_processed: AtomicUsize::new(0),
            active_time: AtomicUsize::new(0),
            idle_time: AtomicUsize::new(0),
            shared_data,
            id,
            // ... initialize other fields
        }
    }

    /// Spawns the worker thread and makes it start waiting for jobs.
    ///
    /// This function creates a new background thread and passes the `shared_data` to it.
    /// The thread will continuously poll for new jobs and execute them.
    ///
    /// # Panics
    ///
    /// This function will panic if the spawned thread panics.
    #[allow(unused_mut)]
    fn spawn(&self) {
        let shared_data_clone = self.shared_data.clone();
        let mut builder = thread::Builder::new();

        // Access shared data for name, stack size, etc.
        if let Some(ref name) = shared_data_clone.get_name() {
            builder = builder.name(format!("{}-{}", name.clone(), self.id));
        }
        if let Some(stack_size) = shared_data_clone.get_stack_size() {
            builder = builder.stack_size(stack_size);
        }

        let worker_id = self.id;

        builder
            .spawn(move || {
                log::info!("worker starting",);
                let mut total_time = Duration::new(0, 0);
                let mut active_time = Duration::new(0, 0);
                // Will spawn a new thread on panic unless it is canceled.
                let binding = shared_data_clone.clone();
                let sentinel = Sentinel::new(worker_id, &binding);
                process_task(
                    binding.clone(),
                    sentinel,
                    total_time,
                    active_time,
                    worker_id,
                );
            })
            .unwrap();
    }
}

#[cfg(test)]
mod tests {

    use crate::core::job::Job;
    use crate::internal::sylklabs::scheduler::v1::ExecuteRequest;

    use super::*;
    use std::sync::mpsc::channel;

    #[test]
    fn test_local_worker_spawn() {
        let (_, rx) = channel::<Job<'static, ExecuteRequest>>();

        // Create mock shared data instance
        let shared_data: Arc<WorkerPoolSharedData> =
            WorkerPoolSharedData::new(1, None, rx, false, Some(String::from("Test")));
        let mut shared_data_clone = Arc::clone(&shared_data);

        // Create the worker
        // let worker: LocalWorker = LocalWorker::new(1, shared_data.clone());
        let mut workers = Vec::new();
        let worker =
            Arc::new(LocalWorker::new(1, Arc::clone(&shared_data_clone))) as Arc<dyn Worker>;

        worker.spawn();
        workers.push(worker);

        // Sleep briefly to allow worker thread to execute the job
        thread::sleep(std::time::Duration::from_millis(100));

        // Assertions
        assert_eq!(shared_data.has_work(), false);
    }
}
