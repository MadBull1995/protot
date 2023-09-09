# Developer's Docker Guide for ProtoT

## Overview

This guide is intended for developers working on `ProtoT`. We use Docker and Docker Compose to manage the project's containers, which include services for the main application, Prometheus, Grafana, and Redis.

## Prerequisites

- Docker
- Docker Compose

## Getting Started

### Clone the Repository

First, clone the repository to your local machine.

```bash
git clone https://github.com/MadBull1995/protot.git
cd protot
```

### Building the Docker Image

You can build the Docker image for the Proto Tasker application by navigating to the root directory and running:

```bash
docker build -t protot .
```

This will build the Docker image based on the provided `Dockerfile`.

### Using Docker Compose

Docker Compose is used to run multiple services as defined in `docker-compose.yml`. To start all services, navigate to the directory containing `docker-compose.yml` and run:

```bash
docker-compose up
```

This will bring up:

- `proto_tasker_app` (the main application)
- `prom` (Prometheus)
- `grafana` (Grafana)
- `redis` (Redis)

### Environment Variables

The Docker Compose file specifies environment variables for the Proto Tasker application:

- `PROTOT_GRPC_PORT`: The gRPC port for the application.
- `PROTOT_REDIS_HOST`: The Redis host URL.

You can change these variables in the `docker-compose.yml` file if necessary.

### Accessing Services

After running `docker-compose up`, you can access the services at the following addresses:

- Proto Tasker App: `http://localhost:44880`
- Prometheus: `http://localhost:9090`
- Grafana: `http://localhost:3000`
- Redis: `http://localhost:6379`

## Cleaning Up

To stop all services, you can run:

```bash
docker-compose down
```

## Advanced Configuration

For advanced configuration options, please refer to the project documentation.