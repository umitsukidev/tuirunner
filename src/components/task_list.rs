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
}

pub struct TaskList<'a> {
    pub store: &'a Store<'a>,
    pub selected_index: usize,
}

impl TaskList<'_> {
    pub fn handle_key_event(
        key: KeyEvent,
        current_index: usize,
        store: &Store,
    ) -> Option<TaskListEvent> {
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            return None;
        }
        let order = &store.runner.execution_order;
        let mut idx = current_index;

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if idx > 0 {
                    idx -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if idx + 1 < order.len() {
                    idx += 1;
                }
            }
            KeyCode::Char('r') | KeyCode::Enter => {
                if let Some(name) = order.get(idx) {
                    return Some(TaskListEvent::Run(name.clone()));
                }
            }
            KeyCode::Char('c') => {
                if let Some(name) = order.get(idx) {
                    return Some(TaskListEvent::Clear(name.clone()));
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
        let execution_order = &self.store.runner.execution_order;
        let states_guard = self.store.runner.states.lock().unwrap();

        for (i, name) in execution_order.iter().enumerate() {
            let state = match states_guard.get(name) {
                Some(s) => s,
                None => continue,
            };
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
