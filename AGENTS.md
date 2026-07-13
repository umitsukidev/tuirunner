# Coding Guidelines & Architecture Rules

This repository follows a strict **React-like Component-Oriented Architecture** and **Separation of Concerns**. When writing or refactoring code, follow these principles:

## 1. React-like Component-Oriented Design (TUI Components)
TUI widgets should behave like modern declarative UI components:

- **Stateless Sub-Components (Props Down)**:
  - Sub-components (e.g., [TaskList](file:///Users/kyg/Documents/GitHub/umitsuki/tuirunner/src/components/task_list.rs), [FlowGraph](file:///Users/kyg/Documents/GitHub/umitsuki/tuirunner/src/components/flow_graph.rs)) should not manage their own internal states or directly mutate global state.
  - Pass read-only views or configuration structs (acting like React **Props**) during component instantiation or rendering. Use the [Store](file:///Users/kyg/Documents/GitHub/umitsuki/tuirunner/src/store.rs) to distribute shared data.
  
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
