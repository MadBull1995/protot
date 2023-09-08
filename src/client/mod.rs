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
    sync::Arc,
    error::Error,
};
use async_trait::async_trait;
use log::debug;
use tokio::{
    sync::{mpsc, Mutex}, time::sleep,
};
use tokio_stream::StreamExt;
use std::time::Duration;
use tonic::{Request, Response, Status, transport::Channel };
use crate::{internal::protot::{
    core::{Task, TaskState},
    scheduler::v1::{
        ExecuteRequest,
        ExecuteResponse,
        scheduler_worker_service_client::SchedulerWorkerServiceClient,
        scheduler_service_client::SchedulerServiceClient, WorkerMessage, RegistrationRequest, worker_message, TaskCompletion, scheduler_message, Pong
    }, metrics::v1::WorkerMetrics
}, core::worker_pool::{AsyncTaskExecutor, GrpcWorkersRegistry},
};

// #[macro_export]
// macro_rules! execute_task {
//     ($client:expr, $name:expr) => {
//         async {
//             async fn process_client_call(c: &mut SchedulerServiceClient<Channel>) -> Result<Response<ExecuteResponse>, Status> {
//                 let request = ExecuteRequest::default();
//                 c.execute(request).await
//             }
//             println!("{}", $name.to_string());
//             let t = Task::default();
//             let request = tonic::Request::new(ExecuteRequest {
//                 task: Some(t),
//             });
//             process_client_call($client).await?
//         }
//     };
// }

// struct GrpcExecutor {
//     workers: Arc<Mutex<Vec<SchedulerServiceClient<Channel>>>>
// }

// #[async_trait(?Send)]
// impl AsyncTaskExecutor for GrpcExecutor {
//     async fn execute(&self, args: ExecuteRequest) -> Result<TaskCompletion, Status> {
//         let mut w = self.workers.lock().await;
//         match w.get_mut(0) {
//             None => { Err(Status::aborted("client worker not found"))? }
//             Some(c) => execute_task!(c, "some-task").await
//         }
//     }
// }

pub struct GrpcWorker {
    registeration_details: RegistrationRequest,
    registry: GrpcWorkersRegistry,
}

impl Default for GrpcWorker {
    fn default() -> Self {
        GrpcWorkerBuilder::new()
            .build()
        // Self::new()
    }
}

pub struct GrpcWorkerBuilder {
    worker_id: Option<String>,
    tasks: Vec<String>,
    cookie: Option<String>,
    registry: Option<GrpcWorkersRegistry>,
}

impl GrpcWorkerBuilder {
    pub fn new() -> Self {
        GrpcWorkerBuilder { 
            worker_id: Some("SomeWorkerId".to_string()),
            tasks: Vec::new(),
            cookie: None,
            registry: None,
        }
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.worker_id = Some(id);
        self
    }

    pub fn with_registry(mut self, registry: GrpcWorkersRegistry) -> Self {
        self.registry = Some(registry);
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
            },
            registry: self.registry.unwrap_or(GrpcWorkersRegistry::new()),
        }
    }
}

impl GrpcWorker {
    
    pub async fn communicate(&self) -> Result<(), Box<dyn Error>>  {
        let mut client = SchedulerWorkerServiceClient::connect("http://0.0.0.0:44880").await?;
        let (tx,mut rx) = mpsc::channel::<WorkerMessage>(32);
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
                    yield t.unwrap();
                } else {
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
                            // println!("executing task: {:?}", bind.unwrap().id);
                            let executor = self.registry.get_executor(&bind.unwrap().id);
                            match executor {
                                Ok(operation) => {

                                    match operation.execute(ExecuteRequest { task: t.task.clone() }).await {
                                        Ok(completion) => {
                                            // Sending task response to the communicate loop
                                            let response = WorkerMessage {
                                                worker_message_type: Some(
                                                    worker_message::WorkerMessageType::Completion(
                                                        completion.clone()
                                                    )
                                                )
                                            };
                                            binding.send(response).await?;
                                        }
                                        Err(err) => {
                                            panic!("failed to execute task: {:?}", err);
                                        }
                                    }
                                    
                                },
                                Err(err) => Err(err)?
                            }
                        },
                        scheduler_message::SchedulerMessageType::Disconnect(disconnect) => {
                            println!("disconnecting worker: {:?}", disconnect);
                        }
                        scheduler_message::SchedulerMessageType::Ack(ack) => {
                            println!("worker registerd on scheduler server: {:?}", ack);
                        }
                        scheduler_message::SchedulerMessageType::Heartbeat(_) => {
                            debug!("got heartbeat from scheduler");
                            binding.send(WorkerMessage { worker_message_type: Some(
                                worker_message::WorkerMessageType::Heartbeat(
                                    Pong {
                                        metrics: Some(
                                            WorkerMetrics {
                                                ..Default::default()
                                            }
                                        )
                                    }
                                )
                            ) }).await?;
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