use garde::Validate;
use schemars::JsonSchema;
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct AppConfig {
    #[serde(default)]
    #[garde(skip)]
    pub tui: bool,
    #[serde(default)]
    #[garde(dive)]
    pub tasks: TasksConfig,
}

#[derive(Debug, Clone, Default, Deserialize, Validate, JsonSchema)]
#[garde(custom(validate_tasks_config))]
#[schemars(transparent)]
pub struct TasksConfig {
    #[serde(flatten)]
    #[garde(dive)]
    pub tasks: HashMap<String, Task>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum RunCommand {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, Deserialize, Validate, JsonSchema)]
pub struct Task {
    #[serde(default)]
    #[garde(skip)]
    pub run: Option<RunCommand>,
    #[garde(skip)]
    pub working_dir: Option<PathBuf>,
    #[garde(skip)]
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
