#[macro_use]
extern crate lazy_static;

use protot::{
    core::worker_pool::TaskRegistry, start, SchedulerError, TaskExecutorImpl1, TaskExecutorImpl2,
};
// use protot::internal::sylklabs::scheduler::v1::ExecuteRequest;
#[tokio::main]
async fn main() -> Result<(), SchedulerError> {
    let mut executors = TaskRegistry::new();

    executors.register_task("task-1", TaskExecutorImpl1 {});
    executors.register_task("task-2", TaskExecutorImpl2 {});

    start(executors, None).await?;

    // let mut client = SchedulerServiceClient::connect("http://[::1]:50051").await.unwrap();

    // let future = execute_task!(&mut client, "some-task-1").await;
    // println!("{:?}", future);

    Ok(())
    // let scheduler = Scheduler::new(pool);
}
