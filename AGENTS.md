# Coding Guidelines & Architecture Rules

This repository follows a strict **React-like Component-Oriented Architecture** and **Separation of Concerns**. When writing or refactoring code, follow these principles:

## 1. React-like Component-Oriented Design (TUI Components)

TUI widgets should behave like modern declarative UI components:

- **Stateless Sub-Components & Pure Rendering (Props Down)**:
    - Sub-components (e.g., [TaskList](src/components/task_list.rs), [FlowGraph](src/components/flow_graph.rs)) should not manage their own internal states or directly mutate global state.
    - All widget rendering must be **pure, declarative, and free of side-effects or lock acquisitions**. Do NOT acquire mutex locks (e.g., `.lock()`) inside `Widget::render` or widget sub-methods.
    - Pass read-only views, primitive types, or configuration structs (acting like React **Props**) during component instantiation or rendering. Use the [Store](src/store.rs) to distribute shared data.
- **Non-Blocking UI & Cache Fallback**:
    - The main TUI event loop/rendering thread must never block waiting for background runner locks.
    - When retrieving dynamic state from shared mutexes, use `.try_lock()` rather than blocking `.lock()`.
    - Maintain a local state cache in `App`. If a `.try_lock()` fails because a background worker thread is holding the lock, immediately fall back and render using the last cached snapshot. This guarantees the UI remains responsive and free of frame drops or deadlocks.

- **Event Delegation (Events Up)**:
    - When a component captures user interaction (e.g., keyboard events), it should translate them into specialized event enums (e.g., `TaskListEvent`) and return them to the parent.
    - Do NOT trigger side-effects directly inside the child component. Let the parent component handle the event and update the application state.
- **Implement Widget Trait**:
    - Components should implement Ratatui's `Widget` trait to define their rendering layout.
    - Keep the `render` function focused strictly on UI formatting and draw operations.

## 2. Separation of Concerns (Responsibility Layers)

Maintain a strict decoupling between:

- **UI / Presentation Layer (`src/components/`)**:
    - Defines how components are rendered based on props.
    - Parses raw input events into semantic component events.
- **Application State & Orchestration (`src/app.rs`)**:
    - Acts as the root component (App/Page level).
    - Holds state, handles component-delegated events, and drives the render loop.
- **Execution & Domain Logic (`src/runner.rs`)**:
    - Manages background tasks, sub-processes, thread states, and log captures.
    - Decoupled from the TUI framework; interacts only via thread-safe states (`Arc`, `Mutex`).
- **Data Sharing Store (`src/store.rs`)**:
    - Serves as a read-only container for components to easily access current execution order and runner state.

## 3. Coding Style & Documentation

- **Group and Combine `use` Imports (Rust)**:
    - Group imports from the same crate/module together using nested paths (e.g. `use tokio::io::{AsyncBufReadExt, BufReader};` instead of separate `use` lines) to keep the import block clean and compact.

- **Relative Paths in Markdown**:
    - Always use relative paths (e.g. `src/components/task_list.rs`) instead of absolute paths (e.g. `file:///absolute/path/...`) when linking to files in markdown documents (such as README.md, AGENTS.md, etc.) to ensure links remain valid and portable across different environments.
