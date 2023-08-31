# proto-tasker

This project has been generated thanks to [```sylk.build```](https://www.sylk.build) !

This project is using gRPC as main code generator and utilize HTTP2 + protobuf protocols for communication.

# Index
Usage:
- [Python](#python)

Resources:
- [SchedulerService](#schedulerservice)
- [SchedulerWorkerService](#schedulerworkerservice)
- [core](#core)
- [scheduler](#scheduler)

# Services

## SchedulerService

__`Execute`__ [Unary]
- Input: [sylklabs.scheduler.v1.ExecuteRequest](#executerequest)
- Output: [sylklabs.scheduler.v1.ExecuteResponse](#executeresponse)

__`Schedule`__ [Unary]
- Input: [sylklabs.scheduler.v1.ScheduleRequest](#schedulerequest)
- Output: [sylklabs.scheduler.v1.ScheduleResponse](#scheduleresponse)

## SchedulerWorkerService

__`Communicate`__ [Bidi stream]
- Input: [sylklabs.scheduler.v1.WorkerMessage](#workermessage)
- Output: [sylklabs.scheduler.v1.SchedulerMessage](#schedulermessage)

# Packages

## `sylklabs.core`


<details id="#Task">
<summary><b>Task</b></summary>

### __Task__
: 
* __id__ [TYPE_STRING]


* __payload__ [[Any](#Any)]

</details>


<details id="#Config">
<summary><b>Config</b></summary>

### __Config__
: 
* __node_type__ [[NodeType](#NodeType)]


* __num_workers__ [TYPE_INT32]


* __grpc_port__ [TYPE_INT32]

</details>

## `sylklabs.scheduler.v1`


<details id="#ScheduleRequest">
<summary><b>ScheduleRequest</b></summary>

### __ScheduleRequest__
: 
* __task__ [[Task](#Task)]

</details>


<details id="#ScheduleResponse">
<summary><b>ScheduleResponse</b></summary>

### __ScheduleResponse__
: 
* __scheduled_task_id__ [TYPE_INT32]

</details>


<details id="#ExecuteResponse">
<summary><b>ExecuteResponse</b></summary>

### __ExecuteResponse__
: 
* __task_id__ [TYPE_INT32]


* __state__ [[TaskState](#TaskState)]

</details>


<details id="#ExecuteRequest">
<summary><b>ExecuteRequest</b></summary>

### __ExecuteRequest__
: 
* __task__ [[Task](#Task)]

</details>


<details id="#RegistrationRequest">
<summary><b>RegistrationRequest</b></summary>

### __RegistrationRequest__
: 
* __worker_id__ [TYPE_STRING]


* __supported_tasks__ [TYPE_STRING]

</details>


<details id="#AssignTaskRequest">
<summary><b>AssignTaskRequest</b></summary>

### __AssignTaskRequest__
: 
* __task__ [[Task](#Task)]

</details>


<details id="#WorkerMessage">
<summary><b>WorkerMessage</b></summary>

### __WorkerMessage__
: 
* __worker_message_type__ [TYPE_ONEOF]

</details>


<details id="#SchedulerMessage">
<summary><b>SchedulerMessage</b></summary>

### __SchedulerMessage__
: 
* __scheduler_message_type__ [TYPE_ONEOF]

</details>


# Usage

This project supports clients communication in the following languages:

### Python

```py
from clients.python import prototasker

client = prototasker()

# Unary call
response = stub.<Unary>(<InMessage>())
print(response)

# Server stream
responses = stub.<ServerStream>(<InMessage>())
for res in responses:
	print(res)

# Client Stream
requests = iter([<InMessage>(),<InMessage>()])
response = client.<ClientStream>(requests)
print(response)

# Bidi Stream
responses = client.<BidiStream>(requests)
for res in responses:
	print(res)
```


* * *
__This project and README file has been created thanks to [sylk.build](https://www.sylk.build)__