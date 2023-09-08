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
}

#[async_trait]
trait DropClient {
    async fn drop(&mut self);
}

impl<B: LoadBalancer> GrpcSharedState<B> {
    pub async fn distribute_task(&self, task: SchedulerMessage) -> Result<Response<ExecuteResponse>, Status> {
        let worker_channels = self.grpc_worker_channels.lock().await;

        let mut balancer = self.balancer.lock().await;
        if let Some(key) = balancer.select_worker(&worker_channels).await {
            let (sender, _) = worker_channels.get(&key).unwrap();
            if sender.send(Ok(task)).await.is_err() {
                error!("Error while dispatching task");
            };
            return Ok(Response::new(ExecuteResponse::default()))
        } else {
            return Err(Status::aborted("No available workers"));
        }
    }
}

impl<B: LoadBalancer> GrpcSharedState<B> {
    pub fn new(balancer: B) -> Self {
        Self {
            grpc_worker_channels: Mutex::new(HashMap::new()),
            balancer: Mutex::new(balancer),
            worker_heartbeat: Arc::new(Mutex::new(HashMap::new())),
        }
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
}

#[async_trait]
impl<B: LoadBalancer> DropClient for GrpcSharedState<B> {
    async fn drop(&mut self) {
        println!("dropping clients");
        let binding = self.grpc_worker_channels.lock().await;
        drop(binding)
    }
}