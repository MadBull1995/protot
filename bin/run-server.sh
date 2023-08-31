#!/bin/bash

if [[ $1 == "debug" ]]
then
	echo "Debug Mode: $1"
	GRPC_VERBOSITY=DEBUG GRPC_TRACE=all PYTHONPATH=./generated/services:./:./generated/services/protos python3 generated/server/server.py
elif [[ $1 == "info" ]]
then
	echo "Info Mode: $1"
	GRPC_VERBOSITY=INFO GRPC_TRACE=all PYTHONPATH=./generated/services:./:./generated/services/protos python3 generated/server/server.py
else
	PYTHONPATH=./generated/services:./:./generated/services/protos python3 generated/server/server.py
fi