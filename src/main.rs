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
    cli::{Cli, Commands},
    config::AppConfig,
    runner::TaskRunner,
};
use clap::{CommandFactory, Parser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // エラーハンドリングの初期化
    color_eyre::install()?;

    let cli = Cli::parse();

    // スキーマの出力、または補完の出力
    if cli.schema || matches!(cli.command, Some(Commands::Schema)) {
        let schema = schemars::schema_for!(AppConfig);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
        return Ok(());
    }

    if let Some(Commands::Completions { shell }) = cli.command {
        let mut cmd = Cli::command();
        let bin_name = cmd.get_name().to_string();
        clap_complete::generate(shell, &mut cmd, bin_name, &mut std::io::stdout());
        return Ok(());
    }

    let config_path = cli.config;
    let no_tui = cli.no_tui;
    let targets = cli.targets;

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

    if !use_tui {
        // 非TUIモード実行
        if let Err(e) = non_tui::run_non_tui(&runner, &targets) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
        return Ok(());
    }

    // ターミナル初期化
    let mut terminal = ratatui::init();

    // TUIアプリの構築と実行
    let mut app = App::new(runner, targets);
    let app_result = app.run(&mut terminal);

    // ターミナルの復元
    ratatui::restore();

    if let Err(err) = app_result {
        eprintln!("TUI Application error: {:?}", err);
        std::process::exit(1);
    }

    Ok(())
}
