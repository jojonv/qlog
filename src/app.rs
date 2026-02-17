use crate::model::{Filter, FilterSet, LogEntry};
use crossterm::event::KeyCode;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Filter,
    DateRange,
    Command,
}

#[derive(Debug, Clone)]
pub enum LoadingStatus {
    Idle,
    Loading { current: usize, total: usize },
    Complete { loaded: usize, errors: usize },
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
            // Try to receive any new logs without blocking
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
        self.loading_status = LoadingStatus::Complete {
            loaded: self.logs.len(),
            errors: 0,
        };
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

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.mode == Mode::Normal {
                    self.should_quit = true;
                } else {
                    self.mode = Mode::Normal;
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.selected_line =
                    (self.selected_line + 1).min(self.filtered_logs.len().saturating_sub(1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_line = self.selected_line.saturating_sub(1);
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.horizontal_scroll = self.horizontal_scroll.saturating_sub(1);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.horizontal_scroll += 1;
            }
            KeyCode::Char('G') => {
                self.selected_line = self.filtered_logs.len().saturating_sub(1);
            }
            KeyCode::Char('g') if key.modifiers.contains(crossterm::event::KeyModifiers::NONE) => {
                self.selected_line = 0;
            }
            KeyCode::Char(' ') => {
                // Toggle filter
            }
            KeyCode::Char('d') => {
                // Delete filter
            }
            KeyCode::Char('f') => {
                self.mode = Mode::Filter;
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
