use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct LogViewer<'a> {
    pub task_name: Option<&'a str>,
    pub task_description: Option<&'a str>,
    pub logs: &'a [String],
    pub scroll_offset: u16,
}

impl LogViewer<'_> {
    pub fn handle_key_event(
        key: KeyEvent,
        current_offset: u16,
        logs_len: usize,
        area_height: u16,
        has_description: bool,
    ) -> Option<(u16, bool)> {
        let overhead = if has_description { 4 } else { 2 };
        let content_height = area_height.saturating_sub(overhead) as usize; // account for borders
        let max_scroll = logs_len.saturating_sub(content_height) as u16;

        let mut offset = current_offset;
        let auto;

        match key.code {
            KeyCode::Up | KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                offset = offset.saturating_sub(1);
                auto = false;
            }
            KeyCode::Down | KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                offset = offset.saturating_add(1);
                // Maintain auto scroll if scrolled near the bottom
                if (offset as usize) >= logs_len.saturating_sub(15) {
                    auto = true;
                } else {
                    auto = false;
                }
            }
            KeyCode::PageUp => {
                offset = offset.saturating_sub(10);
                auto = false;
            }
            KeyCode::PageDown => {
                offset = offset.saturating_add(10);
                auto = false;
            }
            _ => return None,
        }

        if auto {
            offset = max_scroll;
        } else if offset > max_scroll {
            offset = max_scroll;
        }

        Some((offset, auto))
    }
}

impl Widget for LogViewer<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Some(name) = self.task_name {
            // Stylize log output
            let text: Vec<Line> = self
                .logs
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

            let log_title = format!(" Output: {} ", name);
            let log_block = Block::default()
                .title(log_title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan));

            let inner_area = log_block.inner(area);
            log_block.render(area, buf);

            let logs_area = if let Some(desc) = self.task_description {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![
                        Constraint::Length(1), // description line
                        Constraint::Length(1), // separator line
                        Constraint::Min(0),    // logs
                    ])
                    .split(inner_area);

                // Render description
                let desc_line = Line::from(vec!["Description: ".bold().cyan(), desc.into()]);
                Paragraph::new(desc_line).render(chunks[0], buf);

                // Render separator
                let sep_char = "─";
                let separator_text = sep_char.repeat(inner_area.width as usize);
                Paragraph::new(separator_text.dark_gray()).render(chunks[1], buf);

                chunks[2]
            } else {
                inner_area
            };

            Paragraph::new(text)
                .scroll((self.scroll_offset, 0))
                .render(logs_area, buf);
        } else {
            let empty_block = Block::default()
                .title(" Output ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            Paragraph::new("No task selected.")
                .block(empty_block)
                .dark_gray()
                .render(area, buf);
        }
    }
}
