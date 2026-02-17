use crate::model::{FilterSet, LogEntry};
use crossterm::event::KeyCode;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Filter,
    FilterInput,
    Command,
    DateRange,
}

#[derive(Debug, Clone)]
pub enum LoadingStatus {
    Idle,
    Loading { current: usize, total: usize },
    Complete,
    Error(String),
}

#[derive(Debug)]
pub struct App {
    pub logs: Vec<LogEntry>,
    pub filtered_logs: Vec<LogEntry>,
    pub filters: FilterSet,
    pub mode: Mode,
    pub should_quit: bool,
    pub status_message: String,
    pub scroll_offset: usize,
    pub horizontal_scroll: usize,
    pub selected_line: usize,
    pub loading_status: LoadingStatus,
    pub log_receiver: Option<Receiver<Vec<LogEntry>>>,
    pub selected_group: usize,
    pub selected_filter: usize,
    pub input_buffer: String,
    pub pending_new_group: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            filtered_logs: Vec::new(),
            filters: FilterSet::new(),
            mode: Mode::Normal,
            should_quit: false,
            status_message: String::new(),
            scroll_offset: 0,
            horizontal_scroll: 0,
            selected_line: 0,
            loading_status: LoadingStatus::Idle,
            log_receiver: None,
            selected_group: 0,
            selected_filter: 0,
            input_buffer: String::new(),
            pending_new_group: false,
        }
    }

    pub fn start_loading(&mut self) -> Sender<Vec<LogEntry>> {
        let (tx, rx) = channel();
        self.log_receiver = Some(rx);
        self.loading_status = LoadingStatus::Loading {
            current: 0,
            total: 31,
        };
        tx
    }

    pub fn check_for_loaded_logs(&mut self) {
        if let Some(ref receiver) = self.log_receiver {
            while let Ok(new_logs) = receiver.try_recv() {
                self.logs.extend(new_logs);
                if let LoadingStatus::Loading { current, total } = self.loading_status {
                    self.loading_status = LoadingStatus::Loading {
                        current: current + 1,
                        total,
                    };
                }
            }
        }
    }

    pub fn finish_loading(&mut self) {
        self.loading_status = LoadingStatus::Complete;
        self.log_receiver = None;
        self.update_filtered_logs();
    }

    pub fn update_filtered_logs(&mut self) {
        self.filtered_logs = self
            .logs
            .iter()
            .filter(|entry| self.filters.matches(entry))
            .cloned()
            .collect();
    }

    fn ensure_valid_selection(&mut self) {
        if self.filters.groups.is_empty() {
            self.selected_group = 0;
            self.selected_filter = 0;
            return;
        }
        self.selected_group = self.selected_group.min(self.filters.groups.len() - 1);
        if !self.filters.groups.is_empty() {
            let group = &self.filters.groups[self.selected_group];
            if group.filters.is_empty() {
                self.selected_filter = 0;
            } else {
                self.selected_filter = self.selected_filter.min(group.filters.len() - 1);
            }
        }
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        match self.mode {
            Mode::FilterInput => self.handle_filter_input_key(key),
            Mode::Command => self.handle_command_key(key),
            Mode::Filter | Mode::DateRange => {}
            Mode::Normal => self.handle_normal_key(key),
        }
    }

    fn handle_filter_input_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.input_buffer.clear();
                self.pending_new_group = false;
            }
            KeyCode::Enter => {
                if !self.input_buffer.trim().is_empty() {
                    if self.pending_new_group {
                        self.filters.add_group();
                        self.selected_group = self.filters.groups.len() - 1;
                        self.selected_filter = 0;
                    }
                    if self.filters.groups.is_empty() {
                        self.filters.add_group();
                        self.selected_group = 0;
                        self.selected_filter = 0;
                    }
                    self.filters.add_filter_to_group(
                        self.selected_group,
                        self.input_buffer.trim().to_string(),
                    );
                    self.ensure_valid_selection();
                    self.selected_filter =
                        self.filters.groups[self.selected_group].filters.len() - 1;
                    self.update_filtered_logs();
                }
                self.mode = Mode::Normal;
                self.input_buffer.clear();
                self.pending_new_group = false;
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
    }

    fn handle_command_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.input_buffer.clear();
            }
            KeyCode::Enter => {
                self.mode = Mode::Normal;
                self.input_buffer.clear();
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
    }

    fn handle_normal_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if !self.filters.groups.is_empty() {
                    let group = &self.filters.groups[self.selected_group];
                    if self.selected_filter + 1 < group.filters.len() {
                        self.selected_filter += 1;
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_filter = self.selected_filter.saturating_sub(1);
            }
            KeyCode::Char('h') | KeyCode::Left => {
                if self.selected_group > 0 {
                    self.selected_group -= 1;
                    self.ensure_valid_selection();
                }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if self.selected_group + 1 < self.filters.groups.len() {
                    self.selected_group += 1;
                    self.ensure_valid_selection();
                }
            }
            KeyCode::Char('G') => {
                self.selected_line = self.filtered_logs.len().saturating_sub(1);
            }
            KeyCode::Char('g') if key.modifiers.contains(crossterm::event::KeyModifiers::NONE) => {
                self.selected_line = 0;
            }
            KeyCode::Char(' ') => {
                self.filters
                    .toggle_filter(self.selected_group, self.selected_filter);
                self.update_filtered_logs();
            }
            KeyCode::Char('d') => {
                if !self.filters.groups.is_empty() {
                    self.filters
                        .remove_filter(self.selected_group, self.selected_filter);
                    self.ensure_valid_selection();
                    self.update_filtered_logs();
                }
            }
            KeyCode::Char('F') => {
                self.pending_new_group = true;
                self.mode = Mode::FilterInput;
            }
            KeyCode::Char('f') => {
                self.pending_new_group = false;
                self.mode = Mode::FilterInput;
            }
            KeyCode::Char(':') => {
                self.mode = Mode::Command;
            }
            _ => {}
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
