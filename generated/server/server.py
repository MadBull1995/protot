"""sylk.build Generated Server Code"""
_ONE_DAY_IN_SECONDS = 60 * 60 * 24
from concurrent import futures
import time
import grpc
from generated.services.protos.sylklabs.scheduler.v1 import scheduler_pb2_grpc as scheduler_v1_pb2_grpc
from generated.services.protos.sylklabs.scheduler.v1 import scheduler_worker_pb2_grpc as scheduler_worker_v1_pb2_grpc
from generated.services.SchedulerService.v1.SchedulerService import SchedulerService as SchedulerService_v1
from generated.services.SchedulerWorkerService.v1.SchedulerWorkerService import SchedulerWorkerService as SchedulerWorkerService_v1

def serve(host="0.0.0.0:44880"):
	server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
	scheduler_v1_pb2_grpc.add_SchedulerServiceServicer_to_server(SchedulerService_v1(),server)
	scheduler_worker_v1_pb2_grpc.add_SchedulerWorkerServiceServicer_to_server(SchedulerWorkerService_v1(),server)
	server.add_insecure_port(host)
	server.start()
	print("[*] Started sylk.build server at -> %s" % (host))
	try:
		while True:
			time.sleep(_ONE_DAY_IN_SECONDS)
	except KeyboardInterrupt:
		server.stop(0)

if __name__ == "__main__":
	serve()