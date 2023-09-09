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

extern crate lazy_static;

use protot::config_load;
use protot::internal::protot::core::{Config, DataStore, DataStoreType, NodeType};
use protot::utils::{get_ascii_logo, get_protot_metadata};
use protot::{
    core::worker_pool::TaskRegistry, start, SchedulerError, TaskExecutorImpl1, TaskExecutorImpl2,
};
use clap::{Parser, Subcommand};
use std::process::exit;
use std::{path::PathBuf, ops::RangeInclusive};
use std::fs;

/// Task Scheduler Server
#[derive(Parser)]
#[clap(version = "0.1", author = "Amit Shmulevitch")]
struct Cli {
    /// Sets a custom config file
    #[clap(short, long, default_value = "configs.yaml", value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
 
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// initialize new scheduler server
    Init {
        /// data store host
        #[arg(short, long)]
        data_host: Option<String>,

        /// the allocated num of workers (threads) to be used by scheduler
        #[arg(long, value_parser = num_workers_in_range, default_value = "4")]
        num_workers: u16,

        /// the gRPC server port for scheduler
        #[arg(short, long, value_parser = port_in_range, default_value = "44880")]
        grpc_port: u16,
    },
}

const PORT_RANGE: RangeInclusive<usize> = 1..=65535;

fn port_in_range(s: &str) -> Result<u16, String> {
    let port: usize = s
        .parse()
        .map_err(|_| format!("`{s}` isn't a port number"))?;
    if PORT_RANGE.contains(&port) {
        Ok(port as u16)
    } else {
        Err(format!(
            "port not in range {}-{}",
            PORT_RANGE.start(),
            PORT_RANGE.end()
        ))
    }
}

fn num_workers_in_range(s: &str) -> Result<u16, String> {
    let num_workers: usize = s
        .parse()
        .map_err(|_| format!("`{s}` isn't a valid worker number"))?;
    if num_cpus::get() >= num_workers {
        Ok(num_workers as u16)
    } else {
        Err(format!(
            "num of workers {} is greater then the available CPU's {}",
            num_workers,
            num_cpus::get()
        ))
    }
}

fn check_config_file_available(config_path: PathBuf) -> Result<(), String> {
    if config_path.exists() && config_path.is_file() {
        // Try to open the file to ensure it's readable
        match fs::File::open(&config_path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to open config file: {}", e)),
        }
    } else {
        Err(format!("Config file {} does not exist or is not a regular file", config_path.display()))
    }
}

#[tokio::main]
async fn main() -> Result<(), SchedulerError> {
    let cli: Cli = Cli::parse();
    println!("{}\n{}", get_ascii_logo(), get_protot_metadata());
    let mut executors = TaskRegistry::new();
    
    let loaded_cfgs = match cli.config {
        Some(cfg_file) => {
            let file_path = cfg_file.clone();
            check_config_file_available(cfg_file).map_err(|err| panic!("{}", err)).unwrap();
            config_load(String::from(file_path.to_str().unwrap()))
        }
        None => config_load("configs.yaml".to_string())
    };

    let mut cfgs = Config::default();
    if loaded_cfgs.is_ok() {
        cfgs = loaded_cfgs.unwrap();
    }

    match cli.command {
        Some(Commands::Init { 
            data_host,
            num_workers,
            grpc_port 
        }) => {
            
            if let Some(cli_data_host) = data_host {
                cfgs.data_store = Some(DataStore {
                    host: cli_data_host,
                    ..Default::default()
                });
            }

            cfgs.num_workers = num_workers as i32;
            cfgs.grpc_port = grpc_port as i32;
            cfgs.node_type = NodeType::Scheduler.into();
        },
        None => {
            println!("GoodBye :)");
            exit(1);
        }
    }

    start(executors, Some(cfgs)).await?;

    // executors.register_task("task-1", TaskExecutorImpl1 {});
    // executors.register_task("task-2", TaskExecutorImpl2 {});


    // let mut client = SchedulerServiceClient::connect("http://[::1]:50051").await.unwrap();

    // let future = execute_task!(&mut client, "some-task-1").await;
    // println!("{:?}", future);

    Ok(())
    // let scheduler = Scheduler::new(pool);
}
