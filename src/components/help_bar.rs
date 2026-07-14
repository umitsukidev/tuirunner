use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    text::Line,
    widgets::{Paragraph, Widget},
};

pub struct HelpBar;

impl Widget for HelpBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let help_text = Line::from(vec![
            " Move: ".into(),
            "↑/↓/j/k".bold().cyan(),
            " | Run: ".into(),
            "r/Enter".bold().green(),
            " | Run All: ".into(),
            "A".bold().green(),
            " | Stop: ".into(),
            "s (stop)/S (continue)".bold().red(),
            " | Scroll Log: ".into(),
            "PgUp/PgDn/Shift+↑/Shift+↓".bold().cyan(),
            " | Clear: ".into(),
            "c (selected)/C (all)".bold().yellow(),
            " | Quit: ".into(),
            "q/Esc".bold().red(),
        ]);
        Paragraph::new(help_text).render(area, buf);
    }
}
