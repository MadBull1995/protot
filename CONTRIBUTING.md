# Contributing to ProtoT

Thank you for your interest in contributing to ProtoT! We appreciate your effort and are pleased to guide you through the contribution process.

Before contributing, please read this document carefully as it outlines the code of conduct, the development process, and the steps for submitting a pull request. Your contribution could be code, documentation, bug reports, or feature requests—every contribution counts.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Setting Up Local Environment](#setting-up-local-environment)
- [Submitting an Issue](#submitting-an-issue)
- [Contributing Code](#contributing-code)
  - [Feature Requests](#feature-requests)
  - [Bug Fixes](#bug-fixes)
  - [Creating a Pull Request](#creating-a-pull-request)
- [Documentation](#documentation)
- [Code Review Process](#code-review-process)
- [License](#license)
- [Contact](#contact)

## Code of Conduct

We expect all contributors to adhere to our [Code of Conduct](CODE_OF_CONDUCT.md). Please take a moment to read it.

## Getting Started

### Prerequisites

- Git
- Rust
- Cargo
- Protocol Buffers Compiler (optional)

### Setting Up Local Environment

1. Fork the repository to your GitHub account.
2. Clone the forked repository locally:

    ```sh
    git clone https://github.com/madbull1995/protot.git
    ```

3. Navigate to the project directory:

    ```sh
    cd protot
    ```

4. Add upstream:

    ```sh
    git remote add upstream https://github.com/madbull1995/protot.git
    ```

5. Pull the latest changes from upstream:

    ```sh
    git pull upstream main
    ```

6. Install dependencies:

    ```sh
    cargo install
    ```

## Submitting an Issue

Issues can be feature requests, bug reports, or other inquiries. Make sure to go through existing issues to avoid duplication. Use the issue templates provided to describe your issue clearly.

## Contributing Code

### Feature Requests

1. First, create an issue describing the feature you want to work on (or confirm it’s already an issue).
2. Assign the issue to yourself.
3. Work on your feature on a new branch:

    ```sh
    git checkout -b feature/your-feature-name
    ```

### Bug Fixes

1. Find an issue to work on or create one that describes the bug you are fixing.
2. Assign the issue to yourself.
3. Create a new branch:

    ```sh
    git checkout -b fix/your-bug-name
    ```

### Creating a Pull Request

1. Commit your changes and push your branch to your forked repository:

    ```sh
    git push origin your-branch-name
    ```

2. Go to GitHub and create a pull request from your branch to the ProtoT `main` branch.
3. Address any code review feedback.

## Documentation

Your contribution to documentation—whether it's fixing typos, adding code comments, or creating tutorials—is very valuable. Please create a separate pull request for documentation changes.

## Code Review Process

- Maintain code quality by adhering to the coding standards and guidelines of the project.
- Code reviews will be conducted by maintainers and contributors.
- For significant changes, consider squashing your commits to keep the history clean.
- Once the review process is complete and all tests pass, the pull request will be merged.

## License

By contributing to Sylklabs Task Scheduler, you agree to license your contributions under the Apache License, Version 2.0.

## Contact

For further questions, feel free to contact us at `contact@sylk.build`.

Thank you for contributing to ProtoT! We look forward to collaborating with you.