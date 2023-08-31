#[macro_use]
extern crate lazy_static;
use std::{thread::sleep, time::Duration};

use proto_tasker::{config_load, core, logger, start, SchedulerError};

fn main() -> Result<(), SchedulerError> {
    start()?;

    Ok(())
    // let scheduler = Scheduler::new(pool);
}
