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

use async_trait::async_trait;

use crate::utils::shared::GrpcWorkerChannels;

#[async_trait]
pub trait LoadBalancer: Send + Sync + 'static {
    async fn select_worker(&mut self, channels: &GrpcWorkerChannels) -> Option<String>;
}

pub struct RoundRobinBalancer {
    current_worker: usize,
}

impl RoundRobinBalancer {
    pub fn new() -> Self {
        RoundRobinBalancer { current_worker: 0 }
    }
}

#[async_trait]
impl LoadBalancer for RoundRobinBalancer {
    async fn select_worker(&mut self, channels: &GrpcWorkerChannels) -> Option<String> {
        let keys: Vec<String> = channels.keys().cloned().collect();
        
        if keys.is_empty() {
            return None;
        }

        // Wrap around if we're out of bounds
        if self.current_worker >= keys.len() {
            self.current_worker = 0;
        }

        let key = keys.get(self.current_worker).cloned();

        self.current_worker += 1;
        key
    }
}