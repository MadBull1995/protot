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

use std::{error::Error, sync::Arc, time::Duration};

use async_trait::async_trait;
use protot::{
    // Useful core traits and struct
    core::worker_pool::{TaskExecutor, TaskRegistry, GrpcWorkersRegistry, AsyncTaskExecutor},
    // Protobuf Impl for communication and other common objects
    internal::protot::{
        core::{Config, NodeType, TaskState, LoadBalancer},
        scheduler::v1::{ExecuteRequest, TaskCompletion},
    },
    // Easy startup procedure
    start,
};
use tokio::time::sleep;
use tonic::{Status, Response};

// Define you own task executor
struct MyWorker;
// Impl the TaskExecutor trait to hold the actual task logic
impl TaskExecutor for MyWorker {
    fn execute(&self, args: ExecuteRequest) {
        println!("custom worker executed task");
    }
}

#[tokio::main]
pub async fn main() {
    // ** Scheduler Server Setup **
    // Init a registry for your tasks and Executors
    let mut tr = TaskRegistry::new();
    // Register you custom executor
    tr.register_task("some_task_name", MyWorker {});

    // Optional configuration otherwise must have a 'configs.yaml/json' file in your root directory
    let cfg = Config {
        grpc_port: 44880,
        node_type: NodeType::SingleProcess.into(),
        num_workers: 4,
        graceful_timeout: 30,
        heartbeat_interval: None,
        load_balancer: LoadBalancer::RoundRobin.into(),
        data_store: None
    };

    // Startup the scheduler service and workers
    match start(tr, Some(cfg)).await {
        Err(err) => panic!("some error occured: {:?}", err),
        Ok(()) => println!("protot server exited."),
    };
}
