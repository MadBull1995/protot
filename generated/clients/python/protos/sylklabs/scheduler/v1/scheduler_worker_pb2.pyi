from sylklabs.core import task_pb2 as _task_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Iterable as _Iterable, Mapping as _Mapping, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class RegistrationRequest(_message.Message):
    __slots__ = ["worker_id", "supported_tasks"]
    WORKER_ID_FIELD_NUMBER: _ClassVar[int]
    SUPPORTED_TASKS_FIELD_NUMBER: _ClassVar[int]
    worker_id: str
    supported_tasks: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, worker_id: _Optional[str] = ..., supported_tasks: _Optional[_Iterable[str]] = ...) -> None: ...

class AssignTaskRequest(_message.Message):
    __slots__ = ["task"]
    TASK_FIELD_NUMBER: _ClassVar[int]
    task: _task_pb2.Task
    def __init__(self, task: _Optional[_Union[_task_pb2.Task, _Mapping]] = ...) -> None: ...

class WorkerMessage(_message.Message):
    __slots__ = ["registration"]
    REGISTRATION_FIELD_NUMBER: _ClassVar[int]
    registration: RegistrationRequest
    def __init__(self, registration: _Optional[_Union[RegistrationRequest, _Mapping]] = ...) -> None: ...

class SchedulerMessage(_message.Message):
    __slots__ = ["assign_task"]
    ASSIGN_TASK_FIELD_NUMBER: _ClassVar[int]
    assign_task: AssignTaskRequest
    def __init__(self, assign_task: _Optional[_Union[AssignTaskRequest, _Mapping]] = ...) -> None: ...
