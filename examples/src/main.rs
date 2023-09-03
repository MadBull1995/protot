use protot::{
    // Useful core traits and struct
    core::worker_pool::{TaskExecutor, TaskRegistry},
    // Protobuf Impl for communication and other common objects
    internal::sylklabs::{
        core::{Config, NodeType},
        scheduler::v1::ExecuteRequest,
    },
    // Easy startup procedure
    start,
};

// Define you own task executor
struct MyExecutor;
// Impl the TaskExecutor trait to hold the actual task logic
impl TaskExecutor for MyExecutor {
    fn execute(&self, args: ExecuteRequest) {
        println!("executed task");
    }
}

fn main() {
    // Init a registry for your tasks and Executors
    let mut tr = TaskRegistry::new();

    // Register you custom executor
    tr.register_task("some_task_name", MyExecutor {});

    // Optional configuration otherwise must have a 'configs.yaml/json' file in your root directory
    let cfg = Config {
        grpc_port: 44880,
        node_type: NodeType::SingleProcess.into(),
        num_workers: 4,
    };

    // Startup the scheduler service and workers
    match start(tr, Some(cfg)) {
        Err(err) => panic!("some error occured: {:?}", err),
        Ok(()) => println!("protot server started."),
    };
}
