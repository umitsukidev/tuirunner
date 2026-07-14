---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
    name: "tuirunner"
    text: "Ratatuiで構築されたTUIインターフェースを持つ並行タスクランナー。"
    tagline: 美しいターミナル画面で、タスクを並行して実行・監視できます。
    actions:
        - theme: brand
          text: ユーザーガイド
          link: /ja/guide
        - theme: alt
          text: GitHub
          link: https://github.com/umitsukidev/tuirunner

features:
    - title: インタラクティブなTUI
      details: ログのリアルタイム表示、オートスクロール、手動スクロール、カスタムキー操作に対応しています。
    - title: 並行タスク実行
      details: 依存関係のない複数のタスクを、別スレッドで自動的に並行実行します。
    - title: 実行グラフの視覚化
      details: タスクの依存関係DAG（有向非巡回グラフ）と現在の実行ステータスをリアルタイムに描画します。
---
