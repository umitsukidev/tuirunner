use crate::runner::{TaskRunner, TaskStatus};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::{io, time::Duration};

pub struct App {
    runner: TaskRunner,
    selected_task_index: usize,
    log_scroll_offset: u16,
    exit: bool,
    auto_scroll: bool,
}

impl App {
    pub fn new(runner: TaskRunner) -> Self {
        Self {
            runner,
            selected_task_index: 0,
            log_scroll_offset: 0,
            exit: false,
            auto_scroll: true,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn selected_task_name(&self) -> Option<String> {
        self.runner
            .execution_order
            .get(self.selected_task_index)
            .cloned()
    }

    fn draw(&mut self, frame: &mut Frame) {
        let size = frame.area();

        // 縦方向の分割 (メイン領域 & 下部ヘルプバー)
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(0), Constraint::Length(1)])
            .split(size);

        let body_area = main_layout[0];
        let help_area = main_layout[1];

        // 横方向の分割 (左サイドバー & 右ログエリア)
        let body_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(body_area);

        let sidebar_area = body_layout[0];
        let log_area = body_layout[1];

        // 状態取得
        let states_guard = self.runner.states.lock().unwrap();

        // --- Left sidebar: Task list ---
        let mut list_items = Vec::new();
        for (i, name) in self.runner.execution_order.iter().enumerate() {
            let state = states_guard.get(name).unwrap();
            let (status_icon, base_style) = match state.status {
                TaskStatus::Idle => ("  ·  ", Style::default().fg(Color::DarkGray)),
                TaskStatus::Pending => ("  ~  ", Style::default().fg(Color::Yellow)),
                TaskStatus::Running => (
                    "  ▶  ",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
                TaskStatus::Success => ("  ✔  ", Style::default().fg(Color::Green)),
                TaskStatus::Failed => ("  ✘  ", Style::default().fg(Color::Red)),
            };

            let is_selected = i == self.selected_task_index;
            let item_text = format!("{}{}", status_icon, name);
            let item_style = if is_selected {
                Style::default()
                    .bg(Color::Indexed(238)) // 暗い灰色
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                base_style
            };
            list_items.push(ListItem::new(item_text).style(item_style));
        }

        let sidebar_block = Block::default()
            .title(" Tasks ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let task_list = List::new(list_items).block(sidebar_block);
        frame.render_widget(task_list, sidebar_area);

        // --- 右ログエリア: タスク出力 ---
        if let Some(selected_name) = self.selected_task_name() {
            let state = states_guard.get(&selected_name).unwrap();
            let logs_guard = state.output.lock().unwrap();
            let logs_len = logs_guard.len();

            // ログの折り返しとスタイリング
            let text: Vec<Line> = logs_guard
                .iter()
                .map(|line| {
                    if line.starts_with("=== ") {
                        Line::from(line.clone().cyan().bold())
                    } else if line.starts_with("$ ") {
                        Line::from(line.clone().yellow())
                    } else if line.starts_with("[stderr] ") {
                        Line::from(line.clone().red())
                    } else if line.contains("failed") || line.contains("Failed") {
                        Line::from(line.clone().red().bold())
                    } else if line.contains("succeeded") || line.contains("Success") {
                        Line::from(line.clone().green().bold())
                    } else {
                        Line::from(line.clone())
                    }
                })
                .collect();

            // スコールの範囲制限
            let content_height = log_area.height.saturating_sub(2) as usize; // 境界線分を考慮
            let max_scroll = logs_len.saturating_sub(content_height) as u16;

            if self.auto_scroll {
                self.log_scroll_offset = max_scroll;
            } else if self.log_scroll_offset > max_scroll {
                self.log_scroll_offset = max_scroll;
            }

            let log_title = format!(" Output: {} ", selected_name);
            let log_block = Block::default()
                .title(log_title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));

            let log_paragraph = Paragraph::new(text)
                .block(log_block)
                .scroll((self.log_scroll_offset, 0));

            frame.render_widget(log_paragraph, log_area);
        } else {
            let empty_block = Block::default()
                .title(" Output ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            let empty_paragraph = Paragraph::new("No task selected.")
                .block(empty_block)
                .dark_gray();
            frame.render_widget(empty_paragraph, log_area);
        }

        // --- Bottom help bar ---
        let help_text = Line::from(vec![
            " Move: ".into(),
            "↑/↓/j/k".bold().cyan(),
            " | Run: ".into(),
            "r/Enter".bold().green(),
            " | Run All: ".into(),
            "a".bold().green(),
            " | Scroll Log: ".into(),
            "PgUp/PgDn/Shift+↑/Shift+↓".bold().cyan(),
            " | Clear: ".into(),
            "c (selected)/C (all)".bold().yellow(),
            " | Quit: ".into(),
            "q/Esc".bold().red(),
        ]);
        let help_paragraph = Paragraph::new(help_text);
        frame.render_widget(help_paragraph, help_area);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        // バックグラウンドでタスクが動いているため、TUIがフリーズしないようにタイムアウト付きでイベント待機します
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    self.handle_key_event(key_event);
                }
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.exit = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    // Shift + Up でログスクロール
                    self.log_scroll_offset = self.log_scroll_offset.saturating_sub(1);
                    self.auto_scroll = false;
                } else {
                    if self.selected_task_index > 0 {
                        self.selected_task_index -= 1;
                        self.log_scroll_offset = 0;
                        self.auto_scroll = true;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    // Shift + Down でログスクロール
                    self.log_scroll_offset = self.log_scroll_offset.saturating_add(1);
                    // auto_scroll を有効にするかどうかの判定 (最下部までスクロールした場合は有効にする)
                    if let Some(selected_name) = self.selected_task_name() {
                        let states_guard = self.runner.states.lock().unwrap();
                        if let Some(state) = states_guard.get(&selected_name) {
                            let logs_len = state.output.lock().unwrap().len();
                            // 画面高さ分を考慮してだいたい最下部に近い場合は自動スクロールを維持する
                            if (self.log_scroll_offset as usize) >= logs_len.saturating_sub(15) {
                                self.auto_scroll = true;
                            } else {
                                self.auto_scroll = false;
                            }
                        }
                    }
                } else {
                    if self.selected_task_index + 1 < self.runner.execution_order.len() {
                        self.selected_task_index += 1;
                        self.log_scroll_offset = 0;
                        self.auto_scroll = true;
                    }
                }
            }
            KeyCode::PageUp => {
                self.log_scroll_offset = self.log_scroll_offset.saturating_sub(10);
                self.auto_scroll = false;
            }
            KeyCode::PageDown => {
                self.log_scroll_offset = self.log_scroll_offset.saturating_add(10);
                self.auto_scroll = false;
            }
            KeyCode::Char('r') | KeyCode::Enter => {
                if let Some(name) = self.selected_task_name() {
                    self.runner.run_task(&name);
                    self.auto_scroll = true;
                }
            }
            KeyCode::Char('a') => {
                self.runner.run_all();
                self.auto_scroll = true;
            }
            KeyCode::Char('c') => {
                if let Some(name) = self.selected_task_name() {
                    self.runner.clear_logs(&name);
                    self.log_scroll_offset = 0;
                    self.auto_scroll = true;
                }
            }
            KeyCode::Char('C') => {
                self.runner.clear_all_logs();
                self.log_scroll_offset = 0;
                self.auto_scroll = true;
            }
            _ => {}
        }
    }
}
