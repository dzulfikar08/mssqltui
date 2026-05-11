# Contributing to mssqltui

Thanks for your interest in contributing!

## Reporting Bugs

1. Check [existing issues](https://github.com/dzulfikar08/mssqltui/issues) to avoid duplicates
2. Open a new issue with:
   - **OS and version** (macOS, Linux, Windows)
   - **mssqltui version** (`mssqltui --version`)
   - **SQL Server version**
   - Steps to reproduce
   - Expected vs actual behavior

## Suggesting Features

Open an issue with the label `enhancement`. Describe the feature and why it would be useful.

## Development Setup

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- Access to a SQL Server instance for testing

### Build and Run

```bash
git clone https://github.com/dzulfikar08/mssqltui.git
cd mssqltui
cargo run -- -s localhost -d master -u sa -P 'password'
```

### Run Tests

```bash
cargo test
```

## Submitting Pull Requests

1. Fork the repository
2. Create a feature branch: `git checkout -b my-feature`
3. Make your changes
4. Ensure `cargo build` and `cargo test` pass
5. Commit with a clear message
6. Open a PR against the `main` branch

### PR Guidelines

- One feature or fix per PR
- Keep changes focused and minimal
- Include a description of what changed and why

## Code Style

- Follow standard Rust conventions (`cargo fmt`)
- Run `cargo clippy` and address warnings
- Keep functions focused and readable
