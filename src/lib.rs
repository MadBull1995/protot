//! # Sylklabs Scheduler
//!
//! `main.rs` is the entry point for the Sylklabs Scheduler, a distributed task scheduling application.
//! This file initializes the scheduler, manages configuration loading, and starts the execution of tasks.
//!
//! ## Usage
//!
//! Run the scheduler by executing the binary. You can provide a configuration file as a command-line argument
//! to customize the behavior of the scheduler.
//!
//! ```sh
//! $ ./scheduler --config=config.yaml
//! ```
//!
//! ## Features
//!
//! - Distributed Task Scheduling: Efficiently distributes tasks across worker nodes.
//! - Configuration Loading: Loads configuration from JSON or YAML files for easy customization.
//! - Scalability: Scales to accommodate varying workloads and available resources.
//!
//! ## Configuration
//!
//! The scheduler's behavior can be configured using a JSON or YAML configuration file. Refer to the documentation
//! for the expected format and available options.
//!
//! ## Examples
//!
//! ```rust,no_run
//! use proto_tasker::config_load;
//! // Initializing and running the scheduler.
//! fn main() {
//!     // Load configuration from command-line arguments or defaults.
//!     let config = config_load(String::from("my_config.yaml"));
//!
//!     // Initialize the scheduler using the provided configuration.
//!
//!     // Start scheduling tasks.
//! }
//! ```
//!
//! ## Dependencies
//!
//! The following external crates are used by the Sylklabs Scheduler:
//!
//! - `sylklabs`: Core library for scheduler logic and data structures.
//! - `serde`: Serialization and deserialization.
//! - `clap`: Command-line argument parsing.
//!
//! ## License
//!
//! This project is licensed under the terms of the Apache License, Version 2.0.
//!
//! ---
//! © 2023 Sylklabs Technologies

mod client;
pub mod core;
pub mod internal;
mod server;
mod utils;
use std::thread;

use internal::sylklabs;
pub use utils::{configs::config_load, error::SchedulerError, logger};

use crate::server::start_grpc_server;

pub fn start() -> Result<(), SchedulerError> {
    logger::init(true);
    logger::log(logger::LogLevel::INFO, "Scheduler starting");

    // Loading configurations to `sylklabs.core.Config` message from yaml/json/toml
    let cfgs = match config_load("configs.yaml".to_string()) {
        Ok(cfg) => {
            logger::log(
                logger::LogLevel::DEBUG,
                format!("Loaded configurations {:#?}", cfg).as_str(),
            );
            cfg
        }
        Err(e) => panic!("errored: {:?}", e),
    };

    match cfgs.node_type() {
        sylklabs::core::NodeType::SingleProcess => init_single_process_scheduler(cfgs),
        // sylklabs::core::NodeType::Scheduler => {
        //     init_scheduler_server(cfgs)
        // },
        _ => Err(SchedulerError::SchedulerUnimplemented(format!(
            "Unimplemented node type {}",
            sylklabs::core::NodeType::from_i32(cfgs.node_type)
                .unwrap()
                .as_str_name()
        ))),
    }?;

    Ok(())
}

fn init_scheduler_server() -> Result<(), SchedulerError> {
    Ok(())
}

fn init_single_process_scheduler(cfg: sylklabs::core::Config) -> Result<(), SchedulerError> {
    let pool = core::worker_pool::Builder::new()
        .num_threads(cfg.num_workers as usize)
        .thread_name("scheduler".to_string())
        .thread_stack_size(32 * 1024 * 1024)
        .build()?;
    pool.execute(|| println!("started worker pool"));
    // Todo start scheduler server
    match start_grpc_server(cfg.grpc_port, pool) {
        Err(err) => Err(SchedulerError::SchedulerServiceError(format!(
            "Scheduler errored: {:?}",
            &*err
        )))?,
        Ok(_) => println!("server shutdown"),
    };

    // println!("{:?}",pool);

    Ok(())
}
