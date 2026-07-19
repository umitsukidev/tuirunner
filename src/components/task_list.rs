use crate::{runner::TaskStatus, store::Store};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Widget},
};

pub enum TaskListEvent {
    Select(usize),
    Run(String),
    Clear(String),
    Stop(String),
    StopAndNext(String),
}

pub struct TaskList<'a> {
    pub store: &'a Store<'a>,
    pub selected_index: usize,
}

impl TaskList<'_> {
    pub fn handle_key_event(
        key: KeyEvent,
        current_index: usize,
        visible_tasks: &[String],
    ) -> Option<TaskListEvent> {
        let has_shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let order = visible_tasks;
        let mut idx = current_index;

        match key.code {
            KeyCode::Up | KeyCode::Char('k') if !has_shift => {
                idx = idx.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') if !has_shift => {
                if idx + 1 < order.len() {
                    idx += 1;
                }
            }
            KeyCode::Char('r') | KeyCode::Enter if !has_shift => {
                if let Some(name) = order.get(idx) {
                    return Some(TaskListEvent::Run(name.clone()));
                }
            }
            KeyCode::Char('c') if !has_shift => {
                if let Some(name) = order.get(idx) {
                    return Some(TaskListEvent::Clear(name.clone()));
                }
            }
            KeyCode::Char('s') if !has_shift => {
                if let Some(name) = order.get(idx) {
                    return Some(TaskListEvent::Stop(name.clone()));
                }
            }
            KeyCode::Char('S') | KeyCode::Char('s') if has_shift => {
                if let Some(name) = order.get(idx) {
                    return Some(TaskListEvent::StopAndNext(name.clone()));
                }
            }
            _ => {}
        }

        if idx != current_index {
            Some(TaskListEvent::Select(idx))
        } else {
            None
        }
    }
}

impl Widget for TaskList<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut list_items = Vec::new();
        let execution_order = self.store.visible_tasks;
        let statuses = self.store.task_statuses;

        for (i, name) in execution_order.iter().enumerate() {
            let status = match statuses.get(name) {
                Some(s) => s,
                None => continue,
            };
            let (status_icon, base_style) = match status {
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

            let is_selected = i == self.selected_index;
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

        let block = Block::default()
            .title(" Tasks ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        List::new(list_items).block(block).render(area, buf);
    }
}
