use crate::{
    components::{FlowGraph, HelpBar, LogViewer, TaskList, task_list::TaskListEvent},
    runner::TaskRunner,
    store::Store,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
};
use std::{io, time::Duration};

pub struct App {
    runner: TaskRunner,
    selected_task_index: usize,
    log_scroll_offset: u16,
    exit: bool,
    auto_scroll: bool,
    visible_tasks: Vec<String>,
}

impl App {
    pub fn new(runner: TaskRunner, initial_tasks: Vec<String>) -> Self {
        let visible_tasks = if initial_tasks.is_empty() {
            runner.execution_order.clone()
        } else {
            let subgraph = runner.get_subgraph(&initial_tasks);
            runner.execution_order
                .iter()
                .filter(|name| subgraph.contains(*name))
                .cloned()
                .collect::<Vec<String>>()
        };

        let mut selected_task_index = 0;
        if let Some(first_task) = initial_tasks.first() {
            if let Some(pos) = visible_tasks.iter().position(|name| name == first_task) {
                selected_task_index = pos;
            }
        }

        if !initial_tasks.is_empty() {
            runner.run_tasks(&initial_tasks);
        }

        Self {
            runner,
            selected_task_index,
            log_scroll_offset: 0,
            exit: false,
            auto_scroll: true,
            visible_tasks,
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
        self.visible_tasks
            .get(self.selected_task_index)
            .cloned()
    }

    fn draw(&mut self, frame: &mut Frame) {
        let size = frame.area();

        // Split vertically (Body & Flow Graph & Help Bar)
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Min(0),
                Constraint::Length(5), // Height 5 for execution flow graph (inc borders)
                Constraint::Length(1), // Help bar
            ])
            .split(size);

        let body_area = main_layout[0];
        let graph_area = main_layout[1];
        let help_area = main_layout[2];

        // Split body horizontally (Sidebar & Log Area)
        let body_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(body_area);

        let sidebar_area = body_layout[0];
        let log_area = body_layout[1];

        // Prepare context store for sharing global data
        let store = Store {
            runner: &self.runner,
            visible_tasks: &self.visible_tasks,
        };

        // --- Left Sidebar: Task List ---
        let task_list = TaskList {
            store: &store,
            selected_index: self.selected_task_index,
        };
        frame.render_widget(task_list, sidebar_area);

        // --- Right Log Area: Selected Task Output ---
        if let Some(selected_name) = self.selected_task_name() {
            let states_guard = self.runner.states.lock().unwrap();
            let state = states_guard.get(&selected_name).unwrap();
            let logs_guard = state.output.lock().unwrap();
            let logs_len = logs_guard.len();

            // Calculate auto scroll constraint
            let content_height = log_area.height.saturating_sub(2) as usize;
            let max_scroll = logs_len.saturating_sub(content_height) as u16;

            if self.auto_scroll {
                self.log_scroll_offset = max_scroll;
            } else if self.log_scroll_offset > max_scroll {
                self.log_scroll_offset = max_scroll;
            }

            let log_viewer = LogViewer {
                task_name: Some(&selected_name),
                logs: &logs_guard,
                scroll_offset: self.log_scroll_offset,
            };
            frame.render_widget(log_viewer, log_area);
        } else {
            let log_viewer = LogViewer {
                task_name: None,
                logs: &[],
                scroll_offset: 0,
            };
            frame.render_widget(log_viewer, log_area);
        }

        // --- Execution Flow Graph ---
        let flow_graph = FlowGraph { store: &store };
        frame.render_widget(flow_graph, graph_area);

        // --- Help Bar ---
        frame.render_widget(HelpBar, help_area);
    }

    fn handle_events(&mut self) -> io::Result<()> {
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
        // 1. Global keyboard shortcuts
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.exit = true;
                return;
            }
            KeyCode::Char('a') => {
                self.runner.run_all();
                self.log_scroll_offset = 0;
                self.auto_scroll = true;
                return;
            }
            KeyCode::Char('C') => {
                self.runner.clear_all_logs();
                self.log_scroll_offset = 0;
                self.auto_scroll = true;
                return;
            }
            _ => {}
        }

        let store = Store {
            runner: &self.runner,
            visible_tasks: &self.visible_tasks,
        };

        // 2. Delegate key handling to TaskList
        if let Some(event) = TaskList::handle_key_event(key_event, self.selected_task_index, &store)
        {
            match event {
                TaskListEvent::Select(new_index) => {
                    self.selected_task_index = new_index;
                    self.log_scroll_offset = 0;
                    self.auto_scroll = true;
                }
                TaskListEvent::Run(name) => {
                    self.runner.run_task(&name);
                    self.log_scroll_offset = 0;
                    self.auto_scroll = true;
                }
                TaskListEvent::Clear(name) => {
                    self.runner.clear_logs(&name);
                    self.log_scroll_offset = 0;
                    self.auto_scroll = true;
                }
            }
            return;
        }

        // 3. Delegate to LogViewer for log scroll control
        if let Some(selected_name) = self.selected_task_name() {
            let states_guard = self.runner.states.lock().unwrap();
            if let Some(state) = states_guard.get(&selected_name) {
                let logs_len = state.output.lock().unwrap().len();

                // Estimate log area height from window dimensions
                let (_, term_height) = crossterm::terminal::size().unwrap_or((80, 24));
                let area_height = term_height.saturating_sub(6); // body area height

                if let Some((offset, auto)) = LogViewer::handle_key_event(
                    key_event,
                    self.log_scroll_offset,
                    logs_len,
                    area_height,
                ) {
                    self.log_scroll_offset = offset;
                    self.auto_scroll = auto;
                }
            }
        }
    }
}
