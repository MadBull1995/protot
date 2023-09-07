//! # ProtoT - Task Scheduler: A Distributed Task Scheduling System
//!
//! `ProtoT` is a sophisticated distributed task scheduling library built with Rust, designed to manage and distribute tasks effectively across various worker nodes.
//!
//! This crate uses Protocol Buffers for efficient serialization and is built to be both thread-safe and highly extensible.
//!
//! ## Features
//!
//! - **Distributed Task Scheduling**: Utilizes a balanced and efficient algorithm to distribute tasks across worker nodes.
//! - **Thread Safety**: Ensures safe concurrent execution of tasks across multiple threads.
//! - **Protocol Buffers**: Uses Protocol Buffers for efficient serialization and message passing.
//! - **Scalability**: Built with scalability in mind, allowing for easy addition of more nodes or tasks.
//! - **Configurable**: Allows users to specify custom configurations through JSON or YAML files.
//!
//! ## Architecture
//!
//! The architecture revolves around the concept of a `WorkerPool`, which is responsible for managing worker threads known as `Workers`. These workers are thread-safe, atomic units that can execute tasks.
//!
//! Shared state and metrics between workers are maintained by `WorkerPoolSharedData`. Task execution capabilities are abstracted by the `TaskExecutor` trait and organized within a `TaskRegistry`.
//!
//! For health monitoring, a `Sentinel` struct is responsible for overseeing the entire system.
//!
//! ## Installation and Usage
//!
//! You can include it in your project by adding `protot` to your `Cargo.toml` dependencies.
//!
//! ```toml
//! [dependencies]
//! protot = "0.1.0"
//! ```
//!
//! Detailed installation and usage instructions are available in the `README.md` file.
//!
//! ## Examples
//!
//! Here's a quick example that demonstrates basic usage:
//!
//! ```rust
//!
//! fn main() {
//!     let config = config_load(String::from("my_config.yaml"));
//!     // Further code to initialize and run the scheduler
//! }
//! ```
//!
//! For more examples and configuration options, please refer to the `examples/` directory in the repository.
//!
//! ## License
//!
//! This project is licensed under the terms of the Apache License, Version 2.0.
//!
//! ## Contributions
//!
//! Contributions are very welcome! Please read our [contributing guidelines](CONTRIBUTING.md) for details on the process for submitting pull requests to us.
//!
//! ## Contact Information
//!
//! For any questions or clarifications, feel free to reach out at `contact@sylk.build`.
//!
//! ---
//!
//! This crate and its documentation are brought to you by [sylk.build](https://www.sylk.build), © 2023 Sylk Technologies.
//!

pub mod client;
pub mod core;
pub mod internal;
mod server;
mod utils;
use crate::{core::worker_pool::{TaskExecutor, TaskRegistry}, server::start_single_process_grpc_server};
pub use lazy_static::lazy_static;
use log::{info, debug};
use server::start_scheduler_grpc_server;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use internal::protot::{
    self,
    core::{Config, Task},
    scheduler::v1::ExecuteRequest,
};
use protobuf::well_known_types::{any::Any, struct_};
pub use utils::{configs::config_load, error::SchedulerError, logger};

pub async fn start(
    registry: TaskRegistry,
    configurations: Option<Config>
) -> Result<(), SchedulerError> {

    let _ = logger::init();
    info!("Scheduler starting");

    // Loading configurations to `sylklabs.core.Config` message from yaml/json/toml
    let cfgs = match configurations {
        Some(cfgs) => cfgs,
        None => match config_load("configs.yaml".to_string()) {
            Ok(cfg) => {
                debug!("Loaded configurations: {:?}", cfg,);
                cfg
            }
            Err(e) => panic!("errored: {:?}", e),
        },
    };

    let opts = ProcessOptions { 
        task_executors: registry,
        ..Default::default()
    };

    match cfgs.node_type() {
        protot::core::NodeType::SingleProcess => init_single_process_grpc_scheduler(cfgs, opts).await,
        protot::core::NodeType::Scheduler => {
            init_distributed_grpc_scheduler(cfgs, opts).await
        },
        _ => Err(SchedulerError::SchedulerUnimplemented(format!(
            "Unimplemented node type {}",
            protot::core::NodeType::from_i32(cfgs.node_type)
                .unwrap()
                .as_str_name()
        ))),
    }?;

    Ok(())
}


async fn init_distributed_grpc_scheduler( 
    cfg: protot::core::Config,
    opts: ProcessOptions,
) -> Result<(), SchedulerError> {
    let registry = opts.task_executors;

    let executors = Arc::new(Mutex::new(registry));

    let pool = core::worker_pool::Builder::new()
        .num_workers(cfg.num_workers as usize)
        .name(opts.process_name)
        .thread_stack_size(32 * 1024 * 1024)
        .executors(executors)
        .build()?;

    collect_stats();
    // Todo start scheduler server
    match start_scheduler_grpc_server(cfg.grpc_port, pool, cfg.graceful_timeout).await {
        Err(err) => Err(SchedulerError::SchedulerServiceError(format!(
            "Scheduler errored: {:?}",
            &*err
        )))?,
        Ok(_) => println!("Goodbye :)"),
    };

    Ok(())
}
struct ProcessOptions {
    process_name: String,
    task_executors: TaskRegistry,
}

impl Default for ProcessOptions {
    fn default() -> Self {
        Self {
            process_name: "scheduler".to_string(),
            task_executors: TaskRegistry::new(),
        }
    }
}

pub struct TaskExecutorImpl1 {}

impl TaskExecutor for TaskExecutorImpl1 {
    fn execute(&self, args: ExecuteRequest) {
        let task: Task = args.clone().task.unwrap().clone();
        let payload = task.payload.unwrap();
        let task_id = args.task.unwrap().id;
        let mut data = Any::new();
        data.type_url = payload.type_url;
        data.value = payload.value;

        let data = Any::unpack::<struct_::Struct>(&data);
        println!("task excution 1 with dynamic args! {task_id}");
        thread::sleep(Duration::from_secs(5))
    }
}

pub struct TaskExecutorImpl2 {}

impl TaskExecutor for TaskExecutorImpl2 {
    fn execute(&self, args: ExecuteRequest) {
        let task: Task = args.clone().task.unwrap().clone();
        let payload = task.payload.unwrap();
        let task_id = args.task.unwrap().id;
        let mut data = Any::new();
        data.type_url = payload.type_url;
        data.value = payload.value;

        let data = Any::unpack::<struct_::Struct>(&data);
        thread::sleep(Duration::from_secs(1));
        println!("task excution 2 with dynamic args! {}", task_id);
    }
}

#[cfg(feature = "stats")]
fn collect_stats() {
    log::debug!("collecting stats enabled");
}

#[cfg(not(feature = "stats"))]
fn collect_stats() {
    log::debug!("collecting stats disabled, to enable use feature flag: \"stats\"");
}

async fn init_single_process_grpc_scheduler(
    cfg: protot::core::Config,
    opts: ProcessOptions,
) -> Result<(), SchedulerError> {

    debug!("configs dump: {:#?}", cfg);

    let registry = opts.task_executors;

    let executors = Arc::new(Mutex::new(registry));

    let pool = core::worker_pool::Builder::new()
        .num_workers(cfg.num_workers as usize)
        .name(opts.process_name)
        .thread_stack_size(32 * 1024 * 1024)
        .executors(executors)
        .build()?;

    collect_stats();


    // For examples
    // pool.execute(
    //     |args| println!("sanity check: {:#?}", args),
    //     ExecuteRequest {
    //         task: Some(Task {
    //             id: "sanity-1".to_string(),
    //             ..Default::default()
    //         }),
    //     },
    // ).expect("Oops. Something gone terribly wrong");

    // {
    //     pool.executors
    //         .lock()
    //         .unwrap()
    //         .register_task("task-1", TaskExecutorImpl1 {});
    //     pool.executors
    //         .lock()
    //         .unwrap()
    //         .register_task("task-2", TaskExecutorImpl2 {});
    // }

    // Todo start scheduler server
    match start_single_process_grpc_server(cfg.grpc_port, pool, cfg.graceful_timeout).await {
        Err(err) => Err(SchedulerError::SchedulerServiceError(format!(
            "Scheduler errored: {:?}",
            &*err
        )))?,
        Ok(_) => println!("Goodbye :)"),
    };

    Ok(())
}
