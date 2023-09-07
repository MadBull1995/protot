
use std::{
    sync::Arc,
    error::Error,
};
use tokio::{
    sync::{mpsc, Mutex}, time::sleep,
};
use std::time::Duration;
use tonic::{Request, Response, Status, transport::Channel };
use crate::{internal::sylklabs::{
    core::{Task, TaskState},
    scheduler::v1::{
        ExecuteRequest,
        ExecuteResponse,
        scheduler_worker_service_client::SchedulerWorkerServiceClient,
        scheduler_service_client::SchedulerServiceClient, WorkerMessage, RegistrationRequest, worker_message, TaskCompletion, scheduler_message
    }
}, core::worker_pool::AsyncTaskExecutor,
};

#[macro_export]
macro_rules! execute_task {
    ($client:expr, $name:expr) => {
        async {
            async fn process_client_call(c: &mut SchedulerServiceClient<Channel>) -> Result<Response<ExecuteResponse>, Status> {
                let request = ExecuteRequest::default();
                c.execute(request).await
            }
            println!("{}", $name.to_string());
            let t = Task::default();
            let request = tonic::Request::new(ExecuteRequest {
                task: Some(t),
            });
            match process_client_call($client).await {
                Ok(response) => {
                    let reply = response.into_inner();
                    println!("Server responded with message: {:?}", reply);
                }
                Err(e) => println!("Failed to call SayHello: {}", e),
            }
        }
    };
}

struct GrpcExecutor {
    workers: Arc<Mutex<Vec<SchedulerServiceClient<Channel>>>>
}

#[tonic::async_trait(?Send)]
impl AsyncTaskExecutor for GrpcExecutor {
    async fn execute(&self, args: ExecuteRequest) {
        let mut w = self.workers.lock().await;
        match w.get_mut(0) {
            None => { println!("client worker not found")}
            Some(c) => execute_task!(c, "some-task").await
        }
    }
}

pub struct GrpcWorker {
    registeration_details: RegistrationRequest
}

impl Default for GrpcWorker {
    fn default() -> Self {
        GrpcWorkerBuilder::new()
            .build()
        // Self::new()
    }
}

struct GrpcWorkerBuilder {
    worker_id: Option<String>,
    tasks: Vec<String>,
    cookie: Option<String>,
}

impl GrpcWorkerBuilder {
    pub fn new() -> Self {
        GrpcWorkerBuilder { 
            worker_id: Some("SomeWorkerId".to_string()),
            tasks: Vec::new(),
            cookie: None
        }
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.worker_id = Some(id);
        self
    }

    pub fn add_task(mut self, task_name: String) -> Self {
        self.tasks.push(task_name);
        self
    }

    pub fn build(self) -> GrpcWorker {
        GrpcWorker {
            registeration_details: RegistrationRequest {
                worker_id: self.worker_id
                    .clone()
                    .unwrap_or("SomeWorkerId".to_string()),
                supported_tasks: self.tasks,
                magic_cookie: self.cookie
                    .clone()
                    .unwrap_or("SomeSecert".to_string())
            }
        }
    }
}

impl GrpcWorker {
    
    pub async fn communicate(&self) -> Result<(), Box<dyn Error>>  {
        let mut client = SchedulerWorkerServiceClient::connect("http://0.0.0.0:44880").await?;
        let (tx,mut rx) = mpsc::channel::<TaskCompletion>(32);
        let binding = self.registeration_details.clone();
        
        let outbound = async_stream::stream! {
            let register = WorkerMessage {
                worker_message_type: Some(
                    worker_message::WorkerMessageType::Registration(
                        binding
                    )
                )
            };
            yield register;
            
            loop {
                let t = rx.recv().await;
                if !t.is_none() {
                    let response = WorkerMessage {
                        worker_message_type: Some(
                            worker_message::WorkerMessageType::Completion(
                                t.unwrap()
                            )
                        )
                    };
                    yield response;
                } else {
                    println!("some error from server");
                    break;
                }
            }
        };
    
        let response = client.communicate(Request::new(outbound)).await?;
        let mut inbound = response.into_inner();
        let binding = tx.clone();
        while let Some(scheduler_msg) = inbound.message().await? {
            match scheduler_msg.scheduler_message_type {
                Some(msg) => {
                    match msg {
                        scheduler_message::SchedulerMessageType::AssignTask(t) => {
                            let bind = t.task.clone();
                            println!("executing task: {:?}", bind.unwrap().id);
                            sleep(Duration::from_secs(2)).await;
                            let task_complete = TaskCompletion {
                                task_id: t.task.unwrap().id,
                                state: TaskState::Success.into()
                            };
                            binding.send(task_complete).await?;
                        },
                        scheduler_message::SchedulerMessageType::Disconnect(diconnect) => {
                            println!("disconnecting worker: {:?}", diconnect);
                        }
                        scheduler_message::SchedulerMessageType::Ack(ack) => {
                            println!("worker registerd on scheduler server: {:?}", ack);
                        }
                        _ => println!("unable to communicate with unknown scheduler message")
                    }
                }
                None => println!("invalid scheduler message"),
            }
           
            ()
        }
        Ok(())
    }
}