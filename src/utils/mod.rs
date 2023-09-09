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

pub mod configs;
pub mod error;
pub mod logger;
pub mod shared;

const NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &'static str = env!("CARGO_PKG_DESCRIPTION");
const REPOSITORY: &'static str = env!("CARGO_PKG_REPOSITORY");

/// Fetches the current timestamp.
pub fn current_timestamp() -> i64 {
    // Generate the current timestamp. This function can be more precise depending on your requirements.
    let start = std::time::SystemTime::now();
    let since_the_epoch = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs() as i64
}

pub fn get_ascii_logo() -> String {
    let logo = "
                             
    _____         _       _____ 
   |  _  |___ ___| |_ ___|_   _|
   |   __|  _| . |  _| . | | |  
   |__|  |_| |___|_| |___| |_|  
                                
   ".to_string();
   logo
}

pub fn get_protot_metadata() -> String {
    let metadata = vec![
        ("Application", NAME),
        ("Version", VERSION),
        ("Authors", AUTHORS),
        ("Description", DESCRIPTION),
        ("Github", REPOSITORY),
    ];

    let separator = "=".repeat(60);
    let mut formatted_metadata = String::new();

    formatted_metadata.push_str(&separator);
    formatted_metadata.push('\n');

    for (key, value) in metadata {
        formatted_metadata.push_str(&format!("{:<16}{}\n", key, value));
    }

    formatted_metadata.push_str(&separator);

    formatted_metadata
}