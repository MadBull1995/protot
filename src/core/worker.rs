use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        Arc,
    },
    thread,
};

use crate::logger;

use super::worker_pool::{Sentinel, WorkerPoolSharedData};

#[derive(Debug, Clone, Copy)]
pub enum WorkerType {
    Local,
    Remote,
}

impl Default for WorkerType {
    fn default() -> Self {
        WorkerType::Local
    }
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

pub struct LocalWorker {
    id: usize,
    is_active: AtomicBool,
    jobs_processed: AtomicUsize,
    shared_data: Arc<WorkerPoolSharedData>,
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
            id,
            shared_data: shared_data,
            is_active: AtomicBool::new(false),
            jobs_processed: AtomicUsize::new(0),
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
        let pool_name = shared_data_clone
            .get_name()
            .unwrap_or_else(|| "".to_string());

        logger::log(
            logger::LogLevel::INFO,
            format!("[{}][{}] worker starting", pool_name, worker_id).as_str(),
        );

        builder
            .spawn(move || {
                // Will spawn a new thread on panic unless it is canceled.
                let binding = shared_data_clone.clone();
                let sentinel = Sentinel::new(&binding);
                loop {
                    // Shutdown this thread if the pool has become smaller.
                    let (thread_counter_val, max_thread_count_val) = binding.load_thread_metrics();
                    if thread_counter_val >= max_thread_count_val {
                        break;
                    }

                    // Job retrieval logic would go here, possibly involving more shared state.
                    let message = {
                        // Only lock jobs for the time it takes
                        // to get a job, not run it.
                        let lock = binding
                            .job_receiver
                            .lock()
                            .expect("Worker thread unable to lock job_receiver");
                        lock.recv()
                    };

                    let job = match message {
                        Ok(job) => job,
                        // The ThreadPool was dropped.
                        Err(_) => {
                            logger::log(
                                logger::LogLevel::DEBUG,
                                format!(
                                    "[{}][{}] disconnected; shutting down.",
                                    pool_name, worker_id
                                )
                                .as_str(),
                            );
                            break;
                        }
                    };

                    binding.process_new_excution_metrics();
                    logger::log(
                        logger::LogLevel::INFO,
                        format!(
                            "[{}][{}] worker executing job -> {}",
                            pool_name,
                            worker_id,
                            job.get_id()
                        )
                        .as_str(),
                    );
                    println!("{:?}", job.get_id());
                    // Execute the job and update counters.
                    job.execute_job();

                    binding.decrement_thread_active();

                    // Notify condition variable, or any other logic for signaling that work is done.
                    binding.no_work_notify_all();
                }
                sentinel.cancel(); // Cancel the sentinel.
            })
            .unwrap();
    }
}

// pub struct RemoteWorker {
//     id: usize,
//     shared_data: Arc<WorkerPoolSharedData>,
//     grpc_client: GrpcClient,  // Placeholder for a real gRPC client
// }

// impl Worker for RemoteWorker {
//     fn new(id: usize, shared_data: Arc<WorkerPoolSharedData>) -> Self {
//         RemoteWorker {
//             id,
//             shared_data,
//             grpc_client: GrpcClient::new(),  // Placeholder for real initialization
//         }
//     }

//     fn spawn(&self) {
//         // implementation for spawning a remote worker
//     }
// }

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
        let shared_data =
            WorkerPoolSharedData::new_partial(1, None, rx, false, Some(String::from("Test")));
        let mut shared_data_clone = Arc::clone(&shared_data);

        // Create the worker
        // let worker: LocalWorker = LocalWorker::new(1, shared_data.clone());
        let mut workers = Vec::new();
        let worker =
            Arc::new(LocalWorker::new(1, Arc::clone(&shared_data_clone))) as Arc<dyn Worker>;

        worker.spawn();
        workers.push(worker);

        // Safely populate the `workers` field
        if let Some(data) = Arc::get_mut(&mut shared_data_clone) {
            data.populate_workers(workers);
        }
        // Sleep briefly to allow worker thread to execute the job
        thread::sleep(std::time::Duration::from_millis(100));

        // Assertions
        assert_eq!(shared_data.has_work(), false);
        // ...
    }
}
