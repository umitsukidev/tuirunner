use crate::runner::log_buffer::LogBuffer;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Idle,
    Pending,
    Running,
    Success,
    Failed,
}

#[derive(Debug, Clone)]
pub struct TaskState {
    pub status: TaskStatus,
    pub output: Arc<Mutex<LogBuffer>>,
    pub child_pid: Option<u32>,
    pub stopped_as_success: bool,
    pub stdin_tx: Option<tokio::sync::mpsc::UnboundedSender<Vec<u8>>>,
}

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
