use garde::Validate;
use schemars::JsonSchema;
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

/// The basic application configuration
#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct AppConfig {
    /// Whether to enable the TUI (Terminal User Interface) mode
    #[serde(default)]
    #[garde(skip)]
    pub tui: bool,
    /// The map of tasks to define and run
    #[serde(default)]
    #[garde(dive)]
    pub tasks: TasksConfig,
}

/// A map container for tasks
#[derive(Debug, Clone, Default, Deserialize, Validate, JsonSchema)]
#[garde(custom(validate_tasks_config))]
#[schemars(transparent)]
pub struct TasksConfig {
    /// The dictionary of task names to their configurations
    #[serde(flatten)]
    #[garde(dive)]
    pub tasks: HashMap<String, Task>,
}

/// A command to run, which can be a single string or an array of strings
#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum RunCommand {
    /// Runs a single command string
    Single(String),
    /// Runs multiple command strings in sequence
    Multiple(Vec<String>),
}

/// Configuration details for an individual task
#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct Task {
    /// The command(s) to run
    #[serde(default)]
    #[garde(skip)]
    #[schemars(with = "RunCommand")]
    pub run: Option<RunCommand>,
    /// The working directory in which the command(s) should be executed
    #[serde(default)]
    #[garde(skip)]
    #[schemars(with = "PathBuf")]
    pub working_dir: Option<PathBuf>,
    /// A list of task names that this task depends on and must run after
    #[serde(default)]
    #[garde(skip)]
    #[schemars(with = "Vec<String>")]
    pub depends_on: Option<Vec<String>>,
}

impl AppConfig {
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        let config: AppConfig = match extension {
            "json" => serde_json::from_str(&content)?,
            "yaml" | "yml" => serde_yaml::from_str(&content)?,
            "toml" => toml::from_str(&content)?,
            _ => toml::from_str(&content)?, // デフォルトはTOMLとしてパース試行
        };

        config.validate()?;

        Ok(config)
    }
}

fn validate_tasks_config(tasks_config: &TasksConfig, _ctx: &()) -> garde::Result {
    for (task_name, task) in &tasks_config.tasks {
        if let Some(depends_on) = &task.depends_on {
            for dep in depends_on {
                if !tasks_config.tasks.contains_key(dep) {
                    return Err(garde::Error::new(format!(
                        "Task '{}' depends on non-existent task '{}'",
                        task_name, dep
                    )));
                }
            }
        }
    }
    Ok(())
}
