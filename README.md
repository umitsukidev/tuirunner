# TuiRunner (tuir)

[![CI](https://github.com/umitsukidev/tuirunner/actions/workflows/ci.yml/badge.svg)](https://github.com/umitsukidev/tuirunner/actions/workflows/ci.yml)
[![Crates.io Version](https://img.shields.io/crates/v/tuirunner)](https://crates.io/crates/tuirunner)
[![Crates.io Downloads](https://img.shields.io/crates/d/tuirunner)](https://crates.io/crates/tuirunner)
[![License](https://img.shields.io/crates/l/tuirunner)](LICENSE)

`tuir` is a concurrent task runner with a terminal user interface (TUI) and command-line interface (CLI) written in Rust, built on top of [ratatui](https://github.com/ratatui/ratatui). It allows users to define a dependency graph of tasks and execute them concurrently with real-time log capturing.

Read the [English guide](https://tuir.umitsuki.dev/guide.html) or the [日本語ガイド](https://tuir.umitsuki.dev/ja/guide.html).

## Features

- **TUI & CLI Modes**: Runs interactively with a beautiful terminal interface, or as a traditional CLI tool for CI/CD pipelines.
- **Topological Dependency Resolution**: Automatically resolves execution order and detects dependency cycles.
- **Concurrent Task Execution**: Non-dependent tasks run concurrently in separate threads.
- **Real-Time Logs**: View stdout and stderr in real-time. Supports auto-scrolling and manual scroll control in TUI mode.
- **Visual Execution Graph**: Displays a representation of the execution DAG (directed acyclic graph) with live task statuses.
- **Flexible Configurations**: Supports JSON, YAML, and TOML configuration file formats.
- **Built-in JSON Schema**: Generates schemas for IDE autocomplete and syntax validation.
- **Shell Command Flexibility**: Tasks can run commands silently (`run`) or show them in log buffers/console (`cmd`), as single command strings or arrays of sequential commands.

---

## User Guide

`tuirunner` is published on [crates.io](https://crates.io/crates/tuirunner).

### Installation

To install `tuir`, run the following command:

```bash
cargo install tuirunner
```

This will compile and install the `tuir` binary into your Cargo bin directory (e.g., `~/.cargo/bin`), which should be in your system's `PATH`.

### Quick Start

Create a configuration, then run it:

```bash
mkdir my-project && cd my-project
tuir init
tuir run
```

`tuir init` refuses to overwrite any existing `runner.config.toml`, `runner.config.yaml`, `runner.config.yml`, or `runner.config.json` file. Use `--format toml`, `--format yaml`, or `--format json` to choose a format.

### Usage

By default, `tuir` looks for [runner.config.toml](./runner.config.toml) in the current directory. Running `tuir` without any subcommand will display the help message. Use the `run` subcommand to execute tasks, or run a single task directly.

```bash
# Show help message
tuir

# Run all tasks in TUI or CLI mode (as configured)
tuir run

# Run specific tasks and their dependency subgraphs
tuir run build test

# Run a single specific task directly (short for `tuir run <task-name>`)
# Note: Multiple tasks are not supported in this direct mode.
tuir build

# Run without TUI mode (CLI mode)
tuir --no-tui run

# Use a custom config file (JSON, YAML, or TOML)
tuir --config custom-config.yaml run
tuir -c custom-config.json run

# Initialize a new configuration file
tuir init
```

#### Commands

- `init`: Initialize a new configuration file in the current directory (default format: TOML).
    - `--format <toml|yaml|json>`: Specify the configuration format.
    - `--toml`, `--yaml`, `--json`: Flags to choose the configuration format.
- `run`: Run tasks.
    - `[TARGETS]...`: Specific task name(s) to execute. If omitted, all tasks will be run.
- `schema`: Output the JSON schema of the configuration file and exit immediately. Useful for editor integration.
- `completions <shell>`: Generate shell completion script to stdout.

#### Global Options

- `-c <path>`, `--config <path>`: Path to the configuration file (default: [runner.config.toml](./runner.config.toml)). Can also use `--config=<path>`.
- `--no-tui`: Bypass the TUI interface and execute tasks directly in the CLI.

In non-TUI mode, `tuir` exits with a non-zero status if any selected task fails, which makes it suitable for CI scripts.

### Configuration

`tuir` supports TOML, YAML, and JSON. Below is an example of [runner.config.toml](./runner.config.toml):

```toml
#:schema ./runner.schema.json

tui = true

[tasks.clean]
run = "echo 'Cleaning build directory...'; sleep 1; echo 'Cleaned!'"

[tasks.codegen]
run = "echo 'Running codegen...'; sleep 1.5; echo 'Codegen done!'"

[tasks.build]
run = "echo 'Building...'; sleep 2"
depends_on = ["clean", "codegen"]

[tasks.test]
cmd = "echo 'Running unit tests...'; cargo test" # Prints command execution to console/logs
depends_on = ["build"]

[tasks.deploy]
cmd = [
    "echo 'Deploying release...'",
    "sleep 1",
    "echo 'Deployed!'"
]
depends_on = ["build"]
working_dir = "./deploy_script"
```

#### Configuration Fields

- **`tui`** (boolean, default: `true`): Enable/disable the TUI mode by default.
- **`tasks`** (map): A map of tasks where the key is the task name.
    - **`run`** (string or list of strings): Command(s) to execute silently. The command strings are executed under `sh -c`. (Mutually exclusive with `cmd`).
    - **`cmd`** (string or list of strings): Command(s) to execute. Similar to `run`, but prints command invocation to stdout/stderr. (Mutually exclusive with `run`).
    - **`working_dir`** (string, optional): Directory where the command will be run.
    - **`depends_on`** (list of strings, optional): Tasks that must finish successfully before this task can start.

Task names must be unique, cannot be `run`, and every name in `depends_on` must refer to a configured task. A task must use either `run` or `cmd`, not both.

### Interactive TUI Keybindings

When running in TUI mode, use the following keybindings to interact with the application:

| Key                       | Action                                                            |
| ------------------------- | ----------------------------------------------------------------- |
| `↑` / `↓` / `j` / `k`     | Move selection in the Task List                                   |
| `r` / `Enter`             | Run the selected task (and its dependencies)                      |
| `A`                       | Run all tasks in the list                                         |
| `s`                       | Stop execution of the selected task (skips downstream tasks)      |
| `S` (Shift+S)             | Stop execution of the selected task and continue downstream tasks |
| `c`                       | Clear output logs of the selected task                            |
| `C` (Shift+C)             | Clear output logs of all tasks                                    |
| `PgUp` / `PgDn`           | Scroll logs up/down by 10 lines                                   |
| `Shift + ↑` / `Shift + ↓` | Scroll logs up/down line by line                                  |
| `i`                       | Enter "Interactive Input Mode" for the selected running task      |
| `Esc` (interactive mode)  | Exit Interactive Input Mode and return to standard TUI control    |
| `q` / `Esc`               | Quit the application                                              |

---

## Developer Guide

### Prerequisites

Ensure you have Rust (MSRV 1.88.0+ / 2024 edition) installed.

### Building from Source

To build the executable manually from source:

```bash
git clone https://github.com/umitsukidev/tuirunner.git
cd tuirunner
cargo build --release
```

The compiled binary will be located at `target/release/tuir`.

### Development Tasks

#### Running Tests

```bash
cargo test
```

Before opening a release PR, run the same Rust checks used by CI:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -W clippy::all -D warnings
cargo test --all-targets --all-features
cargo package
```

#### JSON Schema Generation

To update the JSON schema for configuration autocomplete, generate and reference the [runner.schema.json](./runner.schema.json) file:

```bash
# Generate schema using cargo run
cargo run -- schema > runner.schema.json

# Or using mise if installed:
mise run schema
```

In your config files, you can link the schema as follows:

```toml
#:schema ./runner.schema.json
```

---

## Known Limitations & Constraints

- **Platform Support**: Native Windows environments are not fully supported out of the box. Running commands and terminating task process groups relies on Unix shell (`sh -c`) and POSIX signaling (`libc::kill` to negative PIDs). On Windows, it is highly recommended to run `tuir` inside a Unix-like environment such as **WSL (Windows Subsystem for Linux)** or **Git Bash**.
- **Interactive Input Mode**: The interactive mode (toggled by pressing `i` in the TUI) pipes keystrokes to the command's standard input stream. Because it is not a full pseudo-terminal (PTY) wrapper, fullscreen terminal applications (like `nano`, `vim`), advanced progress indicators, or tools prompting for input that rely on active terminal PTY queries may fail or not render properly.
- **Log Persistence**: Current logs are stored entirely in memory (`LogBuffer`) inside the TUI session. If you close the terminal interface, the execution logs are lost.

---

## Roadmap

We plan to implement the following features in future releases:

- **Native Windows Support**: Native command execution using `cmd.exe /C` or PowerShell, and native Windows process tree termination.
- **Log File Archiving**: A configuration/CLI parameter to automatically stream and persist task execution output to log files.
- **PTY Support Investigation**: Explore wrapping execution in a full pseudo-terminal (PTY) backend (e.g. using `portable-pty` or custom bindings) to fully support complex interactive console applications inside the TUI.

---

## Release Workflow

This repository has an automated release tagging workflow. When a Pull Request is merged into the `main` branch, a Git tag is automatically created and published if the PR title matches the format `release: <version>` (e.g., `release: v0.1.0-rc.1`).

- **Format**: `release: <version> [optional comments]` (The tag contains only the extracted version, e.g. `v0.1.0-rc.1`).
- **Version check**: Update the version in `Cargo.toml`, `Cargo.lock`, and `mise.toml` before merging. The version in `Cargo.toml` must match the release tag without its leading `v`.
- **Trigger**: Only a matching release PR merged into `main` creates a tag and runs the publish workflow. Normal pushes to `main` run quality CI but do not publish a release.

---

## License

This project is licensed under the MIT License. See [Cargo.toml](./Cargo.toml) for details.
