use crate::config::ColorConfig;
use crate::model::{FilterSet, LogStorage, VisualLineCache};
use chrono::Local;
use crossterm::event::KeyCode;
use ratatui::style::Color;
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
    /// Memory-mapped log storage (replaces Vec<LogEntry>)
    pub storage: Option<LogStorage>,
    /// Indices of lines that match current filters
    pub filtered_indices: Vec<usize>,
    /// Active filters
    pub filters: FilterSet,
    /// Current UI mode
    pub mode: Mode,
    /// Flag to exit the application
    pub should_quit: bool,
    /// Status message to display
    pub status_message: String,
    /// Vertical scroll offset (in filtered lines)
    pub scroll_offset: usize,
    /// Horizontal scroll offset (in characters)
    pub horizontal_scroll: usize,
    /// Currently selected line index (in filtered lines)
    pub selected_line: usize,
    /// Loading status
    pub loading_status: LoadingStatus,
    /// Receiver for async log loading
    pub log_receiver: Option<Receiver<LogStorage>>,
    /// Selected filter group index
    pub selected_group: usize,
    /// Selected filter index within group
    pub selected_filter: usize,
    /// Input buffer for text input
    pub input_buffer: String,
    /// Flag for creating a new filter group
    pub pending_new_group: bool,
    /// Whether line wrapping is enabled
    pub wrap_mode: bool,
    /// Viewport height (updated by UI)
    pub viewport_height: Cell<usize>,
    /// Viewport width (updated by UI)
    pub viewport_width: Cell<usize>,
    /// Cache for visual line calculations
    visual_cache: VisualLineCache,
    /// Color configuration for log lines
    pub config: Option<ColorConfig>,
}

impl App {
    pub fn new() -> Self {
        let viewport_width = 80;
        Self {
            storage: None,
            filtered_indices: Vec::new(),
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
            viewport_width: Cell::new(viewport_width),
            visual_cache: VisualLineCache::new(10000, viewport_width),
            config: ColorConfig::load(),
        }
    }

    /// Get the number of filtered entries.
    pub fn filtered_len(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Get a line by its index in the storage.
    pub fn get_line(&self, idx: usize) -> Option<crate::model::MmapStr> {
        self.storage.as_ref()?.get_line(idx)
    }

    /// Get a filtered entry by its index in the filtered list.
    pub fn get_filtered_entry(&self, idx: usize) -> Option<crate::model::MmapStr> {
        self.filtered_indices
            .get(idx)
            .and_then(|&log_idx| self.get_line(log_idx))
    }

    /// Get the timestamp of a filtered entry.
    pub fn get_filtered_timestamp(&self, idx: usize) -> Option<chrono::DateTime<chrono::Utc>> {
        self.filtered_indices
            .get(idx)
            .and_then(|&log_idx| self.storage.as_ref()?.get_line_info(log_idx)?.timestamp)
    }

    /// Get the color for a log line based on configuration.
    ///
    /// Returns `None` if no config is loaded or no pattern matches.
    pub fn get_line_color(&self, line: &str) -> Option<Color> {
        self.config.as_ref()?.get_line_color(line)
    }

    /// Start async loading of logs.
    pub fn start_loading(&mut self) -> Sender<LogStorage> {
        let (tx, rx) = channel();
        self.log_receiver = Some(rx);
        self.loading_status = LoadingStatus::Loading {
            current: 0,
            total: 1,
        };
        tx
    }

    /// Check for loaded logs from the receiver.
    pub fn check_for_loaded_logs(&mut self) {
        if let Some(ref receiver) = self.log_receiver {
            while let Ok(storage) = receiver.try_recv() {
                self.storage = Some(storage);
                if let LoadingStatus::Loading { current, total } = self.loading_status {
                    self.loading_status = LoadingStatus::Loading {
                        current: current + 1,
                        total,
                    };
                }
            }
        }
    }

    /// Finish loading and update filtered logs.
    pub fn finish_loading(&mut self) {
        self.loading_status = LoadingStatus::Complete;
        self.log_receiver = None;
        self.update_filtered_logs();
    }

    /// Set the storage directly.
    pub fn set_storage(&mut self, storage: LogStorage) {
        self.storage = Some(storage);
        self.update_filtered_logs();
    }

    /// Update filtered indices based on current filters.
    /// Uses byte-based matching for zero-allocation filtering.
    pub fn update_filtered_logs(&mut self) {
        self.filtered_indices.clear();

        let Some(storage) = &self.storage else {
            return;
        };

        // Get viewport width for visual cache
        let viewport_width = self.viewport_width.get();
        if self.visual_cache.viewport_width() != viewport_width {
            self.visual_cache.set_viewport_width(viewport_width);
        }
        if self.visual_cache.wrap_mode() != self.wrap_mode {
            self.visual_cache.set_wrap_mode(self.wrap_mode);
        }

        // Filter using byte-based matching
        for (idx, mmap_str) in storage.iter_enumerated() {
            let line_bytes = mmap_str.as_bytes();
            if self.filters.matches(line_bytes) {
                self.filtered_indices.push(idx);
            }
        }

        // Clear visual cache since filtered indices changed
        self.visual_cache.clear();
    }

    /// Calculate visual line offsets for the current filtered view.
    /// Uses on-demand calculation with caching.
    pub fn calculate_visual_range(
        &mut self,
        start_idx: usize,
        count: usize,
    ) -> Vec<(usize, usize)> {
        self.filtered_indices
            .iter()
            .skip(start_idx)
            .take(count)
            .enumerate()
            .map(|(i, &line_idx)| {
                let visual_count = if let Some(storage) = &self.storage {
                    if let Some(line) = storage.get_line(line_idx) {
                        self.visual_cache
                            .calculate_visual_lines(&line.as_str_lossy())
                    } else {
                        1
                    }
                } else {
                    1
                };
                (line_idx, i * visual_count)
            })
            .collect()
    }

    /// Get the visual line offset for a specific filtered line index.
    pub fn selected_visual_line(&self) -> usize {
        self.scroll_offset + self.selected_line
    }

    /// Get the current scroll position in visual lines.
    pub fn scroll_visual_line(&self) -> usize {
        self.scroll_offset
    }

    /// Find the entry index by visual line number.
    pub fn find_entry_by_visual_line(&self, target_visual: usize) -> usize {
        // Simplified: just return the target or clamped
        target_visual.min(self.filtered_len().saturating_sub(1))
    }

    /// Ensure selection is valid after filter changes.
    fn ensure_valid_selection(&mut self) {
        if self.filters.groups().is_empty() {
            self.selected_group = 0;
            self.selected_filter = 0;
            return;
        }
        self.selected_group = self
            .selected_group
            .min(self.filters.groups().len().saturating_sub(1));
        if let Some(group) = self.filters.groups().get(self.selected_group) {
            if group.filters().is_empty() {
                self.selected_filter = 0;
            } else {
                self.selected_filter = self
                    .selected_filter
                    .min(group.filters().len().saturating_sub(1));
            }
        }
    }

    /// Handle keyboard input.
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
                        self.filters.add_group(crate::model::FilterGroup::new());
                        self.selected_group = self.filters.groups().len().saturating_sub(1);
                        self.selected_filter = 0;
                    }
                    if self.filters.groups().is_empty() {
                        self.filters.add_group(crate::model::FilterGroup::new());
                        self.selected_group = 0;
                        self.selected_filter = 0;
                    }
                    if let Some(group) = self.filters.groups_mut().get_mut(self.selected_group) {
                        group.add_filter(crate::model::Filter::new(self.input_buffer.trim()));
                    }
                    self.ensure_valid_selection();
                    self.selected_filter = self.filters.groups()[self.selected_group]
                        .filters()
                        .len()
                        .saturating_sub(1);
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
            "q" | "quit" => {
                self.should_quit = true;
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

        let Some(storage) = &self.storage else {
            return Ok(0);
        };

        for &idx in &self.filtered_indices {
            if let Some(line) = storage.get_line(idx) {
                writeln!(file, "{}", line.as_str_lossy())?;
                count += 1;
            }
        }

        Ok(count)
    }

    fn handle_normal_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.status_message.clear();
                if self.selected_line < self.filtered_len().saturating_sub(1) {
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
                self.selected_line = self.filtered_len().saturating_sub(1);
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
                self.visual_cache.set_wrap_mode(self.wrap_mode);
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
        if self.filtered_indices.is_empty() {
            return;
        }

        self.selected_line = self
            .selected_line
            .min(self.filtered_len().saturating_sub(1));

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
                if let Some(group) = self.filters.groups().get(self.selected_group) {
                    if self.selected_filter + 1 < group.filters().len() {
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
                if self.selected_group + 1 < self.filters.groups().len() {
                    self.selected_group += 1;
                    self.ensure_valid_selection();
                }
            }
            KeyCode::Char(' ') => {
                if let Some(group) = self.filters.groups_mut().get_mut(self.selected_group) {
                    if let Some(filter) = group.filters_mut().get_mut(self.selected_filter) {
                        filter.toggle();
                    }
                }
                self.update_filtered_logs();
            }
            KeyCode::Char('d') => {
                if !self.filters.groups().is_empty() {
                    if let Some(group) = self.filters.groups_mut().get_mut(self.selected_group) {
                        if self.selected_filter < group.filters().len() {
                            group.remove_filter(self.selected_filter);
                        }
                    }
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

    /// Get the visual cache (for UI rendering).
    pub fn visual_cache(&self) -> &VisualLineCache {
        &self.visual_cache
    }

    /// Get mutable visual cache.
    pub fn visual_cache_mut(&mut self) -> &mut VisualLineCache {
        &mut self.visual_cache
    }

    /// Get the total number of visual lines (approximation).
    pub fn total_visual_lines(&self) -> usize {
        // Return an estimate based on filtered count
        self.filtered_len()
    }

    /// Get the number of lines in storage.
    pub fn total_lines(&self) -> usize {
        self.storage.as_ref().map(|s| s.len()).unwrap_or(0)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_storage() -> LogStorage {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Line 1").unwrap();
        writeln!(temp_file, "Line 2").unwrap();
        writeln!(temp_file, "Line 3").unwrap();
        LogStorage::from_file(temp_file.path()).unwrap()
    }

    #[test]
    fn test_app_new() {
        let app = App::new();
        assert!(app.storage.is_none());
        assert!(app.filtered_indices.is_empty());
        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn test_set_storage() {
        let mut app = App::new();
        let storage = create_test_storage();

        app.set_storage(storage);

        assert!(app.storage.is_some());
        assert_eq!(app.total_lines(), 3);
    }

    #[test]
    fn test_get_line() {
        let mut app = App::new();
        let storage = create_test_storage();
        app.set_storage(storage);

        let line = app.get_line(0).unwrap();
        assert_eq!(line.as_str_lossy().trim(), "Line 1");

        let line = app.get_line(1).unwrap();
        assert_eq!(line.as_str_lossy().trim(), "Line 2");
    }

    #[test]
    fn test_parse_command() {
        assert_eq!(App::parse_command("write file.log"), ("write", "file.log"));
        assert_eq!(App::parse_command("w"), ("w", ""));
        assert_eq!(
            App::parse_command("  write   file.log  "),
            ("write", "file.log")
        );
    }
}
