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

use crate::{internal::protot::{scheduler::v1::{SchedulerMessage, scheduler_message, AssignTaskRequest, ExecuteResponse}, core::TaskState}, utils::shared::GrpcWorkerChannels};

use super::{worker_pool::AsyncTaskExecutor, load_balancer::LoadBalancer};
use std::{collections::HashMap, error::Error, sync::Arc, time::Instant};
use async_trait::async_trait;
use log::error;
use tokio::sync::{
    mpsc,
    Mutex
};
use tonic::{Status, Response};

pub struct GrpcSharedState<B: LoadBalancer> {
    pub grpc_worker_channels: Mutex<GrpcWorkerChannels>,
    balancer: Mutex<B>,
    pub worker_heartbeat: Arc<Mutex<HashMap<String, Instant>>>,
    max_task_queue: usize,
}

#[async_trait]
trait DropClient {
    async fn drop(&mut self);
}

impl<B: LoadBalancer> GrpcSharedState<B> {

    pub fn new(balancer: B, max_task_queue: Option<usize>) -> Self {
        let max_queue_size = match max_task_queue {
            None => {100},
            Some(queue_size) => {queue_size}
        };
        Self {
            grpc_worker_channels: Mutex::new(HashMap::new()),
            balancer: Mutex::new(balancer),
            worker_heartbeat: Arc::new(Mutex::new(HashMap::new())),
            max_task_queue: max_queue_size,
        }
    }

    pub fn max_queue_size(&self) -> usize {
        self.max_task_queue
    }

    pub async fn drop_workers(self) {
        println!("dropping clients");
        let binding = self.grpc_worker_channels.lock().await;
        for (w, c) in binding.iter() {
            if c.1.send(()).await.is_err() {
                println!("error while closing worker connection: {}", w);
            };
        }
    }

    pub async fn distribute_task(&self, task: SchedulerMessage) -> Result<Response<ExecuteResponse>, Status> {
        let worker_channels = self.grpc_worker_channels.lock().await;

        let mut balancer = self.balancer.lock().await;
        if let Some(key) = balancer.select_worker(&worker_channels).await {
            let (sender, _) = worker_channels.get(&key).unwrap();
            let t = task.clone();
            if sender.send(Ok(task)).await.is_err() {
                error!("Error while dispatching task");
            };
            let (task_id, execution_id) = match t.scheduler_message_type {
                Some(scheduler_message::SchedulerMessageType::AssignTask(task)) => {
                    (task.task.unwrap().id, task.execution_id)
                }
                _ => { ("UnknownTaskId".to_string(), "UnknownExecutionId".to_string()) }
            };
            return Ok(Response::new(ExecuteResponse{
                execution_id,
                task_id,
                state: TaskState::Pending.into(),
            }))
        } else {
            return Err(Status::aborted("No available workers"));
        }
    }
}

#[async_trait]
impl<B: LoadBalancer> DropClient for GrpcSharedState<B> {
    async fn drop(&mut self) {
        println!("dropping clients");
        let binding = self.grpc_worker_channels.lock().await;
        drop(binding)
    }
}