# TuiRunner (tuir)

`tuir` is a concurrent task runner with a terminal user interface (TUI) and command-line interface (CLI) written in Rust, built on top of [ratatui](https://github.com/ratatui/ratatui). It allows users to define a dependency graph of tasks and execute them concurrently with real-time log capturing.

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

### Installation

To install `tuir`, run the following command:

```bash
cargo install tuirunner
```

This will compile and install the `tuir` binary into your Cargo bin directory (e.g., `~/.cargo/bin`), which should be in your system's `PATH`.

### Usage

By default, `tuir` looks for [runner.config.toml](./runner.config.toml) in the current directory. Running `tuir` without any subcommand will display the help message. Use the `run` subcommand to execute tasks.

```bash
# Show help message
tuir

# Run all tasks in TUI or CLI mode (as configured)
tuir run

# Run specific tasks and their dependency subgraphs
tuir run build test

# Run without TUI mode (CLI mode)
tuir --no-tui run

# Use a custom config file (JSON, YAML, or TOML)
tuir --config custom-config.yaml run
tuir -c custom-config.json run
```

#### Commands

- `run`: Run tasks.
  - `[TARGETS]...`: Specific task name(s) to execute. If omitted, all tasks will be run.
- `schema`: Output the JSON schema of the configuration file and exit immediately. Useful for editor integration.
- `completions <shell>`: Generate shell completion script to stdout.

#### Global Options

- `-c <path>`, `--config <path>`: Path to the configuration file (default: [runner.config.toml](./runner.config.toml)). Can also use `--config=<path>`.
- `--no-tui`: Bypass the TUI interface and execute tasks directly in the CLI.

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

- **`tui`** (boolean, default: `false`): Enable/disable the TUI mode by default.
- **`tasks`** (map): A map of tasks where the key is the task name.
  - **`run`** (string or list of strings): Command(s) to execute silently. The command strings are executed under `sh -c`. (Mutually exclusive with `cmd`).
  - **`cmd`** (string or list of strings): Command(s) to execute. Similar to `run`, but prints command invocation to stdout/stderr. (Mutually exclusive with `run`).
  - **`working_dir`** (string, optional): Directory where the command will be run.
  - **`depends_on`** (list of strings, optional): Tasks that must finish successfully before this task can start.

### Interactive TUI Keybindings

When running in TUI mode, use the following keybindings to interact with the application:

| Key                       | Action                                       |
| ------------------------- | -------------------------------------------- |
| `↑` / `↓` / `j` / `k`     | Move selection in the Task List              |
| `r` / `Enter`             | Run the selected task (and its dependencies) |
| `a`                       | Run all tasks in the list                    |
| `c`                       | Clear output logs of the selected task       |
| `C` (Shift+C)             | Clear output logs of all tasks               |
| `PgUp` / `PgDn`           | Scroll logs up/down by half a page           |
| `Shift + ↑` / `Shift + ↓` | Scroll logs up/down line by line             |
| `q` / `Esc`               | Quit the application                         |

---

## Developer Guide

### Prerequisites

Ensure you have Rust (MSRV 1.85.0+ / 2024 edition) installed.

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

## License

This project is licensed under the MIT License. See [Cargo.toml](./Cargo.toml) for details.
