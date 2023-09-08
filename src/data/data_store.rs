use async_trait::async_trait;

use crate::{internal::protot::{scheduler::v1::ExecuteRequest, core::TaskState}, SchedulerError};


#[derive(Debug)]
pub enum DataStoreError {
    NotFound,
    Duplicate,
    InternalError(String),
}

#[async_trait]
pub trait DataStore: Send + Sync + 'static {
    async fn add_task_execution(&self, execute: ExecuteRequest) -> Result<(), SchedulerError>;
    async fn get_task_executions_by_worker(&self, state: TaskState) -> Result<Vec<ExecuteRequest>, SchedulerError>;
    async fn get_tasks_executions_by_state(&self, state: TaskState) -> Result<Vec<ExecuteRequest>, SchedulerError>;
    async fn update_task_execution_state(&self, task_id: &str, execution_id: &str, new_state: &TaskState) -> Result<(), SchedulerError>;
}
