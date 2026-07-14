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
    pub fn estimate_height(is_interactive: bool, width: u16) -> u16 {
        let help_text_len = if is_interactive {
            30 // Length of interactive mode help
        } else {
            146 // Length of standard mode help text
        };
        let term_width = width.max(1) as usize;
        // Add a buffer for word wrap boundaries
        let estimated_len = help_text_len;
        ((estimated_len + term_width - 1) / term_width).max(1) as u16
    }
}

impl Widget for HelpBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let help_text = if self.is_interactive {
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
        };
        Paragraph::new(help_text)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}
