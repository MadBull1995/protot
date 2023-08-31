"""sylk.build service implemantation for -> SchedulerWorkerService"""
import grpc
from google.protobuf.timestamp_pb2 import Timestamp
from typing import Iterator
from generated.services.protos.sylklabs.scheduler.v1 import scheduler_worker_pb2_grpc, scheduler_worker_pb2

class SchedulerWorkerService(scheduler_worker_pb2_grpc.SchedulerWorkerServiceServicer):

	# @rpc @@sylk - DO NOT REMOVE
	def Communicate(self, request: Iterator[scheduler_worker_pb2.WorkerMessage], context: grpc.ServicerContext) -> Iterator[scheduler_worker_pb2.SchedulerMessage]:
		# responses = [scheduler_worker_pb2.SchedulerMessage(scheduler_message_type=None)]
		# for res in responses:
		#    yield res

		super().Communicate(request, context)

