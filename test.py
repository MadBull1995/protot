from generated.clients.python import SchedulerService_v1
from generated.clients.python.protos.sylklabs.scheduler.v1 import scheduler_pb2
from generated.clients.python.protos.sylklabs.core import task_pb2

from google.protobuf.any_pb2 import Any
from google.protobuf.struct_pb2 import Struct
client = SchedulerService_v1()


data = Struct()
data.update({
    "some_task_key": "some_task_value"
})
any = Any()
any.Pack(data)
task1=task_pb2.Task(
    id="task-1",
    payload=any
)
task2=task_pb2.Task(
    id="task-2",
    payload=any
)
exec_req_1 = scheduler_pb2.ExecuteRequest(
    task=task1
)
exec_req_2 = scheduler_pb2.ExecuteRequest(
    task=task2
)
sched_req = scheduler_pb2.ScheduleRequest(
    task=task1
)

res = client.Execute(
    exec_req_1
)

res = client.Execute(
    exec_req_2
)

# res = client.Schedule(
#     sched_req
# )

print(res)