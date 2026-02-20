use crate::clipboard::Clipboard;
use crate::config::AppConfig;
use crate::model::{BMHMatcher, Direction, FilterSet, LogStorage, Selection, VisualLineCache};
use chrono::Local;
use crossterm::event::KeyCode;
use lru::LruCache;
use ratatui::style::Color;
use std::cell::Cell;
use std::fs::File;
use std::io::{self, Write};
use std::num::NonZeroUsize;
use std::sync::mpsc::{channel, Receiver, Sender};

/// Position of a match for O(1) lookup.
#[derive(Debug, Clone, Copy)]
pub struct MatchPosition {
    /// Index into filtered_indices
    pub filtered_idx: usize,
    /// Byte offset in the line
    pub byte_offset: usize,
    /// Match length in bytes
    pub match_len: usize,
}

/// Search state with LRU cache for line matches.
#[derive(Debug)]
pub struct SearchState {
    /// The search query string (lowercase for case-insensitive matching)
    pub query: String,
    /// BMH matcher for efficient searching
    pub matcher: BMHMatcher,
    /// Index of the current match in the flattened match list
    pub current_idx: usize,
    /// Position of the current match for O(1) lookup
    pub current_position: Option<MatchPosition>,
    /// Total number of matches (cached for performance)
    pub total_matches: usize,
    /// Cache of matches per line index (filtered_indices index)
    /// Key: filtered line index, Value: Vec of (byte_start, byte_end)
    pub match_cache: LruCache<usize, Vec<(usize, usize)>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Filter,
    FilterInput,
    Command,
    DateRange,
    SearchInput,
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
    /// Application configuration (colors + search)
    pub config: Option<AppConfig>,
    /// Current search query string
    pub search_query: Option<String>,
    /// Search state with matcher and cache
    pub search_state: Option<SearchState>,
    /// Active selection for Helix-style line selection
    pub selection: Selection,
    /// System clipboard wrapper (may be None on headless systems)
    pub clipboard: Option<Clipboard>,
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
            config: AppConfig::load(),
            search_query: None,
            search_state: None,
            selection: Selection::new(),
            clipboard: Clipboard::new().ok(),
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
        self.config.as_ref()?.colors.get_line_color(line)
    }

    /// Get the search configuration.
    pub fn search_config(&self) -> Option<&crate::config::SearchConfig> {
        self.config.as_ref().map(|c| &c.search)
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

        // Clear selection since filter indices are now invalid
        self.selection.clear();
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
            Mode::SearchInput => self.handle_search_input_key(key),
        }
    }

    fn handle_search_input_key(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.input_buffer.clear();
            }
            KeyCode::Enter => {
                if self.input_buffer.trim().is_empty() {
                    // Empty query clears search
                    self.clear_search();
                } else {
                    // Execute search with non-empty query
                    let query = self.input_buffer.trim().to_string();
                    self.search_query = Some(query.clone());
                    self.init_search_state(query);
                }
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
                    self.clear_search_on_refilter();
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
                    let old_line = self.selected_line;
                    self.selected_line += 1;
                    // Extend selection if active
                    if self.selection.is_active() {
                        let direction = if self.selected_line > old_line {
                            Direction::Down
                        } else {
                            Direction::Up
                        };
                        self.selection.extend(self.selected_line, direction);
                    }
                }
                self.clamp_scroll();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.status_message.clear();
                let old_line = self.selected_line;
                self.selected_line = self.selected_line.saturating_sub(1);
                // Extend selection if active
                if self.selection.is_active() {
                    let direction = if self.selected_line < old_line {
                        Direction::Up
                    } else {
                        Direction::Down
                    };
                    self.selection.extend(self.selected_line, direction);
                }
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
            KeyCode::Char('x') => {
                if self.selection.is_active() {
                    // Extend selection - determine direction based on cursor movement
                    if let Some((start, _end)) = self.selection.range(self.selected_line) {
                        let direction = if self.selected_line > start {
                            Direction::Down
                        } else {
                            Direction::Up
                        };
                        self.selection.extend(self.selected_line, direction);
                    }
                } else {
                    // Start new selection at current cursor
                    self.selection.start(self.selected_line);
                }
            }
            KeyCode::Char('y') => {
                self.handle_yank();
            }
            KeyCode::Esc => {
                self.selection.clear();
                self.status_message.clear();
            }
            KeyCode::Char('/') => {
                self.mode = Mode::SearchInput;
                // Pre-populate with last search query if exists
                if let Some(last_query) = &self.search_query {
                    self.input_buffer = last_query.clone();
                } else {
                    self.input_buffer.clear();
                }
            }
            KeyCode::Char('n') => {
                self.next_match();
            }
            KeyCode::Char('N') => {
                self.prev_match();
            }
            _ => {}
        }
    }

    /// Handle yank (copy) operation
    fn handle_yank(&mut self) {
        // Check if selection is active
        if !self.selection.is_active() {
            return;
        }

        // Check if clipboard is available
        let Some(ref mut clipboard) = self.clipboard else {
            self.status_message = "Clipboard unavailable - install display server".to_string();
            return;
        };

        // Get the selection range
        let Some((start, end)) = self.selection.range(self.selected_line) else {
            return;
        };

        // Retrieve the raw lines from storage
        let Some(ref storage) = self.storage else {
            return;
        };

        let mut lines = Vec::new();
        for idx in start..=end {
            if let Some(&storage_idx) = self.filtered_indices.get(idx) {
                if let Some(line) = storage.get_line(storage_idx) {
                    lines.push(line.as_str_lossy().to_string());
                }
            }
        }

        if lines.is_empty() {
            return;
        }

        // Join lines with newline
        let text = lines.join("\n");

        // Copy to clipboard
        match clipboard.copy(&text) {
            Ok(()) => {
                self.status_message = format!("Copied {} lines to clipboard", lines.len());
            }
            Err(e) => {
                self.status_message = format!("Failed to copy: {}", e);
            }
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
                self.clear_search_on_refilter();
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
                    self.clear_search_on_refilter();
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

    /// Clear search when filters change.
    fn clear_search_on_refilter(&mut self) {
        self.search_query = None;
        self.search_state = None;
    }

    /// Initialize search state with a query.
    pub fn init_search_state(&mut self, query: String) {
        if query.is_empty() {
            self.clear_search();
            return;
        }
        let lower_query = query.to_lowercase();
        let pattern_bytes = lower_query.bytes().collect::<Vec<u8>>();
        let matcher = BMHMatcher::new(pattern_bytes);

        // Compute total matches and first match position (before creating SearchState)
        let (total, first_position) = self.compute_total_matches(&matcher);

        // Create the search state with cached values
        let state = SearchState {
            query: lower_query,
            matcher,
            current_idx: 0,
            current_position: first_position,
            total_matches: total,
            match_cache: LruCache::new(NonZeroUsize::new(100).unwrap()),
        };
        self.search_state = Some(state);
        self.search_query = Some(query);

        // Navigate to first match if any
        if total > 0 {
            self.jump_to_match(0);
        }
    }

    /// Compute total matches and optionally first match position.
    fn compute_total_matches(&self, matcher: &BMHMatcher) -> (usize, Option<MatchPosition>) {
        let Some(storage) = &self.storage else {
            return (0, None);
        };

        let mut total = 0;
        let mut first_position = None;

        for (filtered_idx, &line_idx) in self.filtered_indices.iter().enumerate() {
            let Some(line) = storage.get_line(line_idx) else {
                continue;
            };
            let lower_bytes: Vec<u8> = line
                .as_bytes()
                .iter()
                .map(|&b| b.to_ascii_lowercase())
                .collect();
            let matches = matcher.find_all(&lower_bytes);

            for (start, end) in &matches {
                if first_position.is_none() {
                    first_position = Some(MatchPosition {
                        filtered_idx,
                        byte_offset: *start,
                        match_len: end - start,
                    });
                }
                total += 1;
            }
        }

        (total, first_position)
    }

    /// Recompute total matches when filters change (but keep search query).
    fn recompute_search_matches(&mut self) {
        // Extract matcher reference first to avoid borrow issues
        let matcher_ref = if let Some(state) = &self.search_state {
            &state.matcher
        } else {
            return;
        };

        let (total, first_position) = self.compute_total_matches(matcher_ref);

        // Now update the state with the computed values
        if let Some(state) = &mut self.search_state {
            state.total_matches = total;
            state.current_idx = 0;
            state.current_position = first_position;
            state.match_cache.clear();
        }
    }

    /// Clear search state.
    pub fn clear_search(&mut self) {
        self.search_query = None;
        self.search_state = None;
    }

    /// Get matches for a specific line (with caching).
    pub fn get_line_matches(&mut self, filtered_idx: usize) -> Vec<(usize, usize)> {
        let Some(state) = &mut self.search_state else {
            return Vec::new();
        };

        // Check cache first
        if let Some(matches) = state.match_cache.get(&filtered_idx) {
            return matches.clone();
        }

        // Get the line text
        let Some(storage) = &self.storage else {
            return Vec::new();
        };
        let Some(&line_idx) = self.filtered_indices.get(filtered_idx) else {
            return Vec::new();
        };
        let Some(line) = storage.get_line(line_idx) else {
            return Vec::new();
        };

        // Convert line to lowercase bytes for case-insensitive matching
        let lower_bytes: Vec<u8> = line
            .as_bytes()
            .iter()
            .map(|&b| b.to_ascii_lowercase())
            .collect();

        // Find all matches
        let matches = state.matcher.find_all(&lower_bytes);

        // Cache the result (clone for return value, original goes into cache)
        let result = matches.clone();
        state.match_cache.put(filtered_idx, matches);

        result
    }

    /// Get total match count across all filtered lines.
    /// Returns cached value for O(1) performance.
    pub fn total_matches(&self) -> usize {
        self.search_state
            .as_ref()
            .map(|s| s.total_matches)
            .unwrap_or(0)
    }

    /// Get current match display string (e.g., "3/42").
    pub fn current_match_display(&self) -> Option<String> {
        let state = self.search_state.as_ref()?;
        if state.total_matches == 0 {
            return None;
        }
        Some(format!("{}/{}", state.current_idx + 1, state.total_matches))
    }

    /// Navigate to next match (with wrap-around).
    pub fn next_match(&mut self) {
        let total = self.total_matches();
        if total == 0 {
            return;
        }

        let next_idx = if let Some(state) = &self.search_state {
            (state.current_idx + 1) % total
        } else {
            0
        };

        self.jump_to_match(next_idx);
    }

    /// Navigate to previous match (with wrap-around).
    pub fn prev_match(&mut self) {
        let total = self.total_matches();
        if total == 0 {
            return;
        }

        let prev_idx = if let Some(state) = &self.search_state {
            if state.current_idx == 0 {
                total - 1
            } else {
                state.current_idx - 1
            }
        } else {
            0
        };

        self.jump_to_match(prev_idx);
    }

    /// Jump to a specific match by index.
    fn jump_to_match(&mut self, match_idx: usize) {
        let Some(state) = &mut self.search_state else {
            return;
        };

        if state.total_matches == 0 || match_idx >= state.total_matches {
            return;
        }

        // Update current index
        state.current_idx = match_idx;

        // Find the position of this match
        if let Some(position) = self.get_match_position(match_idx) {
            // Cache the position
            if let Some(ref mut state) = self.search_state {
                state.current_position = Some(position);
            }

            // Update vertical position
            self.selected_line = position.filtered_idx;
            self.clamp_scroll();

            // Horizontal auto-scroll
            let Some(storage) = &self.storage else { return };
            let Some(&line_idx) = self.filtered_indices.get(position.filtered_idx) else {
                return;
            };
            let Some(line) = storage.get_line(line_idx) else {
                return;
            };
            let line_text = line.as_str_lossy();

            // Calculate character offset from byte offset
            let match_char_pos = byte_to_char_offset(&line_text, position.byte_offset);
            // Calculate match length in characters (not bytes) for consistent scroll math
            let match_text =
                &line_text[position.byte_offset..position.byte_offset + position.match_len];
            let match_char_len = match_text.chars().count();

            let viewport_width = self.viewport_width.get();
            let margin = 10;

            // Only adjust horizontal scroll if match is outside viewport
            if match_char_pos < self.horizontal_scroll {
                // Match is left of viewport - scroll to show it with margin
                self.horizontal_scroll = match_char_pos.saturating_sub(margin);
            } else if match_char_pos + match_char_len
                > self.horizontal_scroll + viewport_width.saturating_sub(margin)
            {
                // Match is right of viewport - scroll to show it
                self.horizontal_scroll =
                    (match_char_pos + match_char_len + margin).saturating_sub(viewport_width);
            }
        }
    }

    /// Get the position of a match by its global index.
    fn get_match_position(&self, match_idx: usize) -> Option<MatchPosition> {
        let state = self.search_state.as_ref()?;
        let storage = self.storage.as_ref()?;

        let mut current_match = 0;

        for (filtered_idx, &line_idx) in self.filtered_indices.iter().enumerate() {
            let line = storage.get_line(line_idx)?;
            let lower_bytes: Vec<u8> = line
                .as_bytes()
                .iter()
                .map(|&b| b.to_ascii_lowercase())
                .collect();
            let matches = state.matcher.find_all(&lower_bytes);

            for (start, end) in matches {
                if current_match == match_idx {
                    return Some(MatchPosition {
                        filtered_idx,
                        byte_offset: start,
                        match_len: end - start,
                    });
                }
                current_match += 1;
            }
        }

        None
    }

    /// Check if a specific position is the current match.
    /// Uses O(1) cached position lookup instead of linear scan.
    pub fn is_current_match(&self, filtered_idx: usize, byte_offset: usize) -> bool {
        let Some(state) = &self.search_state else {
            return false;
        };

        // Use cached position for O(1) lookup
        if let Some(pos) = state.current_position {
            return pos.filtered_idx == filtered_idx && pos.byte_offset == byte_offset;
        }

        false
    }

    /// Check if there is an active search.
    pub fn has_search(&self) -> bool {
        self.search_state.is_some()
    }

    /// Get the search query if any.
    pub fn get_search_query(&self) -> Option<&str> {
        self.search_query.as_deref()
    }
}

/// Convert byte offset to character offset in a string.
/// Safely handles multi-byte UTF-8 characters by using char_indices.
fn byte_to_char_offset(text: &str, byte_offset: usize) -> usize {
    text.char_indices()
        .take_while(|(idx, _)| *idx < byte_offset)
        .count()
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

    #[test]
    fn test_search_init_and_clear() {
        let mut app = App::new();

        // Initially no search
        assert!(!app.has_search());
        assert_eq!(app.get_search_query(), None);

        // Initialize search
        app.init_search_state("test".to_string());

        // Now has search
        assert!(app.has_search());
        assert_eq!(app.get_search_query(), Some("test"));

        // Clear search
        app.clear_search();

        // No search anymore
        assert!(!app.has_search());
        assert_eq!(app.get_search_query(), None);
    }

    #[test]
    fn test_search_matches() {
        let mut app = App::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "This is a test line").unwrap();
        writeln!(temp_file, "Another test line").unwrap();
        writeln!(temp_file, "No match here").unwrap();
        let storage = LogStorage::from_file(temp_file.path()).unwrap();
        app.set_storage(storage);

        // Initialize search for "test"
        app.init_search_state("test".to_string());

        // Check total matches (should be 2)
        assert_eq!(app.total_matches(), 2);

        // Get line matches for line 0
        let matches = app.get_line_matches(0);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], (10, 14)); // "test" position in "This is a test line"

        // Get line matches for line 1
        let matches = app.get_line_matches(1);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0], (8, 12)); // "test" position in "Another test line"

        // Get line matches for line 2 (no matches)
        let matches = app.get_line_matches(2);
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut app = App::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "ERROR message").unwrap();
        writeln!(temp_file, "error message").unwrap();
        writeln!(temp_file, "Error message").unwrap();
        let storage = LogStorage::from_file(temp_file.path()).unwrap();
        app.set_storage(storage);

        // Search for lowercase "error" - should match all variations
        app.init_search_state("error".to_string());

        // Each line should have one match
        assert_eq!(app.get_line_matches(0).len(), 1);
        assert_eq!(app.get_line_matches(1).len(), 1);
        assert_eq!(app.get_line_matches(2).len(), 1);
    }

    #[test]
    fn test_search_filter_clears_search() {
        let mut app = App::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test line 1").unwrap();
        writeln!(temp_file, "test line 2").unwrap();
        let storage = LogStorage::from_file(temp_file.path()).unwrap();
        app.set_storage(storage);

        // Set up a search
        app.init_search_state("test".to_string());
        assert!(app.has_search());

        // Simulate filter change - clear search
        app.clear_search_on_refilter();

        // Search should be cleared
        assert!(!app.has_search());
        assert_eq!(app.get_search_query(), None);
    }

    #[test]
    fn test_byte_to_char_offset() {
        assert_eq!(byte_to_char_offset("hello", 0), 0);
        assert_eq!(byte_to_char_offset("hello", 5), 5);
        assert_eq!(byte_to_char_offset("hello world", 6), 6);
        assert_eq!(byte_to_char_offset("", 0), 0);
    }
}
