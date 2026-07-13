use crate::{runner::TaskStatus, store::Store};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::collections::HashMap;

pub struct FlowGraph<'a> {
    pub store: &'a Store<'a>,
}

impl Widget for FlowGraph<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let execution_order = &self.store.runner.execution_order;
        let tasks = &self.store.runner.tasks;
        let states_guard = self.store.runner.states.lock().unwrap();

        // Calculate levels (column positions) for topological execution order
        let mut levels = HashMap::new();
        for name in execution_order {
            let task = match tasks.get(name) {
                Some(t) => t,
                None => continue,
            };
            let level = match &task.depends_on {
                None => 0,
                Some(deps) if deps.is_empty() => 0,
                Some(deps) => {
                    let mut max_dep_level = 0;
                    for dep in deps {
                        let dep_lvl = levels.get(dep).cloned().unwrap_or(0);
                        if dep_lvl > max_dep_level {
                            max_dep_level = dep_lvl;
                        }
                    }
                    max_dep_level + 1
                }
            };
            levels.insert(name.clone(), level);
        }

        let max_level = levels.values().cloned().max().unwrap_or(0);
        let mut col_nodes: Vec<Vec<String>> = vec![Vec::new(); max_level + 1];
        for name in execution_order {
            if let Some(&lvl) = levels.get(name) {
                col_nodes[lvl].push(name.clone());
            }
        }

        // Calculate max width for each column
        let mut col_widths = vec![0; max_level + 1];
        for c in 0..=max_level {
            let mut max_w = 0;
            for name in &col_nodes[c] {
                let w = name.len() + 2; // account for borders [ ]
                if w > max_w {
                    max_w = w;
                }
            }
            col_widths[c] = max_w;
        }

        let h = 3; // layout height for execution graph
        let mut grid: Vec<Vec<Option<String>>> = vec![vec![None; max_level + 1]; h];
        for c in 0..=max_level {
            let k = col_nodes[c].len();
            if k == 1 {
                grid[h / 2][c] = Some(col_nodes[c][0].clone());
            } else if k == 2 {
                grid[0][c] = Some(col_nodes[c][0].clone());
                grid[h - 1][c] = Some(col_nodes[c][1].clone());
            } else if k >= 3 {
                for i in 0..k {
                    let r = i * (h - 1) / (k - 1);
                    grid[r][c] = Some(col_nodes[c][i].clone());
                }
            }
        }

        let mut lines = Vec::new();
        for r in 0..h {
            let mut spans = Vec::new();
            for c in 0..=max_level {
                let width = col_widths[c];
                if let Some(ref name) = grid[r][c] {
                    let state = states_guard.get(name).unwrap();
                    let color = match state.status {
                        TaskStatus::Idle => Color::DarkGray,
                        TaskStatus::Pending => Color::Yellow,
                        TaskStatus::Running => Color::Blue,
                        TaskStatus::Success => Color::Green,
                        TaskStatus::Failed => Color::Red,
                    };
                    spans.push(Span::raw("["));
                    spans.push(Span::styled(
                        name.clone(),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ));
                    spans.push(Span::raw("]"));

                    let rendered_len = name.len() + 2;
                    if rendered_len < width {
                        spans.push(Span::raw(" ".repeat(width - rendered_len)));
                    }
                } else {
                    spans.push(Span::raw(" ".repeat(width)));
                }

                if c < max_level {
                    let k1 = col_nodes[c].len();
                    let k2 = col_nodes[c + 1].len();
                    let conn = match (k1, k2) {
                        (1, 1) => {
                            if r == h / 2 {
                                " ──► "
                            } else {
                                "        "
                            }
                        }
                        (n, 1) if n > 1 => {
                            if r == 0 {
                                " ─┐   "
                            } else if r == h / 2 {
                                "  ├► "
                            } else if r == h - 1 {
                                " ─┘   "
                            } else {
                                "        "
                            }
                        }
                        (1, n) if n > 1 => {
                            if r == 0 {
                                "  ┌► "
                            } else if r == h / 2 {
                                "  ─┤   "
                            } else if r == h - 1 {
                                "  └► "
                            } else {
                                "        "
                            }
                        }
                        _ => " ──► ",
                    };
                    spans.push(Span::styled(conn, Style::default().fg(Color::DarkGray)));
                }
            }
            lines.push(Line::from(spans));
        }

        let block = Block::default()
            .title(" Execution Flow Graph ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        Paragraph::new(lines).block(block).render(area, buf);
    }
}
