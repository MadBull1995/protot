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
task=task_pb2.Task(
    id="task-1",
    payload=any
)
exec_req = scheduler_pb2.ExecuteRequest(
    task=task
)
sched_req = scheduler_pb2.ScheduleRequest(
    task=task
)

res = client.Execute(
    exec_req
)

# res = client.Schedule(
#     sched_req
# )

print(res)