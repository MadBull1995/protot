#[macro_use]
extern crate lazy_static;

use proto_tasker::{start, SchedulerError, core::worker_pool::TaskRegistry, TaskExecutorImpl1, TaskExecutorImpl2};

fn main() -> Result<(), SchedulerError> {
    
    let mut executors = TaskRegistry::new();
    
    executors.register_task("task-1", TaskExecutorImpl1 {});
    executors.register_task("task-2", TaskExecutorImpl2 {});


    start(executors, None)?;

    Ok(())
    // let scheduler = Scheduler::new(pool);
}
