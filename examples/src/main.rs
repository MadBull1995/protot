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
        core::{Config, NodeType, TaskState},
        scheduler::v1::{ExecuteRequest, TaskCompletion},
    },
    // Easy startup procedure
    start,
};
use tokio::time::sleep;
use tonic::{Status, Response};

// Define you own task executor
struct MyExecutor;
// Impl the TaskExecutor trait to hold the actual task logic
impl TaskExecutor for MyExecutor {
    fn execute(&self, args: ExecuteRequest) {
        println!("executed task");
    }
}
struct MyAsyncExecutor;

#[async_trait]
impl AsyncTaskExecutor for MyAsyncExecutor {
    async fn execute(&self, args: ExecuteRequest) -> Result<TaskCompletion, ()> {
        
        println!("executed async task {:?}", args);
        // tokio::spawn returns a JoinHandle that is a Future.
        // The spawned future is running in the background and you can await the JoinHandle whenever you're ready.
        let join_handle = tokio::spawn(async {
            tokio::time::sleep(Duration::from_secs(10)).await;
        });
         // Awaiting the join handle here to make sure the spawned task completes.
        // If you wish, you can store it somewhere and await it later.
        let _ = join_handle.await.map_err(|e| {
            eprintln!("Failed to join spawned task: {}", e);
        });
        Ok(TaskCompletion { task_id: args.task.unwrap().id.clone(), state: TaskState::Pending.into(), execution_id: args.execution_id })
    }
}


#[tokio::main]
async fn main() {


    // ** gRPC Worker Setup **
    let mut grpc_registry = GrpcWorkersRegistry::new();
    grpc_registry.register_task("task-1", MyAsyncExecutor {});
    grpc_registry.register_task("task-2", MyAsyncExecutor {});
 
    let worker = protot::client::GrpcWorkerBuilder::new()
        .with_id("some-worker-2".to_string())
        .with_registry(grpc_registry)
        .build();
    
    let res = worker.communicate().await;
    match res {
        Ok(()) => println!("worker done"),
        Err(e) => println!("some error: {:?}", e)
    };


    // handler_worker_1.join().unwrap().await;

    // ** Scheduler Server Setup **

    // Init a registry for your tasks and Executors
    // let mut tr = TaskRegistry::new();
    // tr.register_task(task_name, executor)
    // // Register you custom executor
    // tr.register_task("some_task_name", MyExecutor {});

    // // Optional configuration otherwise must have a 'configs.yaml/json' file in your root directory
    // let cfg = Config {
    //     grpc_port: 44880,
    //     node_type: NodeType::SingleProcess.into(),
    //     num_workers: 4,
    // };

    // // Startup the scheduler service and workers
    // match start(tr, Some(cfg)) {
    //     Err(err) => panic!("some error occured: {:?}", err),
    //     Ok(()) => println!("protot server started."),
    // };
}
