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

use std::{error::Error, sync::Arc, time::Duration, env};

use async_trait::async_trait;
use protot::{
    // Useful core traits and struct
    core::worker_pool::{TaskExecutor, TaskRegistry, GrpcWorkersRegistry, AsyncTaskExecutor},
    // Protobuf Impl for communication and other common objects
    internal::protot::{
        core::{Config, NodeType, TaskState},
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

/// Can add a custom data structure to our worker
struct MyGrpcWorker;

/// gRPC worker must implement `AsyncTaskExecutor` trait for handling async calls from the scheduler server 
#[async_trait]
impl AsyncTaskExecutor for MyGrpcWorker {
    /// the execute will be invoked on worker once it recieved `AssignTaskRequest` from scheduler
    async fn execute(&self, args: ExecuteRequest) -> Result<TaskCompletion, ()> {
        println!("gRPC worker executing async task {:?}", args);

        // ..Your code execution logic goes here..

        // Just for mimicing "Really long running" task
        let join_handle = tokio::spawn(async {
            tokio::time::sleep(Duration::from_secs(10)).await;
        });
        
        let _ = join_handle.await.map_err(|e| {
            eprintln!("Failed to join spawned task: {}", e);
        });

        Ok(
            TaskCompletion {
                task_id: args.task.unwrap().id.clone(),
                state: TaskState::Pending.into(),
                execution_id: args.execution_id 
            }
        )
    }
}

#[tokio::main]
async fn main() {

    // Parse command-line arguments to get worker ID
    let args: Vec<String> = env::args().collect();
    let worker_id = if args.len() > 1 {
        args[1].clone()
    } else {
        "default-worker-id".to_string() // fallback to a default worker ID
    };

    // ** gRPC Worker Setup **
    let mut grpc_registry = GrpcWorkersRegistry::new();

    // Register tasks execution logic
    grpc_registry.register_task("task-1", MyGrpcWorker {});
    grpc_registry.register_task("task-2", MyGrpcWorker {});
 
    // ..any task that is not registered on the worker will bot be executed on this worker..

    // worker id must be unique on the scheduler
    // otherwise it will return error on communicate
    let worker = protot::client::GrpcWorkerBuilder::new()
        .with_id(worker_id) 
        .with_registry(grpc_registry)
        .build();
    // the only communication channel process for scheduler<->worker
    let res = worker.communicate().await;
    
    match res {
        Ok(()) => println!("worker done"),
        Err(e) => println!("some error: {:?}", e)
    };
}
