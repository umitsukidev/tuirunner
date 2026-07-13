use crate::runner::{TaskRunner, TaskStatus};
use std::collections::HashSet;

/// Runs the specified tasks (or all tasks if targets is empty) concurrently,
/// honoring dependencies. Blocks until execution completes.
pub async fn run_non_tui(
    runner: &TaskRunner,
    targets: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
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

    // Trigger parallel execution via the task scheduler and wait for it to complete
    let handle = if targets.is_empty() {
        runner.run_all()
    } else {
        runner.run_tasks(targets)
    };

    // Wait for the scheduler to complete
    let _ = handle.await;

    // Execution finished. Check if any tasks failed.
    let states_guard = runner.states.lock().unwrap();
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

    Ok(())
}
