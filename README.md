# Sylklabs Scheduler

![Sylklabs Scheduler Logo](logo.png)

Sylklabs Scheduler is a distributed task scheduling application designed to efficiently distribute tasks across worker nodes.

## Table of Contents
- [Main Components](#main-components)
- [Installation](#installation)
- [Usage](#usage)
- [Features](#features)
- [Configuration](#configuration)
- [Examples](#examples)
- [Docker](#docker)
- [License](#license)
- [Contributing](#contributing)
- [Contact](#contact)

### Main Components
In a nutshell, the architecture centers around a `WorkerPool` that manages a set of task execution units, known as `Workers`. These workers can be of different types like `LocalWorker` and are __thread-safe__, __atomic__ units governed by the `Worker` trait. They operate in sync through `WorkerPoolSharedData`, which maintains shared state and metrics. Task execution capabilities are abstracted by the `TaskExecutor` trait and organized by a `TaskRegistry`. The pool's configuration is customizable via a `Builder` struct. Meanwhile, a `Sentinel` struct oversees the health of the workers in the pool. 

This cohesive system efficiently __distributes__ tasks, maintains __synchronized__ state, and allows for __future extensibility__.

#### General Flow
1. A `TaskRegistry` is initialized to hold all task executors.
2. A `WorkerPool` is created using `Builder`, with options to specify various configurations like the number of workers, force shutdown, etc.
3. Tasks are registered in `TaskRegistry`.
4. `WorkerPool` executes tasks based on the registered executors.

## Installation

Clone this repository and navigate to the root folder. Build the scheduler using the following command:

```sh
$ cargo build --release
# Build with stats collection
$ cargo build --release --features "stats"
```

After building the scheduler, the executable will be available under `target/release/`.

## Usage

Run the scheduler by executing the binary. You can provide a configuration file as a command-line argument to customize the behavior of the scheduler.

```sh
$ ./proto_tasker --config=config.yaml
```

## Features

- **Distributed Task Scheduling**: Efficiently distributes tasks across worker nodes.
- **Configuration Loading**: Supports JSON and YAML configuration files for easy customization.
- **Scalability**: Designed to scale and adapt to varying workloads and available resources.

## Configuration

The scheduler's behavior can be customized using a JSON or YAML configuration file. Please refer to our [documentation](#) for the expected format and available options.

## Examples

Here's a Rust code snippet showing a simple example:

```rust,no_run
use proto_tasker::config_load;

// Initializing and running the scheduler.
fn main() {
    // Load configuration from command-line arguments or defaults.
    let config = config_load(String::from("my_config.yaml"));

    // Initialize the scheduler using the provided configuration.

    // Start scheduling tasks.
}
```


## Docker

Run the docker'ed version:

```bash
docker compose up --build -d
```

Components:
- `proto_tasker_app`: Scheduler service main process
- `grafana`: Visualizing metrics for scheduler
- `prom`: Collect and expose the internal metrics collected by `src/server/metrics.rs` module

See `Dockerfile` and `docker-compose.yml` for more details.


## License

This project is licensed under the terms of the Apache License, Version 2.0.

## Contributing

Contributions are welcome! Please read our [contributing guidelines](CONTRIBUTING.md) for more information.

## Contact

For any questions or clarifications, feel free to reach out to us at `contact@sylk.build`.

* * *

__This project and README file has been created thanks to [sylk.build](https://www.sylk.build)__ © 2023 Sylklabs Technologies