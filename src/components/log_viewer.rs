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
    pub is_interactive: bool,
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
                auto = (offset as usize) >= logs_len.saturating_sub(15);
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

        if auto || offset > max_scroll {
            offset = max_scroll;
        }

        Some((offset, auto))
    }
}

impl<'a> Widget for LogViewer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Some(name) = self.task_name {
            let log_title = if self.is_interactive {
                format!(" Output (Interactive): {} ", name)
            } else {
                format!(" Output: {} ", name)
            };
            let border_color = if self.is_interactive {
                Color::Yellow
            } else {
                Color::Cyan
            };
            let log_block = Block::default()
                .title(log_title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color));

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

            // Calculate visible logs based on scroll offset and logs area height
            let start = self.scroll_offset as usize;
            let end = (start + logs_area.height as usize).min(self.logs.len());
            let visible_logs = if start < self.logs.len() {
                &self.logs[start..end]
            } else {
                &[]
            };

            // Stylize log output for the visible range only
            let mut text: Vec<Line<'a>> = Vec::new();
            for line in visible_logs {
                if line.starts_with("=== ") {
                    text.push(Line::from(line.clone().cyan().bold()));
                } else if line.starts_with("$ ") {
                    text.push(Line::from(line.clone().dim()));
                } else if line.contains('\x1b') {
                    use ansi_to_tui::IntoText;
                    if let Ok(ansi_text) = line.as_bytes().into_text() {
                        text.extend(ansi_text.lines);
                    } else {
                        text.push(Line::from(line.clone()));
                    }
                } else if line.contains("failed") || line.contains("Failed") {
                    text.push(Line::from(line.clone().red().bold()));
                } else if line.contains("succeeded") || line.contains("Success") {
                    text.push(Line::from(line.clone().green().bold()));
                } else {
                    text.push(Line::from(line.clone()));
                }
            }

            Paragraph::new(text).render(logs_area, buf);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn test_page_navigation_clamps_to_log_bounds() {
        let page_down = key(KeyCode::PageDown, KeyModifiers::NONE);

        assert_eq!(
            LogViewer::handle_key_event(page_down, 10, 20, 5, false),
            Some((17, false))
        );
    }

    #[test]
    fn test_shift_down_restores_auto_scroll_near_the_bottom() {
        let result =
            LogViewer::handle_key_event(key(KeyCode::Down, KeyModifiers::SHIFT), 24, 40, 10, false);

        assert_eq!(result, Some((32, true)));
    }

    #[test]
    fn test_unhandled_key_does_not_change_scroll() {
        assert_eq!(
            LogViewer::handle_key_event(
                key(KeyCode::Char('j'), KeyModifiers::NONE),
                5,
                20,
                10,
                false
            ),
            None
        );
    }
}
