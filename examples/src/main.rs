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
use tonic::Status;

// Define you own task executor
struct MyExecutor;
// Impl the TaskExecutor trait to hold the actual task logic
impl TaskExecutor for MyExecutor {
    fn execute(&self, args: ExecuteRequest) {
        println!("executed task");
    }
}

struct MyAsyncExecutor;

#[async_trait(?Send)]
impl AsyncTaskExecutor for MyAsyncExecutor {
    async fn execute(&self, args: ExecuteRequest) -> Result<TaskCompletion, Box<dyn Error>> {
        println!("executed async task {:?}", args);
        
        tokio::spawn(sleep(Duration::from_secs(10))).await?;

        Ok(TaskCompletion { task_id: args.task.unwrap().id.clone(), state: TaskState::Pending.into() })
    }
}


#[tokio::main]
async fn main() {


    // ** gRPC Worker Setup **
    let mut grpc_registry = GrpcWorkersRegistry::new();
    grpc_registry.register_task("task-1", MyAsyncExecutor {});
    grpc_registry.register_task("task-2", MyAsyncExecutor {});
 
    let worker = protot::client::GrpcWorkerBuilder::new()
        .with_id("some-worker-3".to_string())
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
