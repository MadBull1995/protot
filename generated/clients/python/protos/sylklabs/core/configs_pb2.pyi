from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class NodeType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = []
    SINGLE_PROCESS: _ClassVar[NodeType]
    WORKER: _ClassVar[NodeType]
    SCHEDULER: _ClassVar[NodeType]
SINGLE_PROCESS: NodeType
WORKER: NodeType
SCHEDULER: NodeType

class Config(_message.Message):
    __slots__ = ["node_type", "num_workers", "grpc_port"]
    NODE_TYPE_FIELD_NUMBER: _ClassVar[int]
    NUM_WORKERS_FIELD_NUMBER: _ClassVar[int]
    GRPC_PORT_FIELD_NUMBER: _ClassVar[int]
    node_type: NodeType
    num_workers: int
    grpc_port: int
    def __init__(self, node_type: _Optional[_Union[NodeType, str]] = ..., num_workers: _Optional[int] = ..., grpc_port: _Optional[int] = ...) -> None: ...
