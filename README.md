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
- [Dependencies](#dependencies)
- [License](#license)
- [Contributing](#contributing)
- [Contact](#contact)

### Main Components
In a nutshell, the architecture centers around the `WorkerPool` struct, which manages a group of workers and task execution. The pool is constructed via a `Builder` struct, allowing customization like setting the number of workers. Task execution skills are encapsulated in the `TaskExecutor` trait, which must be implemented by any struct that wants to handle well you guessed it.. tasks.

 These tasks and their corresponding executors are organized by the `TaskRegistry` struct. For health monitoring of the worker pool, we have the `Sentinel` struct. Finally, `WorkerPoolSharedData` serves as a shared memory, keeping track of key metrics and providing synchronization tools.

#### General Flow
1. A `TaskRegistry` is initialized to hold all task executors.
2. A `WorkerPool` is created using `Builder`, with options to specify various configurations like the number of workers, force shutdown, etc.
3. Tasks are registered in `TaskRegistry`.
4. `WorkerPool` executes tasks based on the registered executors.

## Installation

Clone this repository and navigate to the root folder. Build the scheduler using the following command:

```sh
$ cargo build --release
```

After building the scheduler, the executable will be available under `target/release/`.

## Usage

Run the scheduler by executing the binary. You can provide a configuration file as a command-line argument to customize the behavior of the scheduler.

```sh
$ ./scheduler --config=config.yaml
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

## Dependencies

The following external crates are used by the Sylklabs Scheduler:

- `sylklabs`: Core library for scheduler logic and data structures.
- `serde`: For serialization and deserialization.
- `clap`: For command-line argument parsing.

## License

This project is licensed under the terms of the Apache License, Version 2.0.

## Contributing

Contributions are welcome! Please read our [contributing guidelines](CONTRIBUTING.md) for more information.

## Contact

For any questions or clarifications, feel free to reach out to us at `contact@sylk.build`.

* * *

__This project and README file has been created thanks to [sylk.build](https://www.sylk.build)__ © 2023 Sylklabs Technologies