mod app;
mod cli;
mod components;
mod config;
mod runner;
mod store;
mod utils;

use app::App;
use config::AppConfig;
use runner::TaskRunner;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // エラーハンドリングの初期化
    color_eyre::install()?;

    let args: Vec<String> = std::env::args().collect();

    // スキーマの早期出力
    if args.len() > 1 && (args[1] == "--schema" || args[1] == "schema") {
        let schema = schemars::schema_for!(AppConfig);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
        return Ok(());
    }

    let mut config_path = PathBuf::from("runner.config.toml");
    let mut no_tui = false;
    let mut targets = Vec::new();

    let mut skip = false;
    for i in 1..args.len() {
        if skip {
            skip = false;
            continue;
        }
        let arg = &args[i];
        if arg == "--config" || arg == "-c" {
            if i + 1 < args.len() {
                config_path = PathBuf::from(&args[i + 1]);
                skip = true;
            } else {
                eprintln!("Error: Missing value for --config flag");
                std::process::exit(1);
            }
        } else if arg.starts_with("--config=") {
            config_path = PathBuf::from(arg.trim_start_matches("--config="));
        } else if arg == "--no-tui" {
            no_tui = true;
        } else if arg.starts_with("-") {
            eprintln!("Error: Unknown flag '{}'", arg);
            std::process::exit(1);
        } else {
            targets.push(arg.clone());
        }
    }

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
        if let Err(e) = cli::run_non_tui(&runner, &targets) {
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
