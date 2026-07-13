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
    #[arg(short, long, default_value = "runner.config.toml")]
    pub config: PathBuf,

    /// Bypass the TUI interface and execute tasks directly in the CLI
    #[arg(long)]
    pub no_tui: bool,

    /// Output the JSON schema of the configuration file and exit
    #[arg(long, hide = true)]
    pub schema: bool,

    /// Specific task name(s) to execute. If omitted, all tasks will be run
    pub targets: Vec<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Commands {
    /// Output the JSON schema of the configuration file and exit
    Schema,
    /// Generate shell completion script to stdout
    Completions {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
}
