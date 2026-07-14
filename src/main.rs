mod app;
mod cli;
mod components;
mod config;
mod non_tui;
mod runner;
mod store;
mod utils;

use crate::{
    app::App,
    cli::{Cli, Commands, ConfigFormat},
    config::AppConfig,
    runner::TaskRunner,
};
use clap::{CommandFactory, Parser};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // エラーハンドリングの初期化
    color_eyre::install()?;

    let cli = Cli::parse();

    // スキーマの出力（グローバルフラグ用）
    if cli.schema {
        let schema = schemars::schema_for!(AppConfig);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
        return Ok(());
    }

    // サブコマンドの処理およびtargetsの抽出
    let targets = match cli.command {
        Some(Commands::Init {
            format,
            toml,
            yaml,
            json,
        }) => {
            let final_format = if toml {
                ConfigFormat::Toml
            } else if yaml {
                ConfigFormat::Yaml
            } else if json {
                ConfigFormat::Json
            } else {
                format.unwrap_or(ConfigFormat::Toml)
            };

            // Check if runner.config.{toml,yaml,yml,json} already exists.
            let files = [
                "runner.config.toml",
                "runner.config.yaml",
                "runner.config.yml",
                "runner.config.json",
            ];
            for file in &files {
                let path = std::path::Path::new(file);
                if path.exists() {
                    eprintln!("Error: {} already exists", file);
                    std::process::exit(1);
                }
            }

            // Create the configuration file based on format
            let (filename, content) = match final_format {
                ConfigFormat::Toml => (
                    "runner.config.toml",
                    r#"#:schema https://tuir.umitsuki.dev/runner.schema.json

tui = true

[tasks.hello]
description = "Say hello"
cmd = "echo 'Hello, World!'"
"#,
                ),
                ConfigFormat::Yaml => (
                    "runner.config.yaml",
                    r#"# yaml-language-server: $schema=https://tuir.umitsuki.dev/runner.schema.json

tui: true
tasks:
  hello:
    description: "Say hello"
    cmd: "echo 'Hello, World!'"
"#,
                ),
                ConfigFormat::Json => (
                    "runner.config.json",
                    r#"{
  "$schema": "https://tuir.umitsuki.dev/runner.schema.json",
  "tui": true,
  "tasks": {
    "hello": {
      "description": "Say hello",
      "cmd": "echo 'Hello, World!'"
    }
  }
}
"#,
                ),
            };

            std::fs::write(filename, content)?;
            println!("Initialized {}", filename);
            return Ok(());
        }
        Some(Commands::Run { targets }) => targets,
        Some(Commands::External(args)) => {
            if args.len() > 1 {
                eprintln!(
                    "Error: Multiple tasks are not allowed when executing a task directly. Use 'run' subcommand for multiple tasks."
                );
                std::process::exit(1);
            }
            if args.is_empty() {
                let mut cmd = Cli::command();
                cmd.print_help()?;
                println!();
                return Ok(());
            }
            let task_name = args[0].clone();
            vec![task_name]
        }
        Some(Commands::Schema) => {
            let schema = schemars::schema_for!(AppConfig);
            println!("{}", serde_json::to_string_pretty(&schema).unwrap());
            return Ok(());
        }
        Some(Commands::Completions { shell }) => {
            let mut cmd = Cli::command();
            let bin_name = cmd.get_name().to_string();
            clap_complete::generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
            return Ok(());
        }
        None => {
            let mut cmd = Cli::command();
            cmd.print_help()?;
            println!();
            return Ok(());
        }
    };

    let config_path = cli.config;
    let no_tui = cli.no_tui;

    // 設定ファイルの読み込みとバリデーション
    let config = match AppConfig::load_from_file(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading configuration from {:?}: {}", config_path, e);
            std::process::exit(1);
        }
    };

    let use_tui = config.tui && !no_tui;

    // タスクランナーの初期化と依存関係解決
    let runner = match TaskRunner::new(config.tasks, use_tui) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
    };

    // Validate target task names exist if targets are specified
    for target in &targets {
        if !runner.tasks.contains_key(target) {
            eprintln!("Error: Task '{}' not found in configuration", target);
            std::process::exit(1);
        }
    }

    if !use_tui {
        // 非TUIモード実行
        if let Err(e) = non_tui::run_non_tui(&runner, &targets).await {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        return Ok(());
    }

    // ターミナル初期化
    let mut terminal = ratatui::init();

    // TUIアプリの構築と実行
    let mut app = App::new(runner, targets);
    let app_result = app.run(&mut terminal).await;

    // ターミナルの復元
    ratatui::restore();

    if let Err(err) = app_result {
        eprintln!("TUI Application error: {:?}", err);
        std::process::exit(1);
    }

    Ok(())
}
