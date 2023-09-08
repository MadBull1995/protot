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

/// Enumerates all possible errors that can occur within the Scheduler.
///
/// This error type contains variations for each type of error that can
/// occur, including errors related to configuration loading, task execution, and
/// thread pool creation.
///
/// Each variant contains a `String` message that provides additional details about the error.
///
/// # Examples
///
/// ```rust,no_run
/// use protot::SchedulerError; // Replace with actual import
///
/// let error = SchedulerError::ConfigLoadError("File not found".to_string());
/// match error {
///     SchedulerError::ConfigLoadError(msg) => println!("Error loading config: {}", msg),
///     _ => println!("Some other error occurred"),
/// }
/// ```
#[derive(Debug)]
pub enum SchedulerError {
    /// Represents an error that occurs when loading the configuration.
    ///
    /// The contained string provides additional details about the failure.
    ConfigLoadError(String),

    /// Represents an error that occurs during the creation of the thread pool.
    ///
    /// The contained string provides additional details about the failure.
    PoolCreationError(String),

    /// Represents an error that occurs during the execution of a task.
    ///
    /// The contained string provides additional details about the failure.
    TaskExecutionError(String),

    /// Represents an error that occurs beacause lack of support.
    ///
    /// The contained string provides additional details about the implementaion lack of support.
    SchedulerUnimplemented(String),

    SchedulerServiceError(String),
    LoggerSetupError(String),
    DataLayerError(String),
}

/// Implementation of the `std::fmt::Display` trait for `SchedulerError`.
///
/// This allows a `SchedulerError` instance to be converted to a string representation,
/// making it easier to output the error information.
impl std::fmt::Display for SchedulerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SchedulerError::ConfigLoadError(msg) => write!(f, "Config load error: {}", msg),
            SchedulerError::TaskExecutionError(msg) => write!(f, "Task execution error: {}", msg),
            SchedulerError::PoolCreationError(msg) => write!(f, "Pool creation error: {}", msg),
            SchedulerError::SchedulerUnimplemented(msg) => write!(f, "Unimplemented: {}", msg),
            SchedulerError::SchedulerServiceError(msg) => {
                write!(f, "Scheduler service error: {}", msg)
            }
            SchedulerError::LoggerSetupError(msg) => {
                write!(f, "Scheduler logger error: {}", msg)
            } // _ => write!(f, "Unknown internal scheduler error"),
            SchedulerError::DataLayerError(msg) => {
                write!(f, "Scheduler data layer error: {}", msg)
            }
        }
    }
}

/// Implementation of the `std::error::Error` trait for `SchedulerError`.
///
/// By implementing this trait, `SchedulerError` can be used with interfaces that
/// expect a type implementing the `Error` trait, such as error-chain or
/// functions that return a `Result`.
impl std::error::Error for SchedulerError {}
