"""sylk.build service implemantation for -> SchedulerService"""
import grpc
from google.protobuf.timestamp_pb2 import Timestamp
from typing import Iterator
from generated.services.protos.sylklabs.scheduler.v1 import scheduler_pb2_grpc, scheduler_pb2

class SchedulerService(scheduler_pb2_grpc.SchedulerServiceServicer):

	# @rpc @@sylk - DO NOT REMOVE
	def Execute(self, request: scheduler_pb2.ExecuteRequest, context: grpc.ServicerContext) -> scheduler_pb2.ExecuteResponse:
		# response = scheduler_pb2.ExecuteResponse(task_id=None,state=None)
		# return response

		super().Execute(request, context)

	# @rpc @@sylk - DO NOT REMOVE
	def Schedule(self, request: scheduler_pb2.ScheduleRequest, context: grpc.ServicerContext) -> scheduler_pb2.ScheduleResponse:
		# response = scheduler_pb2.ScheduleResponse(scheduled_task_id=None)
		# return response

		super().Schedule(request, context)

