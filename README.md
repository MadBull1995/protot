# ProtoT: An Efficient Distributed Task Scheduling Engine Built in Rust

> **Note**
> Work In Progress

<!-- ![Sylklabs Logo](logo.png) -->

Welcome to ProtoT Scheduler, an innovative task scheduling engine engineered to intelligently allocate tasks across a distributed network of worker nodes.

## Why?

Developed to not only facilitate a deep understanding of Rust but also to offer a robust and extensible interface for task scheduling requirements. We leverage Protocol Buffers for efficient serialization, ensuring streamlined communication protocols between different components. Here, "T" stands for "Task", the core element of our scheduling logic.

## Table of Contents
- [Core Architecture](#core-architecture)
- [Getting Started](#getting-started)
  - [Installation](#installation)
  - [Usage](#usage)
- [Key Features](#key-features)
- [Customization](#customization)
  - [Configuration](#configuration)
  - [Examples](#examples)
- [Docker Deployment](#docker-deployment)
- [Protobuf Development](#protobuf-development)
- [License](#license)
- [Contributing](#contributing)
- [Contact Information](#contact-information)

## Core Architecture

Our system is architected around a dynamic `WorkerPool`, responsible for managing an array of task execution units known as `Workers`. These workers, which come in various types such as `LocalWorker`, are designed to be **thread-safe**, **atomic**, and adhere to the `Worker` trait. 

A shared data entity named `WorkerPoolSharedData` maintains state and metrics in a synchronized manner. Task management capabilities are abstracted by the `TaskExecutor` trait and are arranged via a `TaskRegistry`. 

Configurability is a first-class citizen in our architecture; all system parameters can be set using a `Builder` struct. Additionally, a `Sentinel` structure keeps a vigilant eye on the health metrics of each worker in the pool. 

Our system architecture aims for:
- **Efficient Distribution** of tasks
- **Synchronized State Management**
- **High Extensibility** for future functionalities

### Workflow Overview
1. Initialize a `TaskRegistry` to accommodate all task executors.
2. Construct a `WorkerPool` via `Builder`, specifying configurations like the number of workers, force shutdown settings, etc.
3. Register the tasks within the `TaskRegistry`.
4. The `WorkerPool` begins task execution based on the executors registered.

## Getting Started

### Installation

Clone the repository and navigate to the project root. Use the following commands to build the scheduler:

```sh
$ cargo build --release
# To enable stats collection
$ cargo build --release --features "stats"
```

Upon successful build, the executable can be found under `target/release/`.

### Usage

Execute the scheduler using the binary. For advanced settings, a configuration file can be supplied as a command-line argument.

```sh
$ ./protot --config=config.yaml
```

## Key Features

- **Distributed Task Scheduling**: Optimized to allocate tasks across multiple worker nodes.
- **Flexible Configuration**: Supports both JSON and YAML configuration files.
- **Scalability**: Engineered to adapt to various workloads and resource availability.

## Customization

### Configuration

Tweak the behavior of the scheduler using either a JSON or YAML configuration file. Refer to our [comprehensive documentation](#) for format details and options.

### Examples

Here’s a quick Rust code snippet to illustrate basic usage:

```rust,no_run
use proto_tasker::config_load;

fn main() {
    let config = config_load(String::from("my_config.yaml"));
    // Further initialization and task scheduling logic here.
}
```

## Docker Deployment

Run the Dockerized version of our scheduler:

```bash
docker compose up --build -d
```

Components:
- `proto_tasker_app`: Main process of the scheduler service
- `grafana`: For metrics visualization
- `prom`: Metrics collection and exposure

Consult the `Dockerfile` and `docker-compose.yml` for additional information.

## Protobuf Development

ProtoT uses protocol buffers as main protocol for communicating with data across different components in the core library.
All the protobuf files we use and compile in `ProtoT` are located under `/protos` directory.

We use `Sylk Build CLI` to unify the way we structure our protobuf schema, for more details see [`protos/README.md`](/protos/README.md)

## License

This project is licensed under the Apache License, Version 2.0.

## Contributing

We appreciate community contributions. Kindly refer to our [Contributing Guidelines](CONTRIBUTING.md) for further details.

## Contact Information

For any queries or clarifications, please don’t hesitate to contact us at `contact@sylk.build`.

---

**This project and documentation are brought to you by [sylk.build](https://www.sylk.build)**  
© 2023 Sylklabs Technologies. All rights reserved.