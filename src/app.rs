use crate::{
    components::{FlowGraph, HelpBar, LogViewer, TaskList, task_list::TaskListEvent},
    runner::{TaskRunner, TaskStatus, log_buffer::LogBuffer},
    store::Store,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
};
use std::{collections::HashMap, io, time::Duration};

pub struct App {
    runner: TaskRunner,
    selected_task_index: usize,
    log_scroll_offset: u16,
    exit: bool,
    auto_scroll: bool,
    visible_tasks: Vec<String>,
    cached_statuses: HashMap<String, TaskStatus>,
    cached_logs: HashMap<String, LogBuffer>,
    is_interactive: bool,
}

impl App {
    pub fn new(runner: TaskRunner, initial_tasks: Vec<String>) -> Self {
        let visible_tasks = if initial_tasks.is_empty() {
            runner.execution_order.clone()
        } else {
            let subgraph = runner.get_subgraph(&initial_tasks);
            runner
                .execution_order
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

        let mut cached_statuses = HashMap::new();
        {
            let states_guard = runner.states.lock().unwrap();
            for (name, state) in states_guard.iter() {
                cached_statuses.insert(name.clone(), state.status);
            }
        }

        let mut cached_logs = HashMap::new();
        if let Some(first_task) = visible_tasks.get(selected_task_index) {
            let states_guard = runner.states.lock().unwrap();
            if let Some(state) = states_guard.get(first_task) {
                cached_logs.insert(first_task.clone(), state.output.lock().unwrap().clone());
            }
        }

        Self {
            runner,
            selected_task_index,
            log_scroll_offset: 0,
            exit: false,
            auto_scroll: true,
            visible_tasks,
            cached_statuses,
            cached_logs,
            is_interactive: false,
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut result = Ok(());
        while !self.exit {
            if let Err(e) = terminal.draw(|frame| self.draw(frame)) {
                result = Err(e);
                break;
            }
            if let Err(e) = self.handle_events() {
                result = Err(e);
                break;
            }
        }
        self.runner.shutdown().await;
        result
    }

    fn selected_task_name(&self) -> Option<String> {
        self.visible_tasks.get(self.selected_task_index).cloned()
    }

    fn draw(&mut self, frame: &mut Frame) {
        let selected_name = self.selected_task_name();

        // try_lock to update cached statuses and logs
        if let Ok(states_guard) = self.runner.states.try_lock() {
            for (name, state) in states_guard.iter() {
                self.cached_statuses.insert(name.clone(), state.status);
            }
            if let Some(ref selected) = selected_name {
                if let Some(state) = states_guard.get(selected) {
                    if let Ok(logs_guard) = state.output.try_lock() {
                        let needs_update = match self.cached_logs.get(selected) {
                            Some(cached) => cached.version != logs_guard.version,
                            None => true,
                        };
                        if needs_update {
                            self.cached_logs
                                .insert(selected.clone(), (*logs_guard).clone());
                        }
                    }
                }
            }
        }

        let size = frame.area();

        let help_height = HelpBar::estimate_height(self.is_interactive, size.width);

        // Split vertically (Body & Flow Graph & Help Bar)
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Min(0),
                Constraint::Length(5), // Height 5 for execution flow graph (inc borders)
                Constraint::Length(help_height), // Help bar
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

        // Prepare context store for sharing global data (pure snapshotted data)
        let store = Store {
            tasks: &self.runner.tasks,
            task_statuses: &self.cached_statuses,
            visible_tasks: &self.visible_tasks,
        };

        // --- Left Sidebar: Task List ---
        let task_list = TaskList {
            store: &store,
            selected_index: self.selected_task_index,
        };
        frame.render_widget(task_list, sidebar_area);

        // --- Right Log Area: Selected Task Output ---
        let logs_slice = if let Some(ref name) = selected_name {
            self.cached_logs
                .get(name)
                .map(|v| v.lines.as_slice())
                .unwrap_or(&[])
        } else {
            &[]
        };
        let logs_len = logs_slice.len();

        let selected_task_desc = selected_name
            .as_ref()
            .and_then(|name| store.tasks.get(name))
            .and_then(|task| task.description.as_deref());

        // Calculate auto scroll constraint
        let has_description = selected_task_desc.is_some();
        let overhead = if has_description { 4 } else { 2 };
        let content_height = log_area.height.saturating_sub(overhead) as usize;
        let max_scroll = logs_len.saturating_sub(content_height) as u16;

        if self.auto_scroll {
            self.log_scroll_offset = max_scroll;
        } else if self.log_scroll_offset > max_scroll {
            self.log_scroll_offset = max_scroll;
        }

        let log_viewer = LogViewer {
            task_name: selected_name.as_deref(),
            task_description: selected_task_desc,
            logs: logs_slice,
            scroll_offset: self.log_scroll_offset,
            is_interactive: self.is_interactive,
        };
        frame.render_widget(log_viewer, log_area);

        // --- Execution Flow Graph ---
        let flow_graph = FlowGraph { store: &store };
        frame.render_widget(flow_graph, graph_area);

        // --- Help Bar ---
        frame.render_widget(
            HelpBar {
                is_interactive: self.is_interactive,
            },
            help_area,
        );
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
        if self.is_interactive {
            let has_stdin = if let Some(selected_name) = self.selected_task_name() {
                self.cached_statuses.get(&selected_name) == Some(&TaskStatus::Running)
            } else {
                false
            };

            if !has_stdin {
                self.is_interactive = false;
            } else if key_event.code == KeyCode::Esc {
                self.is_interactive = false;
                return;
            } else if let Some(selected_name) = self.selected_task_name() {
                let tx = if let Ok(states) = self.runner.states.try_lock() {
                    states.get(&selected_name).and_then(|s| s.stdin_tx.clone())
                } else {
                    None
                };
                if let Some(tx) = tx {
                    let bytes = match key_event.code {
                        KeyCode::Char(c) => {
                            if key_event.modifiers.contains(event::KeyModifiers::CONTROL) {
                                match c {
                                    'c' => vec![3],  // Ctrl+C (ETX)
                                    'd' => vec![4],  // Ctrl+D (EOT)
                                    'z' => vec![26], // Ctrl+Z (SUB)
                                    _ => {
                                        let c_upper = c.to_ascii_uppercase();
                                        if c_upper >= 'A' && c_upper <= 'Z' {
                                            vec![(c_upper as u8) - b'A' + 1]
                                        } else {
                                            c.to_string().into_bytes()
                                        }
                                    }
                                }
                            } else {
                                c.to_string().into_bytes()
                            }
                        }
                        KeyCode::Enter => vec![b'\n'],
                        KeyCode::Tab => vec![b'\t'],
                        KeyCode::Backspace => vec![8], // BS
                        KeyCode::Delete => vec![127],  // DEL
                        KeyCode::Up => vec![27, 91, 65],
                        KeyCode::Down => vec![27, 91, 66],
                        KeyCode::Right => vec![27, 91, 67],
                        KeyCode::Left => vec![27, 91, 68],
                        _ => vec![],
                    };
                    if !bytes.is_empty() {
                        let _ = tx.send(bytes);
                    }
                }
                return;
            }
        }

        // 1. Global keyboard shortcuts
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.exit = true;
                return;
            }
            KeyCode::Char('i') => {
                if let Some(selected_name) = self.selected_task_name() {
                    let has_stdin = if let Ok(states) = self.runner.states.try_lock() {
                        states
                            .get(&selected_name)
                            .map(|s| s.status == TaskStatus::Running && s.stdin_tx.is_some())
                            .unwrap_or(false)
                    } else {
                        self.cached_statuses.get(&selected_name) == Some(&TaskStatus::Running)
                    };
                    if has_stdin {
                        self.is_interactive = true;
                    }
                }
                return;
            }
            KeyCode::Char('A') => {
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

        // 2. Delegate key handling to TaskList
        if let Some(event) =
            TaskList::handle_key_event(key_event, self.selected_task_index, &self.visible_tasks)
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
                TaskListEvent::Stop(name) => {
                    self.runner.stop_task(&name, false);
                }
                TaskListEvent::StopAndNext(name) => {
                    self.runner.stop_task(&name, true);
                }
            }
            return;
        }

        // 3. Delegate to LogViewer for log scroll control
        if let Some(selected_name) = self.selected_task_name() {
            let logs_len = self
                .cached_logs
                .get(&selected_name)
                .map(|v| v.len())
                .unwrap_or(0);

            let has_description = self
                .runner
                .tasks
                .get(&selected_name)
                .and_then(|task| task.description.as_ref())
                .is_some();

            // Estimate log area height from window dimensions
            let (_, term_height) = crossterm::terminal::size().unwrap_or((80, 24));
            let area_height = term_height.saturating_sub(6); // body area height

            if let Some((offset, auto)) = LogViewer::handle_key_event(
                key_event,
                self.log_scroll_offset,
                logs_len,
                area_height,
                has_description,
            ) {
                self.log_scroll_offset = offset;
                self.auto_scroll = auto;
            }
        }
    }
}
