use prost_types::{Struct, Any};
use std::collections::BTreeMap;
use protot::internal::protot::{
    core::Task,
    scheduler::v1::{
        ExecuteRequest,
        scheduler_service_client,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the gRPC client
    
    let mut client = scheduler_service_client::SchedulerServiceClient::connect("http://[::1]:44880")
        .await?;

    // Create Struct for payload
    let mut fields = BTreeMap::new();
    fields.insert("some_task_key".to_string(), prost_types::Value { kind: Some(prost_types::value::Kind::StringValue("some_task_value".to_string())) });

    let payload = Struct { fields };
    let mut any = Any::default();
    any.type_url = "type.googleapis.com/google.protobuf.Struct".to_string();
    any.value = prost::Message::encode_to_vec(&payload);

    // Create Tasks
    let task1 = Task {
        id: "task-1".to_string(),
        payload: Some(any),
        ..Default::default()
    };
    // You can create more tasks similarly.

    // Create Execute Request
    let exec_req_1 = ExecuteRequest {
        task: Some(task1),
        ..Default::default()
    };

    // Execute the task
    let response = client.execute(exec_req_1).await?;

    println!("Response = {:?}", response);

    Ok(())
}