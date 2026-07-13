mod app;
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

    // 引数から設定ファイルのパスを取得、未指定なら "runner.config.toml"
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && (args[1] == "--schema" || args[1] == "schema") {
        let schema = schemars::schema_for!(AppConfig);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
        return Ok(());
    }

    let config_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from("runner.config.toml")
    };

    // 設定ファイルの読み込みとバリデーション
    let config = match AppConfig::load_from_file(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading configuration from {:?}: {}", config_path, e);
            std::process::exit(1);
        }
    };

    // tui=falseのときはtodo!を発生させる
    if !config.tui {
        todo!("Non-TUI mode is not implemented yet");
    }

    // タスクランナーの初期化と依存関係解決
    let runner = match TaskRunner::new(config.tasks) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
    };

    // ターミナル初期化
    let mut terminal = ratatui::init();

    // TUIアプリの構築と実行
    let mut app = App::new(runner);
    let app_result = app.run(&mut terminal);

    // ターミナルの復元
    ratatui::restore();

    if let Err(err) = app_result {
        eprintln!("TUI Application error: {:?}", err);
        std::process::exit(1);
    }

    Ok(())
}
