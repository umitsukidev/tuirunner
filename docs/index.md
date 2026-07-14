---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
    name: "tuirunner"
    text: "A concurrent task runner with a TUI interface built on Ratatui."
    tagline: Run and monitor tasks concurrently with a beautiful terminal interface.
    actions:
        - theme: brand
          text: User Guide
          link: /guide
        - theme: alt
          text: GitHub
          link: https://github.com/umitsukidev/tuirunner

features:
    - title: Interactive TUI
      details: Live log viewer with auto-scrolling, manual scrolling, and custom controls.
    - title: Concurrent Tasks
      details: Automatically runs non-dependent tasks concurrently in separate threads.
    - title: Visual Execution Graph
      details: Displays a live representation of the task execution dependency DAG (Directed Acyclic Graph).
---
