use std::sync::Arc;
use async_trait::async_trait;
use log::debug;
// use prost_types::Any;
use protobuf::well_known_types::any::Any;
use redis::{Client, RedisError, aio::Connection, RedisResult, AsyncCommands, ConnectionLike};
use tokio::sync::Mutex;
use crate::{internal::protot::{scheduler::v1::ExecuteRequest, core::{Task, TaskState}}, SchedulerError, utils::current_timestamp};

use super::data_store::{DataStore, DataStoreError};

pub struct RedisClient {
    connection: Connection,
}

pub struct RedisDataStore {
    con: Arc<Mutex<Connection>>,
}

impl RedisDataStore {
    pub async fn new(redis_url: &str) -> Result<Self, SchedulerError> {
        let cleaned_redis_host = redis_url.replace("\"", "");
        let redis_host = cleaned_redis_host.clone();
        let client = Client::open(cleaned_redis_host)
            .map_err(|err| SchedulerError::DataLayerError(format!("Error when calling redis host: {} {:?}", redis_host, err)))?;
        
        let con = client.get_async_connection().await
            .map_err(|_| SchedulerError::DataLayerError(format!("Unable to connect to redis host: {}", redis_host)))?;

        Ok(Self { con: Arc::new(Mutex::new(con)) })
    }
}

#[async_trait]
impl DataStore for RedisDataStore {
    async fn add_task_execution(&self, execute: ExecuteRequest) -> Result<(), SchedulerError> {
        let mut db = self.con.lock().await;
    
        let task = execute
            .task
            .as_ref()
            .ok_or_else(|| SchedulerError::DataLayerError("Task is missing in ExecuteRequest".to_string()))?;
        
        let payload = task
            .payload
            .as_ref()
            .ok_or_else(|| SchedulerError::DataLayerError("Payload is missing in Task".to_string()))?;
            
        let key = format!("task:{}", task.id);
        
        let mut val = Any::new();
        val.type_url = payload.type_url.clone();
        val.value = payload.value.clone();

        db.hset(&key, "execution", &val.to_string())
            .await
            .map_err(|_| SchedulerError::DataLayerError("Failed to add task execution".to_string()))?;
    
        Ok(())
    }

    async fn get_task_executions_by_worker(&self, state: TaskState) -> Result<Vec<ExecuteRequest>, SchedulerError> {
        todo!()
    }

    async fn get_tasks_executions_by_state(&self, state: TaskState) -> Result<Vec<ExecuteRequest>, SchedulerError> {
        todo!()
    }

    async fn update_task_execution_state(&self, task_id: &str, execution_id: &str, new_state: &TaskState) -> Result<(), SchedulerError> {
        todo!()
    }

}