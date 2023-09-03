#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Task {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub payload: ::core::option::Option<::prost_types::Any>,
}
/// The possible task states
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum TaskState {
    Pending = 0,
    Success = 1,
    Fail = 2,
}
impl TaskState {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            TaskState::Pending => "PENDING",
            TaskState::Success => "SUCCESS",
            TaskState::Fail => "FAIL",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "PENDING" => Some(Self::Pending),
            "SUCCESS" => Some(Self::Success),
            "FAIL" => Some(Self::Fail),
            _ => None,
        }
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Config {
    #[prost(enumeration = "NodeType", tag = "1")]
    pub node_type: i32,
    #[prost(int32, tag = "2")]
    pub num_workers: i32,
    #[prost(int32, tag = "3")]
    pub grpc_port: i32,
}
/// The possible node types for proto tasker process
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum NodeType {
    SingleProcess = 0,
    Worker = 1,
    Scheduler = 2,
}
impl NodeType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            NodeType::SingleProcess => "SINGLE_PROCESS",
            NodeType::Worker => "WORKER",
            NodeType::Scheduler => "SCHEDULER",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SINGLE_PROCESS" => Some(Self::SingleProcess),
            "WORKER" => Some(Self::Worker),
            "SCHEDULER" => Some(Self::Scheduler),
            _ => None,
        }
    }
}
