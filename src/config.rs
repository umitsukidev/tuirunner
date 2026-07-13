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
    /// The command(s) to run (hidden command execution)
    #[serde(default)]
    #[garde(skip)]
    #[schemars(with = "RunCommand")]
    pub run: Option<RunCommand>,
    /// The command(s) to run (dimmed command execution printed to console)
    #[serde(default)]
    #[garde(skip)]
    #[schemars(with = "RunCommand")]
    pub cmd: Option<RunCommand>,
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
        if task.run.is_some() && task.cmd.is_some() {
            return Err(garde::Error::new(format!(
                "Task '{}' cannot have both 'run' and 'cmd' specified. They are mutually exclusive.",
                task_name
            )));
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_mutual_exclusion() {
        let mut tasks = HashMap::new();
        tasks.insert(
            "invalid_task".to_string(),
            Task {
                run: Some(RunCommand::Single("echo run".to_string())),
                cmd: Some(RunCommand::Single("echo cmd".to_string())),
                working_dir: None,
                depends_on: None,
            },
        );
        let config = TasksConfig { tasks };
        let res = validate_tasks_config(&config, &());
        assert!(res.is_err());
        assert!(
            res.unwrap_err()
                .to_string()
                .contains("cannot have both 'run' and 'cmd'")
        );
    }
}
