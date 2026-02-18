use crate::model::{FilterSet, LogEntry};
use chrono::Local;
use crossterm::event::KeyCode;
use std::cell::Cell;
use std::fs::File;
use std::io::{self, Write};
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
    pub wrap_mode: bool,
    pub viewport_height: Cell<usize>,
    pub viewport_width: Cell<usize>,
    pub max_line_width: usize,
    pub visual_line_offsets: Vec<usize>,
    pub total_visual_lines: usize,
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
            wrap_mode: true,
            viewport_height: Cell::new(20),
            viewport_width: Cell::new(80),
            max_line_width: 0,
            visual_line_offsets: Vec::new(),
            total_visual_lines: 0,
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

    pub fn extend_logs(&mut self, new_logs: Vec<LogEntry>) {
        self.logs.extend(new_logs);
    }

    pub fn update_filtered_logs(&mut self) {
        self.filtered_logs = self
            .logs
            .iter()
            .filter(|entry| self.filters.matches(entry))
            .cloned()
            .collect();
        self.recalculate_max_line_width();
        self.recalculate_visual_lines();
    }

    pub fn recalculate_max_line_width(&mut self) {
        self.max_line_width = self
            .filtered_logs
            .iter()
            .map(|entry| {
                let ts_len = entry.timestamp.as_ref().map(|_| 20).unwrap_or(0);
                ts_len + entry.raw.chars().count()
            })
            .max()
            .unwrap_or(0);
    }

    pub fn recalculate_visual_lines(&mut self) {
        self.visual_line_offsets.clear();
        if self.filtered_logs.is_empty() {
            self.total_visual_lines = 0;
            return;
        }

        let viewport_width = self.viewport_width.get();
        let mut offset = 0usize;

        for entry in &self.filtered_logs {
            self.visual_line_offsets.push(offset);
            let ts_len = entry.timestamp.as_ref().map(|_| 20).unwrap_or(0);
            let text_width = ts_len + entry.raw.chars().count();
            let visual_lines = if self.wrap_mode && viewport_width > 0 {
                ((text_width + viewport_width - 1) / viewport_width).max(1)
            } else {
                1
            };
            offset += visual_lines;
        }
        self.total_visual_lines = offset;
    }

    pub fn selected_visual_line(&self) -> usize {
        self.visual_line_offsets
            .get(self.selected_line)
            .copied()
            .unwrap_or(0)
    }

    pub fn scroll_visual_line(&self) -> usize {
        self.visual_line_offsets
            .get(self.scroll_offset)
            .copied()
            .unwrap_or(0)
    }

    pub fn find_entry_by_visual_line(&self, target_visual: usize) -> usize {
        match self.visual_line_offsets.binary_search(&target_visual) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        }
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
            Mode::Filter => self.handle_filter_key(key),
            Mode::DateRange => {}
            Mode::Normal => self.handle_normal_key(key),
        }
    }

    fn handle_filter_input_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Filter;
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
                self.mode = Mode::Filter;
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
                self.execute_command();
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

    fn execute_command(&mut self) {
        let input = self.input_buffer.trim();
        if input.is_empty() {
            return;
        }

        let (command, filename) = Self::parse_command(input);

        match command {
            "w" | "write" => {
                let output_filename = if filename.is_empty() {
                    Self::generate_default_filename()
                } else {
                    filename.to_string()
                };

                match self.write_filtered_logs(&output_filename) {
                    Ok(count) => {
                        self.status_message =
                            format!("Saved {} lines to {}", count, output_filename);
                    }
                    Err(e) => {
                        self.status_message = format!("Error: {}", e);
                    }
                }
            }
            _ => {
                self.status_message = format!("Unknown command: {}", command);
            }
        }
    }

    fn parse_command(input: &str) -> (&str, &str) {
        let input = input.trim();

        if input.starts_with('"') {
            if let Some(end_quote) = input[1..].find('"') {
                let filename = &input[1..end_quote + 1];
                let rest = &input[end_quote + 2..].trim_start();
                if let Some(space_pos) = rest.find(' ') {
                    let cmd = &rest[..space_pos];
                    return (cmd, filename);
                }
                return (rest, filename);
            }
        }

        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        if parts.len() == 2 {
            let filename = parts[1].trim();
            if filename.starts_with('"') && filename.ends_with('"') && filename.len() > 1 {
                return (parts[0], &filename[1..filename.len() - 1]);
            }
            (parts[0], filename)
        } else {
            (parts[0], "")
        }
    }

    fn generate_default_filename() -> String {
        format!("filtered-logs-{}.log", Local::now().format("%Y%m%d-%H%M%S"))
    }

    fn write_filtered_logs(&self, filename: &str) -> io::Result<usize> {
        let mut file = File::create(filename)?;
        let mut count = 0;
        for entry in &self.filtered_logs {
            writeln!(file, "{}", entry.raw)?;
            count += 1;
        }
        Ok(count)
    }

    fn handle_normal_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.status_message.clear();
                if self.selected_line < self.filtered_logs.len().saturating_sub(1) {
                    self.selected_line += 1;
                }
                self.clamp_scroll();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.status_message.clear();
                self.selected_line = self.selected_line.saturating_sub(1);
                self.clamp_scroll();
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.horizontal_scroll = self.horizontal_scroll.saturating_add(4);
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.horizontal_scroll = self.horizontal_scroll.saturating_sub(4);
            }
            KeyCode::Char('G') => {
                self.selected_line = self.filtered_logs.len().saturating_sub(1);
                self.clamp_scroll();
            }
            KeyCode::Char('g') if key.modifiers.contains(crossterm::event::KeyModifiers::NONE) => {
                self.selected_line = 0;
                self.scroll_offset = 0;
            }
            KeyCode::Char('t') => {
                self.mode = Mode::Filter;
            }
            KeyCode::Char(':') => {
                self.mode = Mode::Command;
            }
            KeyCode::Char('w') => {
                self.wrap_mode = !self.wrap_mode;
                self.recalculate_visual_lines();
                self.clamp_scroll();
                self.status_message = if self.wrap_mode {
                    "Wrap mode enabled".to_string()
                } else {
                    "Wrap mode disabled".to_string()
                };
            }
            _ => {}
        }
    }

    fn clamp_scroll(&mut self) {
        if self.filtered_logs.is_empty() {
            return;
        }

        self.selected_line = self
            .selected_line
            .min(self.filtered_logs.len().saturating_sub(1));

        let viewport_height = self.viewport_height.get();

        let effective_height = if self.wrap_mode {
            (viewport_height / 2).max(1)
        } else {
            viewport_height
        };

        if self.selected_line < self.scroll_offset {
            self.scroll_offset = self.selected_line;
        } else if self.selected_line >= self.scroll_offset + effective_height {
            self.scroll_offset = self
                .selected_line
                .saturating_sub(effective_height.saturating_sub(1));
        }
    }

    fn handle_filter_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
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
            KeyCode::Char('t') | KeyCode::Esc => {
                self.mode = Mode::Normal;
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
