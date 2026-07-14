use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigFormat {
    Toml,
    Yaml,
    Json,
}

#[derive(Parser, Debug)]
#[command(
    name = "tuir",
    version,
    about = "A concurrent task runner with TUI and CLI",
    long_about = None
)]
pub struct Cli {
    /// Path to the configuration file
    #[arg(short, long, default_value = "runner.config.toml", global = true)]
    pub config: PathBuf,

    /// Bypass the TUI interface and execute tasks directly in the CLI
    #[arg(long, global = true)]
    pub no_tui: bool,

    /// Output the JSON schema of the configuration file and exit
    #[arg(long, hide = true, global = true)]
    pub schema: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Commands {
    /// Initialize a new configuration file
    Init {
        /// Format of the configuration file (toml, yaml, json)
        #[arg(short, long, value_enum, conflicts_with_all = &["toml", "yaml", "json"])]
        format: Option<ConfigFormat>,

        /// Initialize as TOML format
        #[arg(long, conflicts_with_all = &["yaml", "json", "format"])]
        toml: bool,

        /// Initialize as YAML format
        #[arg(long, conflicts_with_all = &["toml", "json", "format"])]
        yaml: bool,

        /// Initialize as JSON format
        #[arg(long, conflicts_with_all = &["toml", "yaml", "format"])]
        json: bool,
    },
    /// Run tasks
    Run {
        /// Specific task name(s) to execute. If omitted, all tasks will be run
        targets: Vec<String>,
    },
    /// Output the JSON schema of the configuration file and exit
    Schema,
    /// Generate shell completion script to stdout
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_init_default() {
        let args = vec!["tuir", "init"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(
            cli.command,
            Some(Commands::Init {
                format: None,
                toml: false,
                yaml: false,
                json: false,
            })
        );
    }

    #[test]
    fn test_cli_init_format_yaml() {
        let args = vec!["tuir", "init", "--format", "yaml"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(
            cli.command,
            Some(Commands::Init {
                format: Some(ConfigFormat::Yaml),
                toml: false,
                yaml: false,
                json: false,
            })
        );
    }

    #[test]
    fn test_cli_init_flag_yaml() {
        let args = vec!["tuir", "init", "--yaml"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(
            cli.command,
            Some(Commands::Init {
                format: None,
                toml: false,
                yaml: true,
                json: false,
            })
        );
    }

    #[test]
    fn test_cli_init_conflicts() {
        // format and flags conflict
        let args = vec!["tuir", "init", "--format", "yaml", "--yaml"];
        let res = Cli::try_parse_from(args);
        assert!(res.is_err());

        // flags conflict with each other
        let args2 = vec!["tuir", "init", "--yaml", "--json"];
        let res2 = Cli::try_parse_from(args2);
        assert!(res2.is_err());
    }
}
