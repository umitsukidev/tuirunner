# ユーザーガイド

このガイドでは、`tuirunner` (`tuir`) のインストール、設定、および使用方法について説明します。

---

## インストール

`tuirunner` をインストールするには、以下のコマンドを実行します：

```bash
cargo install tuirunner
```

これにより、`tuir` バイナリがコンパイルされ、Cargoのバイナリディレクトリ（例: `~/.cargo/bin`）にインストールされます。このディレクトリがシステムの `PATH` に含まれていることを確認してください。

---

## 使い方

デフォルトでは、`tuir` は現在の作業ディレクトリにある `runner.config.toml` という名前の設定ファイルを探します。

```bash
# ヘルプメッセージを表示
tuir

# すべてのタスクを実行（デフォルトの設定に従ってTUIまたはCLIモードで起動します）
tuir run

# 特定のタスクとその依存タスクを実行
tuir run build test

# TUIモードを使用せずに実行（CLIモード）
tuir --no-tui run

# カスタム設定ファイルのパスを指定して実行（JSON、YAML、またはTOML）
tuir --config custom-config.yaml run
tuir -c custom-config.json run
```

### CLIサブコマンド

- **`run`**: タスクを実行します。
    - `[TARGETS]...`: 実行する特定のタスク名。指定しない場合、設定ファイルに定義されているすべてのタスクが実行されます。
- **`schema`**: 設定ファイルのJSONスキーマを出力して終了します。エディタの入力補完やバリデーションの連携に便利です。
- **`completions <shell>`**: 指定したシェルの補完スクリプトを標準出力に出力します。

### グローバルオプション

- **`-c, --config <path>`**: 設定ファイルのパスを指定します（デフォルト: `runner.config.toml`）。
- **`--no-tui`**: TUIインターフェースをバイパスし、CLIモードで直接タスクを実行します。

---

## 設定

`tuirunner` は、**TOML**、**YAML**、**JSON** 形式の設定ファイルをサポートしています。

### TOML設定例 (`runner.config.toml`)

```toml
#:schema ./runner.schema.json

tui = true

[tasks.clean]
description = "ビルドディレクトリのクリーンアップ"
run = "echo 'Cleaning build directory...'; sleep 1; echo 'Cleaned!'"

[tasks.codegen]
description = "ソースコードの自動生成"
run = "echo 'Running codegen...'; sleep 1.5; echo 'Codegen done!'"

[tasks.build]
description = "アプリケーションのビルド"
run = "echo 'Building...'; sleep 2"
depends_on = ["clean", "codegen"]

[tasks.test]
description = "テストスイートの実行"
cmd = "echo 'Running unit tests...'; cargo test" # コマンド自体の実行内容もログやコンソールに出力します
depends_on = ["build"]

[tasks.deploy]
description = "アプリケーションのデプロイ"
cmd = [
    "echo 'Deploying release...'",
    "sleep 1",
    "echo 'Deployed!'"
]
depends_on = ["build"]
working_dir = "./deploy_script"
```

### 設定フィールド

- **`tui`** (boolean, デフォルト: `true`): デフォルトでTUIモードを有効にするかどうかを指定します。
- **`tasks`** (オブジェクト / マップ): タスクの定義。キーがタスク名になります。
    - **`description`** (string, 任意): タスクの概要説明。TUIのタスク一覧に表示されます。
    - **`run`** (string または string の配列): 出力を抑えて実行するコマンド。コマンド文字列は `sh -c` 経由で実行されます（`cmd` とは排他）。
    - **`cmd`** (string または string の配列): 実行するコマンド。`run` と同様ですが、実行するコマンド文そのものもログ/コンソールに出力します（`run` とは排他）。
    - **`working_dir`** (string, 任意): コマンドが実行される作業ディレクトリ。
    - **`depends_on`** (string の配列, 任意): このタスクを開始する前に、正常に終了している必要がある依存先タスク名の一覧。

### JSONスキーマの連携

エディタでの設定ファイルの自動補完や構文検証を有効にするために、スキーマファイルを生成することができます：

```bash
tuir schema > runner.schema.json
```

生成後、設定ファイルの先頭で以下のようにスキーマを参照します：

- **TOML**: `#:schema ./runner.schema.json`
- **JSON**: ルートに `"$schema": "./runner.schema.json"` を追加します。
- **YAML**: エディタ固有のコメントやワークスペース設定を使用してスキーマをマッピングします。

---

## TUIのキーバインド一覧

TUIモードでの実行中は、以下のキー操作でアプリケーションを操作できます：

| キー                      | アクション                                         |
| ------------------------- | -------------------------------------------------- |
| `↑` / `↓` / `j` / `k`     | タスク一覧の選択位置を移動                         |
| `r` / `Enter`             | 選択されているタスク（およびその依存タスク）を実行 |
| `A`                       | 一覧にあるすべてのタスクを実行                     |
| `c`                       | 選択されているタスクの出力ログをクリア             |
| `C` (Shift+C)             | すべてのタスクの出力ログをクリア                   |
| `PgUp` / `PgDn`           | ログを半ページ分アップ/ダウン                      |
| `Shift + ↑` / `Shift + ↓` | ログを1行ずつアップ/ダウン                         |
| `q` / `Esc`               | アプリケーションを終了                             |
