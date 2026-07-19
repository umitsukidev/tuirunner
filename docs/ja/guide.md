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

## クイックスタート

プロジェクト用ディレクトリを作成し、設定を生成してから、定義済みのすべてのタスクを実行します。

```bash
mkdir my-project && cd my-project
tuir init
tuir run
```

`tuir init` は既存の `runner.config.toml`、`runner.config.yaml`、`runner.config.yml`、`runner.config.json` を上書きしません。`--format toml`、`--format yaml`、`--format json` で形式を選択できます。

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

# 特定のタスクを直接実行（`tuir run <task-name>` のショートカット）
# ※この直接実行モードでは、複数のタスクを同時に指定することはできません。
tuir build

# TUIモードを使用せずに実行（CLIモード）
tuir --no-tui run

# カスタム設定ファイルのパスを指定して実行（JSON、YAML、またはTOML）
tuir --config custom-config.yaml run
tuir -c custom-config.json run

# 新しい設定ファイルを初期化して生成
tuir init
```

### CLIサブコマンド

- **`init`**: 現在の作業ディレクトリに新しい設定ファイルを生成して初期化します（デフォルト形式: TOML）。
    - `--format <format>`: 設定ファイルのフォーマットを指定（toml、yaml、json）。
    - `--toml`, `--yaml`, `--json`: 生成する設定ファイルのフォーマットを指定するフラグ。
- **`run`**: タスクを実行します。
    - `[TARGETS]...`: 実行する特定のタスク名。指定しない場合、設定ファイルに定義されているすべてのタスクが実行されます。
- **`schema`**: 設定ファイルのJSONスキーマを出力して終了します。エディタの入力補完やバリデーションの連携に便利です。
- **`completions <shell>`**: 指定したシェルの補完スクリプトを標準出力に出力します。

### グローバルオプション

- **`-c, --config <path>`**: 設定ファイルのパスを指定します（デフォルト: `runner.config.toml`）。
- **`--no-tui`**: TUIインターフェースをバイパスし、CLIモードで直接タスクを実行します。

`--no-tui` で実行した場合、指定したタスクのいずれかが失敗すると `tuir` は非ゼロの終了ステータスを返します。同じ設定をシェルスクリプトや CI ジョブでも利用できます。

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
    - **`description`** (string, 任意): タスクの概要説明。TUI で選択中のタスクの出力ペインに表示されます。
    - **`run`** (string または string の配列): 出力を抑えて実行するコマンド。コマンド文字列は `sh -c` 経由で実行されます（`cmd` とは排他）。
    - **`cmd`** (string または string の配列): 実行するコマンド。`run` と同様ですが、実行するコマンド文そのものもログ/コンソールに出力します（`run` とは排他）。
    - **`working_dir`** (string, 任意): コマンドが実行される作業ディレクトリ。
    - **`depends_on`** (string の配列, 任意): このタスクを開始する前に、正常に終了している必要がある依存先タスク名の一覧。

タスク名は重複できず、`run` という名前にはできません。`depends_on` の各要素は設定済みタスクを参照する必要があり、`run` と `cmd` の同時指定はできません。

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

| キー                      | アクション                                                       |
| ------------------------- | ---------------------------------------------------------------- |
| `↑` / `↓` / `j` / `k`     | タスク一覧の選択位置を移動                                       |
| `r` / `Enter`             | 選択されているタスク（およびその依存タスク）を実行               |
| `A`                       | 一覧にあるすべてのタスクを実行                                   |
| `s`                       | 選択されているタスクを停止（依存関係のある後続タスクはスキップ） |
| `S` (Shift+S)             | 選択されているタスクを停止し、後続タスクは継続して実行           |
| `c`                       | 選択されているタスクの出力ログをクリア                           |
| `C` (Shift+C)             | すべてのタスクの出力ログをクリア                                 |
| `PgUp` / `PgDn`           | ログを10行ずつアップ/ダウン                                      |
| `Shift + ↑` / `Shift + ↓` | ログを1行ずつアップ/ダウン                                       |
| `i`                       | 選択されている実行中タスクの「インタラクティブ入力モード」を開始 |
| `Esc` (対話入力モード中)  | インタラクティブ入力モードを終了し、通常のTUI制御モードに復帰    |
| `q` / `Esc`               | アプリケーションを終了                                           |

---

## 既知の制限事項と制約

### プラットフォームの互換性（Windowsネイティブ環境での動作）

現在、`tuirunner` はタスクの起動や強制終了（`s` / `S` キーやアプリ終了時のクリーンアップ）において、Unix系シェル（`sh -c`）および POSIX シグナル処理（プロセスグループ ID に対する `libc::kill`）に依存しています。

- **Windows環境での制限**: Windows のネイティブな cmd.exe や PowerShell 環境から `tuir` を直接実行した場合、コマンドが正常に動作しなかったり、プロセスを停止した際に子プロセスがゾンビプロセスとして残ってしまう可能性があります。
- **推奨事項**: Windowsでご利用の際は、**WSL (Windows Subsystem for Linux)** または **Git Bash** などの Unix 互換環境を介して実行することを強く推奨します。

### インタラクティブ入力モードの制限（非PTY）

TUI上で `i` キーを押すことで起動する「インタラクティブ入力モード」は、入力されたキーコードを単に実行プロセスの標準入力（stdin）ストリームに流し込むことで動作しています。

- この方式は仮想端末（PTY）を確保するものではないため、`vim` や `nano` などのフルスクリーン・テキストエディタ、高度なインタラクティブシェル、プロンプトの表示位置を動的に制御するツールなどは正常にレンダリングされない、あるいは実行を拒否される場合があります。

### ログの保持

ログは現在の TUI セッション中だけメモリに保持されます。`tuir` を終了すると失われるため、ファイルに残す必要がある場合はコマンド側で標準出力・標準エラー出力をリダイレクトしてください。

---

## 将来のロードマップ

今後のリリースにおいて、以下の機能の実装・導入を検討・計画しています：

1. **Windowsネイティブ対応**: `cmd.exe /C` や PowerShell を用いたコマンドの起動、およびWindowsネイティブなプロセスツリー強制終了処理の実装。
2. **ログのファイル出力・アーカイブ機能**: タスクの実行ログをメモリ上だけでなく、ローカルファイルに永続保存するためのオプション（`--log-dir` などの引数）の追加。
3. **PTY（疑似ターミナル）対応の検討**: `portable-pty` などの外部ライブラリを組み込み、TUI内でも複雑な対話型CUIツールやエディタを完全な描画と入力追従で動作させられるような仕組みの検討。
