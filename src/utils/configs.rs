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

use prost_types::Duration;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_yaml;
use std::fs;
use crate::internal::protot::core::{self, Config};

use super::error::SchedulerError; // Import Serialize and Deserialize traits

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[allow(non_camel_case_types)]
pub enum NodeType {
    #[serde(rename = "SINGLE_PROCESS")]
    SingleProcess,
    #[serde(rename = "WORKER")]
    Worker,
    #[serde(rename = "SCHEDULER")]
    Scheduler,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[allow(non_camel_case_types)]
pub enum LoadBalancer {
    #[serde(rename = "ROUND_ROBIN")]
    RoundRobin,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WrapperDuration {
    pub seconds: i64,
    pub nanos: i32,
}

#[derive(Debug, Serialize, Deserialize)] // Use the derive macros for serialization and deserialization
pub struct SerdeConfig {
    #[serde(rename = "node_type")]
    node_type: NodeType,
    #[serde(rename = "num_workers")]
    num_workers: i32,
    #[serde(rename = "grpc_port")]
    grpc_port: i32,
    #[serde(rename = "graceful_timeout")]
    graceful_timeout: u64,
    #[serde(rename = "load_balancer")]
    load_balancer: LoadBalancer,
    #[serde(rename = "heartbeat_interval")]
    heartbeat_interval: Option<WrapperDuration>,
}

#[allow(unused)]
fn serialize_to_json(config: &SerdeConfig) -> Result<String, Box<dyn std::error::Error>> {
    let json = serde_json::to_string(config)?;
    Ok(json)
}

#[allow(unused)]
fn serialize_to_yaml(config: &SerdeConfig) -> Result<String, Box<dyn std::error::Error>> {
    let yaml = serde_yaml::to_string(config)?;
    Ok(yaml)
}

fn deserialize_from_json(json: &str) -> Result<SerdeConfig, Box<dyn std::error::Error>> {
    let config: SerdeConfig = serde_json::from_str(json)?;
    Ok(config)
}

fn deserialize_from_yaml(yaml: &str) -> Result<SerdeConfig, Box<dyn std::error::Error>> {
    let config: SerdeConfig = serde_yaml::from_str(yaml)?;
    Ok(config)
}

/// Load configuration from a JSON or YAML file.
///
/// This function reads the content of a JSON or YAML configuration file from the specified path
/// and converts it into a `sylklabs::core::Config` struct. The appropriate deserialization method
/// is determined based on the file extension.
///
/// # Arguments
///
/// * `path` - The path to the configuration file (JSON or YAML).
///
/// # Returns
///
/// A `Result` containing either the deserialized `sylklabs::core::Config` or a `SchedulerError`
/// indicating the reason for failure.
///
/// # Examples
///
/// ```rust,no_run
/// use protot::{config_load, SchedulerError};
///
/// let config = config_load("config.yaml".to_string());
/// match config {
///     Ok(cfg) => println!("Loaded configuration: {:?}", cfg),
///     Err(err) => match err {
///         SchedulerError::ConfigLoadError(e) => eprintln!("Error loading configuration: {}", e),
///         _ => eprintln!("Error internal: panic")
///         // Handle other error cases as needed
///     }
/// }
/// ```
pub fn config_load(path: String) -> Result<Config, SchedulerError> {
    // Load the content of your JSON or YAML file
    let content = fs::read_to_string(&path).map_err(|e| {
        SchedulerError::ConfigLoadError(format!("failed to load configurations: {:?}", e))
    })?;

    // Determine the file format based on the extension
    let config: SerdeConfig = if path.ends_with(".yaml") || path.ends_with(".yml") {
        deserialize_from_yaml(&content).map_err(|e| {
            SchedulerError::ConfigLoadError(format!("failed to deserialize from yaml: {:?}", e))
        })?
    } else if path.ends_with(".json") {
        deserialize_from_json(&content).map_err(|e| {
            SchedulerError::ConfigLoadError(format!("failed to deserialize from json: {:?}", e))
        })?
    } else {
        return Err(SchedulerError::ConfigLoadError(
            "Unsupported configuration file format.".into(),
        ));
    };

    // Now you have your configuration struct populated
    println!("{:?}", config);

    let heartbeat_interval = config.heartbeat_interval.map(|custom_duration| {
        prost_types::Duration {
            seconds: custom_duration.seconds,
            nanos: custom_duration.nanos,
        }
    });

    let cfg = Config {
        grpc_port: config.grpc_port,
        node_type: match config.node_type {
            NodeType::Scheduler => core::NodeType::Scheduler.into(),
            NodeType::Worker => core::NodeType::Worker.into(),
            _ => core::NodeType::SingleProcess.into(),
        },
        num_workers: config.num_workers,
        graceful_timeout: config.graceful_timeout,
        load_balancer: match config.load_balancer {
            LoadBalancer::RoundRobin => core::LoadBalancer::RoundRobin.into(),
        },
        heartbeat_interval: heartbeat_interval,
    };

    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Define the test configuration structure
    #[derive(Debug, Deserialize, Serialize)]
    struct TestConfig {
        file: String,
        content: String,
    }

    // Load and deserialize test YAML files
    fn load_test_configs(file_path: &str) -> Vec<TestConfig> {
        let content = fs::read_to_string(file_path).unwrap();
        serde_yaml::from_str::<Vec<TestConfig>>(&content).unwrap()
    }

    #[test]
    fn test_load_and_deserialize_configs_from_yaml() {
        let test_configs = load_test_configs("tests/yaml_test.yml");

        for test_config in test_configs {
            let serde_config = deserialize_from_yaml(&test_config.content)
                .expect("Failed to deserialize test config");

            assert_eq!(serde_config.node_type, NodeType::Worker);
            assert_eq!(serde_config.num_workers, 4);
            assert_eq!(serde_config.grpc_port, 50051);
        }
    }

    #[test]
    fn test_load_and_deserialize_configs_from_json() {
        let test_configs = load_test_configs("tests/json_test.yml");

        for test_config in test_configs {
            let serde_config = deserialize_from_json(&test_config.content)
                .expect("Failed to deserialize test config");

            assert_eq!(serde_config.node_type, NodeType::Worker);
            assert_eq!(serde_config.num_workers, 4);
            assert_eq!(serde_config.grpc_port, 50051);
        }
    }
}
