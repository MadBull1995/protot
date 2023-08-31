from sylklabs.core import task_pb2 as _task_pb2
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class ScheduleRequest(_message.Message):
    __slots__ = ["task"]
    TASK_FIELD_NUMBER: _ClassVar[int]
    task: _task_pb2.Task
    def __init__(self, task: _Optional[_Union[_task_pb2.Task, _Mapping]] = ...) -> None: ...

class ScheduleResponse(_message.Message):
    __slots__ = ["scheduled_task_id"]
    SCHEDULED_TASK_ID_FIELD_NUMBER: _ClassVar[int]
    scheduled_task_id: int
    def __init__(self, scheduled_task_id: _Optional[int] = ...) -> None: ...

class ExecuteResponse(_message.Message):
    __slots__ = ["task_id", "state"]
    TASK_ID_FIELD_NUMBER: _ClassVar[int]
    STATE_FIELD_NUMBER: _ClassVar[int]
    task_id: int
    state: _task_pb2.TaskState
    def __init__(self, task_id: _Optional[int] = ..., state: _Optional[_Union[_task_pb2.TaskState, str]] = ...) -> None: ...

class ExecuteRequest(_message.Message):
    __slots__ = ["task"]
    TASK_FIELD_NUMBER: _ClassVar[int]
    task: _task_pb2.Task
    def __init__(self, task: _Optional[_Union[_task_pb2.Task, _Mapping]] = ...) -> None: ...
