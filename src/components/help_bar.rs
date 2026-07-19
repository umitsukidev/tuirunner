use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Paragraph, Widget, Wrap},
};

pub struct HelpBar {
    pub is_interactive: bool,
}

impl HelpBar {
    fn build_line(is_interactive: bool) -> Line<'static> {
        if is_interactive {
            Line::from(vec![
                " Interactive Mode ".bold().yellow(),
                " │ Exit: ".into(),
                "Esc".bold().red(),
            ])
        } else {
            Line::from(vec![
                "Move: ".into(),
                "↑/↓/j/k".bold().cyan(),
                "  •  ".dark_gray(),
                "Run: ".into(),
                "r/Enter".bold().green(),
                "  •  ".dark_gray(),
                "Run All: ".into(),
                "A".bold().green(),
                "  •  ".dark_gray(),
                "Stop: ".into(),
                "s/S".bold().red(),
                "  •  ".dark_gray(),
                "Interactive: ".into(),
                "i".bold().yellow(),
                "  •  ".dark_gray(),
                "Scroll: ".into(),
                "PgUp/PgDn/Shift+↑/Shift+↓".bold().cyan(),
                "  •  ".dark_gray(),
                "Clear: ".into(),
                "c/C".bold().yellow(),
                "  •  ".dark_gray(),
                "Quit: ".into(),
                "q/Esc".bold().red(),
            ])
        }
    }

    pub fn estimate_height(is_interactive: bool, width: u16) -> u16 {
        let line = Self::build_line(is_interactive);
        let plain_text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        let limit = width.max(1) as usize;

        let mut lines = 0;
        let mut current_line_len = 0;

        let words: Vec<&str> = plain_text.split(' ').collect();
        let mut i = 0;
        while i < words.len() {
            let word = words[i];
            let word_len = word.chars().count();

            if word_len == 0 {
                if current_line_len > 0 && current_line_len < limit {
                    current_line_len += 1;
                }
                i += 1;
                continue;
            }

            if current_line_len == 0 {
                if word_len > limit {
                    lines += word_len.div_ceil(limit);
                    current_line_len = word_len % limit;
                } else {
                    current_line_len = word_len;
                }
            } else {
                if current_line_len + 1 + word_len <= limit {
                    current_line_len += 1 + word_len;
                } else {
                    lines += 1;
                    current_line_len = 0;
                    continue;
                }
            }
            i += 1;
        }

        if current_line_len > 0 {
            lines += 1;
        }

        lines.max(1) as u16
    }
}

impl Widget for HelpBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let help_text = Self::build_line(self.is_interactive);
        Paragraph::new(help_text)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_height_interactive() {
        assert_eq!(HelpBar::estimate_height(true, 80), 1);
        assert_eq!(HelpBar::estimate_height(true, 30), 1);
        assert_eq!(HelpBar::estimate_height(true, 15), 3);
    }

    #[test]
    fn test_estimate_height_standard() {
        assert_eq!(HelpBar::estimate_height(false, 160), 1);
        assert_eq!(HelpBar::estimate_height(false, 80), 2);
        assert_eq!(HelpBar::estimate_height(false, 50), 4);
        assert!(HelpBar::estimate_height(false, 20) >= 8);
    }
}
