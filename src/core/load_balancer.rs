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