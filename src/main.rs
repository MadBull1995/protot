#[macro_use]
extern crate lazy_static;

use proto_tasker::{start, SchedulerError};

fn main() -> Result<(), SchedulerError> {
    start()?;

    Ok(())
    // let scheduler = Scheduler::new(pool);
}
