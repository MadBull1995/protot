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

use protot::{
    core::worker_pool::TaskRegistry, start, SchedulerError, TaskExecutorImpl1, TaskExecutorImpl2,
};
// use protot::internal::sylklabs::scheduler::v1::ExecuteRequest;
#[tokio::main]
async fn main() -> Result<(), SchedulerError> {
    let mut executors = TaskRegistry::new();

    executors.register_task("task-1", TaskExecutorImpl1 {});
    executors.register_task("task-2", TaskExecutorImpl2 {});

    start(executors, None).await?;

    // let mut client = SchedulerServiceClient::connect("http://[::1]:50051").await.unwrap();

    // let future = execute_task!(&mut client, "some-task-1").await;
    // println!("{:?}", future);

    Ok(())
    // let scheduler = Scheduler::new(pool);
}
