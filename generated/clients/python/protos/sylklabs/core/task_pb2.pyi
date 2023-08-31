from google.protobuf import any_pb2 as _any_pb2
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class TaskState(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = []
    PENDING: _ClassVar[TaskState]
    SUCCESS: _ClassVar[TaskState]
    FAIL: _ClassVar[TaskState]
PENDING: TaskState
SUCCESS: TaskState
FAIL: TaskState

class Task(_message.Message):
    __slots__ = ["id", "payload"]
    ID_FIELD_NUMBER: _ClassVar[int]
    PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    id: str
    payload: _any_pb2.Any
    def __init__(self, id: _Optional[str] = ..., payload: _Optional[_Union[_any_pb2.Any, _Mapping]] = ...) -> None: ...
