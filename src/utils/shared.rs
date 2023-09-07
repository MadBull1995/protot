use std::collections::HashMap;

use tokio::sync::mpsc;
use tonic::Status;

use crate::internal::protot::scheduler::v1::SchedulerMessage;

pub type GrpcWorkerChannels = HashMap<String, (mpsc::Sender<Result<SchedulerMessage, Status>>, mpsc::Sender<()>)>;