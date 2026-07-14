# User Guide

This guide will help you install, configure, and use `tuirunner` (`tuir`).

---

## Installation

To install `tuirunner`, run the following command:

```bash
cargo install tuirunner
```

This compiles and installs the `tuir` binary into your Cargo bin directory (e.g., `~/.cargo/bin`), which should be in your system's `PATH`.

---

## Usage

By default, `tuir` looks for a configuration file named `runner.config.toml` in the current working directory.

```bash
# Show help message
tuir

# Run all tasks (starts TUI or runs in CLI mode as configured)
tuir run

# Run specific tasks and their dependency subgraphs
tuir run build test

# Run a single specific task directly (short for `tuir run <task-name>`)
# Note: Multiple tasks are not supported in this direct mode.
tuir build

# Run without TUI mode (CLI mode)
tuir --no-tui run

# Specify a custom configuration file path (JSON, YAML, or TOML)
tuir --config custom-config.yaml run
tuir -c custom-config.json run

# Initialize a new configuration file
tuir init
```

### CLI Subcommands

- **`init`**: Initialize a new configuration file in the current working directory (default: TOML format).
    - `--format <format>`: Format of the configuration file (toml, yaml, json).
    - `--toml`, `--yaml`, `--json`: Flags to choose the configuration format.
- **`run`**: Execute tasks.
    - `[TARGETS]...`: Specific task name(s) to run. If none are specified, all tasks defined in the configuration are executed.
- **`schema`**: Output the JSON schema of the configuration file and exit immediately. Useful for editor integration.
- **`completions <shell>`**: Generate shell completion script to stdout.

### Global Options

- **`-c, --config <path>`**: Path to the configuration file (default: `runner.config.toml`).
- **`--no-tui`**: Bypass the TUI interface and execute tasks directly in the CLI.

---

## Configuration

`tuirunner` supports configuration files in **TOML**, **YAML**, and **JSON** formats.

### Example: TOML Configuration (`runner.config.toml`)

```toml
#:schema ./runner.schema.json

tui = true

[tasks.clean]
description = "Clean up the build directory"
run = "echo 'Cleaning build directory...'; sleep 1; echo 'Cleaned!'"

[tasks.codegen]
description = "Generate source code"
run = "echo 'Running codegen...'; sleep 1.5; echo 'Codegen done!'"

[tasks.build]
description = "Build the application"
run = "echo 'Building...'; sleep 2"
depends_on = ["clean", "codegen"]

[tasks.test]
description = "Run test suite"
cmd = "echo 'Running unit tests...'; cargo test" # Prints command execution to logs/console
depends_on = ["build"]

[tasks.deploy]
description = "Deploy the application"
cmd = [
    "echo 'Deploying release...'",
    "sleep 1",
    "echo 'Deployed!'"
]
depends_on = ["build"]
working_dir = "./deploy_script"
```

### Configuration Fields

- **`tui`** (boolean, default: `true`): Enable or disable the TUI mode by default.
- **`tasks`** (map of objects): A map defining the tasks, where the key is the task name.
    - **`description`** (string, optional): A brief explanation of the task's purpose. Shown in the TUI list.
    - **`run`** (string or list of strings): Command(s) to execute silently. The command strings are executed under `sh -c`. (Mutually exclusive with `cmd`).
    - **`cmd`** (string or list of strings): Command(s) to execute. Similar to `run`, but prints the command invocation itself to the logs/console. (Mutually exclusive with `run`).
    - **`working_dir`** (string, optional): The directory where the commands will be executed.
    - **`depends_on`** (list of strings, optional): List of task names that must finish successfully before this task can start.

### JSON Schema Integration

To enable editor autocompletion and validation for your configuration files, you can generate a schema file:

```bash
tuir schema > runner.schema.json
```

Then reference it at the top of your configuration file:

- **TOML**: `#:schema ./runner.schema.json`
- **JSON**: Add `"$schema": "./runner.schema.json"` at the root.
- **YAML**: Use IDE-specific comments or workspace settings to map the schema.

---

## Interactive TUI Keybindings

When running in TUI mode, you can use the following keybindings to interact with the application:

| Key                       | Action                                                            |
| ------------------------- | ----------------------------------------------------------------- |
| `↑` / `↓` / `j` / `k`     | Move selection in the Task List                                   |
| `r` / `Enter`             | Run the selected task (and its dependencies)                      |
| `A`                       | Run all tasks in the list                                         |
| `s`                       | Stop execution of the selected task (skips downstream tasks)      |
| `S` (Shift+S)             | Stop execution of the selected task and continue downstream tasks |
| `c`                       | Clear output logs of the selected task                            |
| `C` (Shift+C)             | Clear output logs of all tasks                                    |
| `PgUp` / `PgDn`           | Scroll logs up/down by half a page                                |
| `Shift + ↑` / `Shift + ↓` | Scroll logs up/down line by line                                  |
| `i`                       | Enter "Interactive Input Mode" for the selected running task      |
| `Esc` (interactive mode)  | Exit Interactive Input Mode and return to standard TUI control    |
| `q` / `Esc`               | Quit the application                                              |
