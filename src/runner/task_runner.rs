use crate::{
    config::{Task, TasksConfig},
    runner::{
        cmd,
        types::{RunnerError, TaskState, TaskStatus},
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};
use tokio::{sync::Notify, task::JoinHandle};

#[derive(Debug)]
pub struct TaskRunner {
    pub tasks: HashMap<String, Task>,
    pub execution_order: Vec<String>,
    pub states: Arc<Mutex<HashMap<String, TaskState>>>,
    pub notify: Arc<Notify>,
    pub is_tui: bool,
}

impl TaskRunner {
    pub fn new(tasks_config: TasksConfig, is_tui: bool) -> Result<Self, RunnerError> {
        let tasks = tasks_config.tasks;
        let execution_order = resolve_dependencies(&tasks)?;

        let mut states_map = HashMap::new();
        for name in tasks.keys() {
            states_map.insert(
                name.clone(),
                TaskState {
                    status: TaskStatus::Idle,
                    output: Arc::new(Mutex::new(Vec::new())),
                },
            );
        }

        Ok(Self {
            tasks,
            execution_order,
            states: Arc::new(Mutex::new(states_map)),
            notify: Arc::new(Notify::new()),
            is_tui,
        })
    }

    pub fn get_subgraph(&self, targets: &[String]) -> HashSet<String> {
        let mut subgraph = HashSet::new();
        for target in targets {
            get_subgraph(&self.tasks, target, &mut subgraph);
        }
        subgraph
    }

    #[allow(dead_code)]
    pub fn execution_order(&self) -> &[String] {
        &self.execution_order
    }

    pub fn run_task(&self, target_name: &str) -> JoinHandle<()> {
        let mut target_subgraph = HashSet::new();
        get_subgraph(&self.tasks, target_name, &mut target_subgraph);
        self.spawn_scheduler(target_subgraph)
    }

    pub fn run_tasks(&self, targets: &[String]) -> JoinHandle<()> {
        let mut target_subgraph = HashSet::new();
        for target in targets {
            get_subgraph(&self.tasks, target, &mut target_subgraph);
        }
        self.spawn_scheduler(target_subgraph)
    }

    pub fn run_all(&self) -> JoinHandle<()> {
        let target_subgraph: HashSet<String> = self.tasks.keys().cloned().collect();
        self.spawn_scheduler(target_subgraph)
    }

    pub fn clear_logs(&self, task_name: &str) {
        let states = self.states.lock().unwrap();
        if let Some(state) = states.get(task_name) {
            state.output.lock().unwrap().clear();
        }
    }

    pub fn clear_all_logs(&self) {
        let states = self.states.lock().unwrap();
        for state in states.values() {
            state.output.lock().unwrap().clear();
        }
    }

    fn spawn_scheduler(&self, target_subgraph: HashSet<String>) -> JoinHandle<()> {
        // Reset state of target tasks to Pending and prepare their output buffers
        {
            let mut states_guard = self.states.lock().unwrap();
            for name in &target_subgraph {
                if let Some(state) = states_guard.get_mut(name) {
                    state.status = TaskStatus::Pending;
                    let mut out = state.output.lock().unwrap();
                    out.clear();
                    out.push(format!("=== Task queued: {} ===", name));
                }
            }
        }

        let states = Arc::clone(&self.states);
        let notify = Arc::clone(&self.notify);
        let tasks = self.tasks.clone();
        let is_tui = self.is_tui;
        let execution_order = self.execution_order.clone();

        tokio::spawn(async move {
            loop {
                let mut has_running = false;
                let mut has_pending = false;

                {
                    let mut states_guard = states.lock().unwrap();
                    let mut to_start = Vec::new();

                    for name in &target_subgraph {
                        let state = states_guard.get(name).unwrap();
                        match state.status {
                            TaskStatus::Running => {
                                has_running = true;
                            }
                            TaskStatus::Pending => {
                                has_pending = true;
                                let task_config = tasks.get(name).unwrap();
                                let deps = task_config.depends_on.as_ref();

                                let all_deps_success = match deps {
                                    None => true,
                                    Some(dep_list) => dep_list.iter().all(|dep| {
                                        states_guard
                                            .get(dep)
                                            .map(|s| s.status == TaskStatus::Success)
                                            .unwrap_or(false)
                                    }),
                                };

                                let any_dep_failed = match deps {
                                    None => false,
                                    Some(dep_list) => dep_list.iter().any(|dep| {
                                        states_guard
                                            .get(dep)
                                            .map(|s| s.status == TaskStatus::Failed)
                                            .unwrap_or(false)
                                    }),
                                };

                                if all_deps_success {
                                    to_start.push(name.clone());
                                } else if any_dep_failed {
                                    let state_mut = states_guard.get_mut(name).unwrap();
                                    state_mut.status = TaskStatus::Failed;
                                    let mut out = state_mut.output.lock().unwrap();
                                    out.push(
                                        "Dependency task failed. Skipping execution.".to_string(),
                                    );
                                }
                            }
                            _ => {}
                        }
                    }

                    // Start tasks that are ready to run
                    for name in to_start {
                        let state_mut = states_guard.get_mut(&name).unwrap();
                        state_mut.status = TaskStatus::Running;

                        let output_buf = Arc::clone(&state_mut.output);
                        let name_clone = name.clone();

                        let prefix = if !is_tui {
                            let colors = [
                                crossterm::style::Color::Blue,
                                crossterm::style::Color::Green,
                                crossterm::style::Color::Yellow,
                                crossterm::style::Color::Magenta,
                                crossterm::style::Color::Cyan,
                                crossterm::style::Color::Red,
                            ];
                            let task_idx = execution_order
                                .iter()
                                .position(|n| n == &name_clone)
                                .unwrap_or(0);
                            let color = colors[task_idx % colors.len()];
                            use crossterm::style::Stylize;
                            Some(format!(
                                "{}",
                                format!("[{}]", name_clone).with(color).bold()
                            ))
                        } else {
                            None
                        };

                        {
                            let mut buf = output_buf.lock().unwrap();
                            buf.clear();
                            buf.push(format!("=== Starting task: {} ===", name));
                        }

                        let task_config = tasks.get(&name).unwrap().clone();
                        let states_worker = Arc::clone(&states);
                        let notify_worker = Arc::clone(&notify);
                        let prefix_worker = prefix.clone();

                        tokio::spawn(async move {
                            let result = cmd::execute_command_capturing(
                                &task_config,
                                &output_buf,
                                &prefix_worker,
                            )
                            .await;

                            let mut guard = states_worker.lock().unwrap();
                            let s = guard.get_mut(&name_clone).unwrap();
                            s.status = if result.is_ok() {
                                TaskStatus::Success
                            } else {
                                TaskStatus::Failed
                            };
                            {
                                let mut buf = output_buf.lock().unwrap();
                                match result {
                                    Ok(_) => {
                                        buf.push(format!("=== Task succeeded: {} ===", name_clone))
                                    }
                                    Err(e) => buf.push(format!(
                                        "=== Task failed: {}: {} ===",
                                        name_clone, e
                                    )),
                                }
                            }
                            notify_worker.notify_one();
                        });
                        has_running = true;
                    }
                }

                if !has_running && !has_pending {
                    break;
                }

                notify.notified().await;
            }
        })
    }
}

fn get_subgraph(tasks: &HashMap<String, Task>, target: &str, subgraph: &mut HashSet<String>) {
    if subgraph.insert(target.to_string()) {
        if let Some(task) = tasks.get(target) {
            if let Some(deps) = &task.depends_on {
                for dep in deps {
                    get_subgraph(tasks, dep, subgraph);
                }
            }
        }
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
            cmd: None,
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
