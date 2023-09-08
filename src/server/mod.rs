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

pub mod metrics;

use std::{
    pin::Pin,
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    }, time::{Duration, Instant}, collections::HashMap,
};
use std::future::Future;
use tokio_util::sync::CancellationToken;

use crate::{internal::protot::{scheduler::v1::{Ack, WorkerChannelStatus, worker_message::{self, WorkerMessageType}}, core::NodeType}, core::{grpc_executor::GrpcSharedState, load_balancer::{LoadBalancer, RoundRobinBalancer}}};
#[allow(unused_imports)]
use crate::{
    core::worker_pool::{self, WorkerPool},
    internal::protot::{
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
use futures::{Stream, StreamExt, TryFutureExt};
use log::{info, error, debug};
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::{mpsc::{self, Receiver, Sender}, Mutex}, select, time::{timeout, sleep},
};
use tokio_stream::wrappers::ReceiverStream; // Import the ReceiverStream type
use tonic::{transport::Server, Request, Response, Status};
use tonic_types::{ErrorDetails, StatusExt};

// Implement the gRPC service
// #[derive(Default)]
pub struct SchedulerServer<B: LoadBalancer> {
    shared_state: Arc<GrpcSharedState<B>>,
}

impl<B: LoadBalancer> SchedulerServer<B> {
    pub fn new(shared_state: Arc<GrpcSharedState<B>>) -> Self {
        Self { shared_state }
    }
}

#[tonic::async_trait]
impl<B: LoadBalancer> SchedulerWorkerService for SchedulerServer<B> {
    type CommunicateStream = Pin<Box<dyn Stream<Item = Result<SchedulerMessage, Status>> + Send>>;

    async fn communicate(
        &self,
        request: Request<tonic::Streaming<WorkerMessage>>,
    ) -> Result<Response<Self::CommunicateStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let (tx_cancel,mut rx_cancel) = tokio::sync::mpsc::channel::<()>(1);
        let tx_binding = tx.clone();

        let shared_state = self.shared_state.clone();
        // Spawn a new task to process incoming messages and send responses
        tokio::spawn(async move {
            let mut stream = request.into_inner();
            let mut registered_worker_id: Option<String> = None;  // Add this line

            while let Some(worker_message) = stream.next().await {
                match worker_message {
                    Ok(message) => {
                        let response = match message.worker_message_type {
                            Some(WorkerMessageType::Heartbeat(pong)) => {
                                let binding = registered_worker_id.clone();
                                debug!("[{}] Got heartbeat from worker: {:?}", binding.clone().unwrap(), pong);
                                if binding.is_some() {
                                    let shared_heartbeat = shared_state.worker_heartbeat.clone();  // assuming shared_data contains worker_heartbeat
                                    let mut heartbeats = shared_heartbeat.lock().await;
                                    heartbeats.insert(binding.unwrap().clone(), Instant::now());
                                }
                                SchedulerMessage::default()
                            }
                            Some(WorkerMessageType::Completion(task_completion)) => {
                                info!("got task completion event: {:?}", task_completion);
                                SchedulerMessage::default()
                            }
                            Some(WorkerMessageType::Registration(registration_request)) => {
                                info!("worker registration: {:?}", registration_request);
                                let worker_id = registration_request.worker_id.clone();
                                let mut channels = shared_state.grpc_worker_channels.lock().await;
                                channels.insert(worker_id.clone(), (tx.clone(), tx_cancel.clone()));
                                registered_worker_id = Some(worker_id.clone());
                                SchedulerMessage {
                                    scheduler_message_type: Some(
                                        scheduler_message::SchedulerMessageType::Ack(
                                            Ack {
                                                message: "worker connected".to_string(),
                                                status: WorkerChannelStatus::Ready.into()
                                            }
                                        ),
                                    ),
                                }
                            },
                            _ => {
                                let status = Status::aborted("Unknown worker message or missing data");
                                if tx.send(Err(status)).await.is_err() {
                                    error!("errored while sending worker error to scheduler");
                                }
                                SchedulerMessage::default()
                            }
                        };

                        // Process the worker message and create a response
                        if response.scheduler_message_type.is_some() {
                            if tx.send(Ok(response)).await.is_err() {
                                error!("{:?}", "errored");
                                // TODO
                                // Handle error sending response
                            }
                        }
                    }
                    Err(status) => {
                        error!("{:?}: {:?}", status, status.metadata());
                        if let Some(worker_id) = registered_worker_id.take() {  // Use the stored worker ID
                            let mut channels = shared_state.grpc_worker_channels.lock().await;
                            channels.remove(&worker_id);
                        }
                    }
                }
            }
            // This line isn't necessary in newer versions of tokio
            // tx.close_channel();
        });

        // Waiting for cancellation events for gRPC server
        // We spawning a new async thread to not block or communication thread
        // And we send the clients `Abort` status message
        tokio::spawn(async move  {
            rx_cancel.recv().await;
            let status = Status::aborted("Aborting channel");
            if tx_binding.send(Err(status)).await.is_err() {
                eprintln!("errored while sending worker error to scheduler");
            }
            println!("disconnecting client")
        });
        
        let response_stream: Self::CommunicateStream = Box::pin(ReceiverStream::new(rx)); // Use ReceiverStream
        Ok(Response::new(response_stream))
    }
}

async fn cancel_client(tx: Sender<Result<SchedulerMessage, Status>>, rx_cancel:&mut Receiver<()>) {
    rx_cancel.recv().await;
    let status = Status::aborted("Aborting channel");
    if tx.send(Err(status)).await.is_err() {
        error!("errored while sending worker error to scheduler");
    }
}

pub struct SharedData {
    worker_pool: Arc<Mutex<WorkerPool>>,
    pub worker_heartbeat: Arc<Mutex<HashMap<String, Instant>>>,
}

// Function to start the scheduler single process node gRPC server
// #[tonic::async_trait]
pub async fn start_scheduler_grpc_server(
    port: i32,
    pool: worker_pool::WorkerPool,
    graceful_timeout: u64,
    heartbeat_interval: std::time::Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "stats")]
    {
        // Spawn a new task for the metrics server
        tokio::spawn(async {
            if let Err(e) = metrics::start_metrics_server().await {
                // Handle the error as appropriate for your application
                eprintln!("Metrics server failed: {:?}", e);
            }
        });
    }

    // Channel for signaling gRPC server shutdown
    let (tx, mut rx) = mpsc::channel(1);

    // Init the worker pool
    let cloned_pool = pool.clone();
    let shared_data = Arc::new(SharedData {
        worker_pool: Arc::new(Mutex::new(pool)), // Pass the actual worker pool here
        worker_heartbeat: Arc::new(Mutex::new(HashMap::new())),
    });

    // gRPC server setup
    let addr = format!("0.0.0.0:{}", port).as_str().parse()?;

    // let binding_rx = rx.clone();
    // SchedulerWorkerService - for communication of workers to scheduler
    let grpc_state = GrpcSharedState::new(RoundRobinBalancer::new());
    let shared_grpc_state = Arc::new(grpc_state);
    let scheduler_worker_svc = SchedulerServer::new(shared_grpc_state.clone());
    let svc = SchedulerWorkerServiceServer::new(scheduler_worker_svc);

    // SchedulerService - admin service for communicating with scheduler by clients.
    let admin_service =
        SchedulerServiceServer::new(SchedulerAdminService::new(shared_data.clone(), Some(shared_grpc_state.clone())));

    // This AtomicBool will be used to track if the interrupt was previously received
    let interrupt_received = Arc::new(AtomicBool::new(false));
    let grpc_binding = shared_grpc_state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(heartbeat_interval).await;
    
            let mut worker_channels = grpc_binding.grpc_worker_channels.lock().await;
    
            for (worker_id, worker_channel) in worker_channels.iter_mut() {
                // Assuming send_heartbeat is an async function in your worker's gRPC API
                worker_channel.0.send(Ok(SchedulerMessage {
                    scheduler_message_type: Some(
                        scheduler_message::SchedulerMessageType::Heartbeat(())
                    )
                })).await;
    
                // match result {
                //     Ok(_) => {
                //         let mut heartbeats = shared_heartbeat.lock().await;
                //         heartbeats.insert(worker_id.clone(), Instant::now());
                //     }
                //     Err(err) => {
                //         eprintln!("Failed to receive heartbeat from worker {}: {:?}", worker_id, err);
                //     }
                // }
            }
        }
    });

    // Spawn a new task to listen for shutdown signals
    tokio::spawn(async move {
        let mut stream = signal(SignalKind::interrupt()).unwrap();
        loop {
            stream.recv().await;
            if interrupt_received.load(Ordering::Relaxed) {
                // Second Ctrl+C received, forcibly terminate
                log::debug!("second interrupt received, force quitting...");
                process::exit(1);
            } else {
                tokio::spawn(async move {
                    sleep(Duration::from_secs(graceful_timeout)).await;
                    log::debug!("Force shutdown after timeout: {}",graceful_timeout);
                        process::exit(1);
                });
                // First Ctrl+C received, initiate graceful shutdown
                log::debug!("server interrupted");

                interrupt_received.store(true, Ordering::Relaxed);

                let pool = cloned_pool.clone();
                let grpc = shared_grpc_state.clone();
                let wc = grpc.grpc_worker_channels.lock().await;
                for w in wc.values() {
                    if w.1.send(()).await.is_err() {
                        println!("Error while canceling worker channels from gRPC server")
                    };
                }
                // Explicitly dropping worker pool to kick off cleanup
                // Drop the pool in a new task
                // Note: we spawn here an `std` thread to overcome any blocking
                // to tokio runtime (for force shutdown)
                std::thread::spawn(move || {
                    // If WorkerPool started with shutdown flag then it is handled on drop impl
                    drop(pool);
                    
                });

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

    log::debug!("gRPC server started");
    server.await?;

    Ok(())
}

pub async fn start_single_process_grpc_server(
    port: i32,
    pool: worker_pool::WorkerPool,
    graceful_timeout: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "stats")]
    {
        // Spawn a new task for the metrics server
        tokio::spawn(async {
            if let Err(e) = metrics::start_metrics_server().await {
                // Handle the error as appropriate for your application
                eprintln!("Metrics server failed: {:?}", e);
            }
        });
    }

    // Channel for signaling gRPC server shutdown
    let (tx, mut rx) = mpsc::channel(1);

    // Init the worker pool
    let cloned_pool = pool.clone();
    let shared_data = Arc::new(SharedData {
        worker_pool: Arc::new(Mutex::new(pool)),
        worker_heartbeat: Arc::new(Mutex::new(HashMap::new())), // Pass the actual worker pool here
    });

    // gRPC server setup
    let addr = format!("0.0.0.0:{}", port).as_str().parse()?;

    // SchedulerService - admin service for communicating with scheduler by clients
    let admin_service =
        SchedulerServiceServer::new(SchedulerSingleProcessAdminService::new(shared_data.clone()));

    // This AtomicBool will be used to track if the interrupt was previously received
    let interrupt_received = Arc::new(AtomicBool::new(false));

    // Spawn a new task to listen for shutdown signals
    tokio::spawn(async move {
        let mut stream = signal(SignalKind::interrupt()).unwrap();
        loop {
            stream.recv().await;
            if interrupt_received.load(Ordering::Relaxed) {
                // Second Ctrl+C received, forcibly terminate
                log::debug!("second interrupt received, force quitting...");
                process::exit(1);
            } else {
                tokio::spawn(async move {
                    sleep(Duration::from_secs(graceful_timeout)).await;
                    log::debug!("Force shutdown after timeout: {}",graceful_timeout);
                        process::exit(1);
                });
                // First Ctrl+C received, initiate graceful shutdown
                log::debug!("server interrupted");

                interrupt_received.store(true, Ordering::Relaxed);

                let pool = cloned_pool.clone();
                // Explicitly dropping worker pool to kick off cleanup
                // Drop the pool in a new task
                // Note: we spawn here an `std` thread to overcome any blocking
                // to tokio runtime (for force shutdown)
                std::thread::spawn(move || {
                    // If WorkerPool started with shutdown flag then it is handled on drop impl
                    drop(pool);
                    
                });

                // Send the signal to shutdown the server
                tx.send(()).await.unwrap();
            }
        }
    });

    let server = Server::builder()
        .add_service(admin_service)
        .serve_with_shutdown(addr, async {
            // Wait for the signal to start the shutdown
            rx.recv().await;
        });

    log::debug!("gRPC server started");
    server.await?;

    Ok(())
}


pub struct SchedulerAdminService<B: LoadBalancer> {
    shared_data: Arc<SharedData>,
    shared_grpc_state: Option<Arc<GrpcSharedState<B>>>,
}

impl<B: LoadBalancer> SchedulerAdminService<B> {
    fn new(shared_data: Arc<SharedData>, shared_grpc_state: Option<Arc<GrpcSharedState<B>>> ) -> Self {
        Self { shared_data , shared_grpc_state }
    }
}

#[tonic::async_trait]
impl<B: LoadBalancer> SchedulerService for SchedulerAdminService<B> {

    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteResponse>, Status> {
        let shared_data = self.shared_data.as_ref().clone();
        let req = request.into_inner().clone();
        let task_name = req.task.as_ref().unwrap().id.clone();
        println!("task->{}", task_name);

        match &self.shared_grpc_state {
            Some(sd) => {
                info!("dispatching task to workers");
                let sm = SchedulerMessage {
                    scheduler_message_type: Some(
                        scheduler_message::SchedulerMessageType::AssignTask(
                            AssignTaskRequest { task: req.task }
                        )
                    )
                };
                sd.distribute_task(sm).await
            }
            None => {
                // Clone the shared data for the closure
                let cloned_shared = shared_data.worker_pool.as_ref().lock().await;
        
                // Capture the 'o' value from the lock before the async block
                let o_clone = { cloned_shared.executors.clone() };
        
                // Execute the logic using the cloned shared data and the cloned request
                match cloned_shared.execute(
                    move |args| {
                        let logic = o_clone.lock().unwrap();
                        log::debug!("executing task: {}", task_name,);
                        logic.get_executor(&task_name).unwrap().execute(args);
                    },
                    req,
                ) {
                    Ok(res) => Ok(Response::new(ExecuteResponse {
                        ..Default::default()
                    })),
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
        }

        
    }

    async fn schedule(
        &self,
        request: Request<ScheduleRequest>,
    ) -> Result<Response<ScheduleResponse>, Status> {
        let shared_data = self.shared_data.clone();
        let mut err_details = ErrorDetails::new();
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


pub struct SchedulerSingleProcessAdminService {
    shared_data: Arc<SharedData>,
}

impl SchedulerSingleProcessAdminService {
    fn new(shared_data: Arc<SharedData>) -> Self {
        Self { shared_data  }
    }
}

#[tonic::async_trait]
impl SchedulerService for SchedulerSingleProcessAdminService {
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
        let o_clone = { cloned_shared.executors.clone() };

        // Execute the logic using the cloned shared data and the cloned request
        match cloned_shared.execute(
            move |args| {
                let logic = o_clone.lock().unwrap();
                log::debug!("executing task: {}", task_name,);
                logic.get_executor(&task_name).unwrap().execute(args);
            },
            req,
        ) {
            Ok(res) => Ok(Response::new(ExecuteResponse {
                ..Default::default()
            })),
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
        let mut err_details = ErrorDetails::new();
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