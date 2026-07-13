use std::collections::HashMap;
use crate::config::{Task, TasksConfig};

#[derive(Debug, PartialEq, Eq)]
pub enum RunnerError {
    DependencyCycle { cycle: Vec<String> },
}

impl std::fmt::Display for RunnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunnerError::DependencyCycle { cycle } => {
                write!(f, "Dependency cycle detected: {}", cycle.join(" -> "))
            }
        }
    }
}

impl std::error::Error for RunnerError {}

#[derive(Debug)]
pub struct TaskRunner {
    pub tasks: HashMap<String, Task>,
    pub execution_order: Vec<String>,
}

impl TaskRunner {
    pub fn new(tasks_config: TasksConfig) -> Result<Self, RunnerError> {
        let tasks = tasks_config.tasks;
        let execution_order = resolve_dependencies(&tasks)?;
        Ok(Self {
            tasks,
            execution_order,
        })
    }

    pub fn execution_order(&self) -> &[String] {
        &self.execution_order
    }
}

fn resolve_dependencies(tasks: &HashMap<String, Task>) -> Result<Vec<String>, RunnerError> {
    crate::utils::topological_sort(tasks.keys().cloned(), |node| {
        tasks
            .get(node)
            .and_then(|t| t.depends_on.as_ref())
            .cloned()
            .unwrap_or_default()
    })
    .map_err(|err| match err {
        crate::utils::TopologicalSortError::DependencyCycle { cycle } => {
            RunnerError::DependencyCycle { cycle }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RunCommand;

    fn make_task(depends_on: Option<Vec<&str>>) -> Task {
        Task {
            run: Some(RunCommand::Single("echo test".to_string())),
            working_dir: None,
            depends_on: depends_on.map(|v| v.into_iter().map(String::from).collect()),
        }
    }

    #[test]
    fn test_resolve_dependencies_simple_dag() {
        let mut tasks = HashMap::new();
        tasks.insert("A".to_string(), make_task(None));
        tasks.insert("B".to_string(), make_task(Some(vec!["A"])));
        tasks.insert("C".to_string(), make_task(Some(vec!["B"])));

        let order = resolve_dependencies(&tasks).unwrap();
        assert_eq!(order, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_resolve_dependencies_multiple_roots() {
        let mut tasks = HashMap::new();
        tasks.insert("A".to_string(), make_task(None));
        tasks.insert("B".to_string(), make_task(None));
        tasks.insert("C".to_string(), make_task(Some(vec!["A", "B"])));

        let order = resolve_dependencies(&tasks).unwrap();
        let idx_a = order.iter().position(|x| x == "A").unwrap();
        let idx_b = order.iter().position(|x| x == "B").unwrap();
        let idx_c = order.iter().position(|x| x == "C").unwrap();
        assert!(idx_a < idx_c);
        assert!(idx_b < idx_c);
    }

    #[test]
    fn test_resolve_dependencies_cycle() {
        let mut tasks = HashMap::new();
        tasks.insert("A".to_string(), make_task(Some(vec!["B"])));
        tasks.insert("B".to_string(), make_task(Some(vec!["C"])));
        tasks.insert("C".to_string(), make_task(Some(vec!["A"])));

        let res = resolve_dependencies(&tasks);
        assert!(res.is_err());
        match res.unwrap_err() {
            RunnerError::DependencyCycle { cycle } => {
                assert_eq!(cycle.first(), cycle.last());
                assert_eq!(cycle.len(), 4);
            }
        }
    }
}
