use clap::{Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;

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
