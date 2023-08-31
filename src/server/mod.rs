use std::{
    pin::Pin,
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::{
    core::{
        job::{Job, Thunk},
        worker_pool::{self, WorkerPool},
    },
    internal::sylklabs::{
        core::Task,
        scheduler::v1::{
            scheduler_message,
            scheduler_service_server::{SchedulerService, SchedulerServiceServer},
            scheduler_worker_service_server::{
                SchedulerWorkerService, SchedulerWorkerServiceServer,
            },
            AssignTaskRequest, ExecuteRequest, ExecuteResponse, ScheduleRequest, ScheduleResponse,
            SchedulerMessage, WorkerMessage,
        },
    },
    logger,
};
use futures::{Stream, StreamExt};
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::{mpsc, Mutex},
};
use tokio_stream::wrappers::ReceiverStream; // Import the ReceiverStream type
use tonic::{transport::Server, Request, Response, Status};
use tonic_types::{ErrorDetails, StatusExt};

// Implement the gRPC service
#[derive(Debug, Default)]
pub struct SchedulerServer {}

#[tonic::async_trait]
impl SchedulerWorkerService for SchedulerServer {
    type CommunicateStream = Pin<Box<dyn Stream<Item = Result<SchedulerMessage, Status>> + Send>>;

    async fn communicate(
        &self,
        request: Request<tonic::Streaming<WorkerMessage>>,
    ) -> Result<Response<Self::CommunicateStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        // Spawn a new task to process incoming messages and send responses
        tokio::spawn(async move {
            let mut stream = request.into_inner();
            while let Some(worker_message) = stream.next().await {
                match worker_message {
                    Ok(message) => {
                        let response = match message.worker_message_type {
                            registration_request => SchedulerMessage {
                                scheduler_message_type: Some(
                                    scheduler_message::SchedulerMessageType::AssignTask(
                                        AssignTaskRequest {
                                            task: Some(Task {
                                                id: "some-task-id".to_string(),
                                                ..Default::default()
                                            }),
                                        },
                                    ),
                                ),
                            },
                        };
                        // Process the worker message and create a response
                        // let response = ;
                        if let Err(_) = tx.send(Ok(response)).await {
                            // Handle error sending response
                        }
                    }
                    Err(status) => {
                        // Handle error from worker
                        // You might want to log or take some action here
                    }
                }
            }
            // This line isn't necessary in newer versions of tokio
            // tx.close_channel();
        });
        let response_stream: Self::CommunicateStream = Box::pin(ReceiverStream::new(rx)); // Use ReceiverStream

        Ok(Response::new(response_stream))
    }
}

pub struct SharedData {
    worker_pool: Arc<Mutex<WorkerPool>>, // Use tokio::sync::mpsc::Sender
}

// Function to start the gRPC server
#[tokio::main]
pub async fn start_grpc_server(
    port: i32,
    pool: worker_pool::WorkerPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Channel for signaling shutdown
    let (tx, mut rx) = mpsc::channel(1);
    // Channel for signaling gRPC server shutdown
    // let (tx, mut rx) = tokio::sync::oneshot::channel();

    // Init the worker pool
    let cloned_pool = pool.clone();
    let shared_data = Arc::new(SharedData {
        worker_pool: Arc::new(Mutex::new(pool)), // Pass the actual worker pool here
    });

    // gRPC server setup
    let addr = format!("127.0.0.1:{}", port).as_str().parse()?;

    // SchedulerWorkerService - for communication of workers to scheduler
    let scheduler_worker_svc = SchedulerServer {};
    let svc = SchedulerWorkerServiceServer::new(scheduler_worker_svc);

    // SchedulerService - admin service for communicating with scheduler by clients.
    let admin_service =
        SchedulerServiceServer::new(SchedulerAdminService::new(shared_data.clone()));

    // This AtomicBool will be used to track if the interrupt was previously received
    let interrupt_received = Arc::new(AtomicBool::new(false));

    // Spawn a new task to listen for shutdown signals
    tokio::spawn(async move {
        let mut stream = signal(SignalKind::interrupt()).unwrap();
        loop {
            stream.recv().await;
            if interrupt_received.load(Ordering::Relaxed) {
                // Second Ctrl+C received, forcibly terminate
                logger::log(
                    logger::LogLevel::DEBUG,
                    "second interrupt received, force quitting...",
                );
                process::exit(1);
            } else {
                // First Ctrl+C received, initiate graceful shutdown
                logger::log(logger::LogLevel::DEBUG, "server interrupted");

                interrupt_received.store(true, Ordering::Relaxed);

                let pool = cloned_pool.clone();
                // Explicitly dropping worker pool to kick off cleanup
                // Drop the pool in a new task
                // tokio::spawn(async move {
                // drop(pool);
                std::thread::spawn(move || {
                    drop(pool);
                });
                // });
                // Send the signal to shutdown the server
                tx.send(()).await.unwrap();
            }
        }
    });

    let server = Server::builder()
        .add_service(svc)
        .add_service(admin_service)
        .serve_with_shutdown(addr, async {
            // Wait for the signal to start the shutdown
            rx.recv().await;
        });

    server.await?;

    Ok(())
}

pub struct SchedulerAdminService {
    shared_data: Arc<SharedData>,
}

impl SchedulerAdminService {
    fn new(shared_data: Arc<SharedData>) -> Self {
        Self { shared_data }
    }
}

// impl TaskArguments for ExecuteRequest {
//     fn get_data(&self) -> &dyn TaskArguments {

//     }
// }

#[tonic::async_trait]
impl SchedulerService for SchedulerAdminService {
    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteResponse>, Status> {
        let shared_data = self.shared_data.as_ref().clone();
        let req = request.into_inner().clone();
        let task_name = req.task.as_ref().unwrap().id.clone();
        println!("task->{}", task_name);

        // Clone the shared data for the closure
        let cloned_shared = shared_data.worker_pool.as_ref().lock().await;

        // Capture the 'o' value from the lock before the async block
        let o_clone = {
            let executor = cloned_shared.executors.clone();
            executor
        };
        // Execute the logic using the cloned shared data and the cloned request
        match cloned_shared.execute(
            move |args| {
                let logic = o_clone.lock().unwrap();
                logger::log(
                    logger::LogLevel::DEBUG,
                    format!("executing task: {}", task_name).as_str(),
                );
                logic.get_executor(&task_name).unwrap().execute(args);
                // logic.execute(args)
                // .get_executor(&task_name).expect("Executor not found in registry").execute(args)
            },
            req,
        ) {
            Ok(res) => {
                Ok(Response::new(ExecuteResponse {
                    ..Default::default()
                }))
            }
            Err(err) => {
                let mut err_details = ErrorDetails::new();
                err_details
                    .add_precondition_failure_violation(
                        "EXECUTOR",
                        "WorkerPool",
                        format!("failed to execute task on worker pool: {:?}", err),
                    )
                    .add_help_link("documentation", "https://protot.io/docs/help")
                    .set_localized_message("en-US", "error executing task");
                // Generate error status
                let status = Status::with_error_details(
                    tonic::Code::FailedPrecondition,
                    "request contains invalid arguments",
                    err_details,
                );

                Err(status)    
            }
        }

        
    }

    async fn schedule(
        &self,
        request: Request<ScheduleRequest>,
    ) -> Result<Response<ScheduleResponse>, Status> {
        let shared_data = self.shared_data.clone();
        // let worker_pool = shared_data.worker_pool.lock().await;
        // worker_pool.execute(|args| println!("excute from grpc server"), ExecuteRequest { task: Some(Task { ..Default::default() }) });
        // logger::log(
        //     logger::LogLevel::DEBUG,
        //     format!("got schedule request : {}", worker_pool.max_count()).as_str(),
        // );
        let mut err_details = ErrorDetails::new();
        // err_details.bad_request("name", "name cannot be empty");
        err_details
            .add_precondition_failure_violation(
                "unimplemented",
                "schedule",
                "admin schedule server isnt supporting scheduling of tasks",
            )
            .add_help_link("description of link", "https://resource.example.local")
            .set_localized_message("en-US", "message for the user");
        // Generate error status
        let status = Status::with_error_details(
            tonic::Code::Unimplemented,
            "request contains invalid arguments",
            err_details,
        );
        Err(status)
    }
}
