use crate::runner::{TaskRunner, TaskStatus};
use std::collections::HashSet;
use std::sync::Arc;

/// Runs the specified tasks (or all tasks if targets is empty) concurrently,
/// honoring dependencies. Blocks until execution completes.
pub fn run_non_tui(runner: &TaskRunner, targets: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    // Determine the set of tasks that need to run
    let target_subgraph = if targets.is_empty() {
        runner.tasks.keys().cloned().collect::<HashSet<String>>()
    } else {
        // Validate target task names exist
        for target in targets {
            if !runner.tasks.contains_key(target) {
                return Err(format!("Task '{}' not found in configuration", target).into());
            }
        }
        runner.get_subgraph(targets)
    };

    // Trigger parallel execution via the task scheduler
    if targets.is_empty() {
        runner.run_all();
    } else {
        runner.run_tasks(targets);
    }

    // Monitor the execution states in a loop on the main thread
    let states = Arc::clone(&runner.states);
    let condvar = Arc::clone(&runner.condvar);

    loop {
        let states_guard = states.lock().unwrap();
        let mut any_active = false;
        
        for name in &target_subgraph {
            if let Some(state) = states_guard.get(name) {
                if state.status == TaskStatus::Pending || state.status == TaskStatus::Running {
                    any_active = true;
                    break;
                }
            }
        }

        if !any_active {
            // Execution finished. Check if any tasks failed.
            let mut failed_tasks = Vec::new();
            for name in &target_subgraph {
                if let Some(state) = states_guard.get(name) {
                    if state.status == TaskStatus::Failed {
                        failed_tasks.push(name.clone());
                    }
                }
            }
            if !failed_tasks.is_empty() {
                return Err(format!("Some tasks failed: {}", failed_tasks.join(", ")).into());
            }
            break;
        }

        // Wait for states to change
        let _guard = condvar.wait(states_guard).unwrap();
    }

    Ok(())
}
