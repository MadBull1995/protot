use crate::{internal::protot::{scheduler::v1::{SchedulerMessage, scheduler_message, AssignTaskRequest, ExecuteResponse}, core::TaskState}, utils::shared::GrpcWorkerChannels};

use super::{worker_pool::AsyncTaskExecutor, load_balancer::LoadBalancer};
use std::{collections::HashMap, error::Error};
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