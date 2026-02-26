use crate::clipboard::Clipboard;
use crate::command::{self, CommandEffect};
use crate::config::AppConfig;
use crate::key_bindings::{Mode, Msg};
use crate::model::{
    BMHMatcher, Direction, FilterKind, FilterList, LogStorage, Selection, VisualLineCache,
};
use lru::LruCache;
use ratatui::style::Color;
use std::cell::Cell;
use std::fs::File;
use std::io::Write;
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
    /// Active filters (command-based)
    pub filters: FilterList,
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
    /// Selected filter index in :list-filters view
    pub filter_list_selected: usize,
    /// Input buffer for text input
    pub input_buffer: String,
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
    /// Index for cycling through command completions (None when not completing)
    pub completion_index: Option<usize>,
    /// Original prefix for completion (stored to enable cycling)
    completion_prefix: String,
}

impl App {
    pub fn new() -> Self {
        let viewport_width = 80;
        Self {
            storage: None,
            filtered_indices: Vec::new(),
            filters: FilterList::new(),
            mode: Mode::Normal,
            should_quit: false,
            status_message: String::new(),
            scroll_offset: 0,
            horizontal_scroll: 0,
            selected_line: 0,
            loading_status: LoadingStatus::Idle,
            log_receiver: None,
            filter_list_selected: 0,
            input_buffer: String::new(),
            wrap_mode: true,
            viewport_height: Cell::new(20),
            viewport_width: Cell::new(viewport_width),
            visual_cache: VisualLineCache::new(10000, viewport_width),
            config: AppConfig::load(),
            search_query: None,
            search_state: None,
            selection: Selection::new(),
            clipboard: Clipboard::new().ok(),
            completion_index: None,
            completion_prefix: String::new(),
        }
    }

    /// Apply the next completion from the matching commands list.
    /// Cycles through matches and wraps around when reaching the end.
    pub fn apply_completion(&mut self) {
        if self.completion_index.is_none() {
            self.completion_prefix = self
                .input_buffer
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();
        }

        let idx = self.completion_index.map_or(0, |i| i + 1);

        if let Some((completed, new_idx)) = command::complete(&self.completion_prefix, idx) {
            self.completion_index = Some(new_idx);

            let args = self
                .input_buffer
                .split_once(' ')
                .map(|(_, rest)| format!(" {}", rest))
                .unwrap_or_default();

            self.input_buffer = format!("{}{}", completed, args);
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

    /// Handle keyboard input by translating to messages and processing them.
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        use crate::key_bindings::translate;
        if let Some(msg) = translate(key, self.mode) {
            self.process_message(msg);
        }
    }

    /// Process a message and update application state accordingly.
    fn process_message(&mut self, msg: Msg) {
        match msg {
            // Navigation
            Msg::ScrollDown => self.on_scroll_down(),
            Msg::ScrollUp => self.on_scroll_up(),
            Msg::ScrollRight => self.on_scroll_right(),
            Msg::ScrollLeft => self.on_scroll_left(),
            Msg::GoToBottom => self.on_go_to_bottom(),
            Msg::GoToTop => self.on_go_to_top(),

            // Command mode
            Msg::EnterCommand => self.on_enter_command(),
            Msg::CancelCommand => self.on_cancel_command(),
            Msg::SubmitCommand => self.on_submit_command(),
            Msg::CommandTypeChar(c) => self.on_command_type_char(c),
            Msg::CommandBackspace => self.on_command_backspace(),
            Msg::CommandComplete => self.on_command_complete(),

            // Search
            Msg::EnterSearch => self.on_enter_search(),
            Msg::CancelSearch => self.on_cancel_search(),
            Msg::SubmitSearch => self.on_submit_search(),
            Msg::SearchTypeChar(c) => self.on_search_type_char(c),
            Msg::SearchBackspace => self.on_search_backspace(),
            Msg::NextMatch => self.next_match(),
            Msg::PrevMatch => self.prev_match(),
            Msg::ClearSearch => self.on_clear_search(),

            // Selection
            Msg::ToggleSelection => self.on_toggle_selection(),
            Msg::YankSelection => self.on_yank(),
            Msg::ClearSelection => self.on_clear_selection(),

            // Filter list
            Msg::FilterListDown => self.on_filter_list_down(),
            Msg::FilterListUp => self.on_filter_list_up(),
            Msg::DeleteSelectedFilter => self.on_delete_selected_filter(),
            Msg::CloseFilterList => self.on_close_filter_list(),

            // View options
            Msg::ToggleWrap => self.on_toggle_wrap(),

            // Application
            Msg::Quit => self.should_quit = true,
            // Keys that don't map to an action in the current mode (e.g., unmapped keys in Normal mode)
            Msg::NoOp => {}
        }
    }

    // Navigation handlers

    fn on_scroll_down(&mut self) {
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

    fn on_scroll_up(&mut self) {
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

    fn on_scroll_right(&mut self) {
        self.horizontal_scroll = self.horizontal_scroll.saturating_add(4);
    }

    fn on_scroll_left(&mut self) {
        self.horizontal_scroll = self.horizontal_scroll.saturating_sub(4);
    }

    fn on_go_to_bottom(&mut self) {
        self.selected_line = self.filtered_len().saturating_sub(1);
        self.clamp_scroll();
    }

    fn on_go_to_top(&mut self) {
        self.selected_line = 0;
        self.scroll_offset = 0;
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

    // Command mode handlers

    fn on_enter_command(&mut self) {
        self.mode = Mode::Command;
    }

    fn on_cancel_command(&mut self) {
        self.mode = Mode::Normal;
        self.input_buffer.clear();
    }

    fn on_submit_command(&mut self) {
        self.mode = self.execute_command();
        self.input_buffer.clear();
    }

    fn on_command_type_char(&mut self, c: char) {
        self.completion_index = None;
        self.completion_prefix.clear();
        self.input_buffer.push(c);
    }

    fn on_command_backspace(&mut self) {
        self.completion_index = None;
        self.completion_prefix.clear();
        self.input_buffer.pop();
    }

    fn on_command_complete(&mut self) {
        self.apply_completion();
    }

    fn execute_command(&mut self) -> Mode {
        let result = command::parse(&self.input_buffer);
        self.status_message = result.status;

        if let Some(effect) = result.effect {
            match effect {
                CommandEffect::Quit => {
                    self.should_quit = true;
                }
                CommandEffect::AddFilter { kind, pattern } => {
                    match kind {
                        FilterKind::Include => self.filters.add_include(&pattern),
                        FilterKind::Exclude => self.filters.add_exclude(&pattern),
                    }
                    self.update_filtered_logs();
                }
                CommandEffect::ClearFilters => {
                    self.filters.clear();
                    self.update_filtered_logs();
                }
                CommandEffect::WriteFilteredLogs { filename } => {
                    match self.write_filtered_logs(&filename) {
                        Ok(count) => {
                            self.status_message = format!("Saved {} lines to {}", count, filename);
                        }
                        Err(e) => {
                            self.status_message = format!("Error: {}", e);
                        }
                    }
                }
                CommandEffect::ListFilters => {
                    self.filter_list_selected = 0;
                    return Mode::FilterList;
                }
            }
        }
        Mode::Normal
    }

    fn write_filtered_logs(&self, filename: &str) -> std::io::Result<usize> {
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

    // Search handlers

    fn on_enter_search(&mut self) {
        self.mode = Mode::SearchInput;
        // Pre-populate with last search query if exists
        if let Some(last_query) = &self.search_query {
            self.input_buffer = last_query.clone();
        } else {
            self.input_buffer.clear();
        }
    }

    fn on_cancel_search(&mut self) {
        self.mode = Mode::Normal;
        self.input_buffer.clear();
    }

    fn on_submit_search(&mut self) {
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

    fn on_search_type_char(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    fn on_search_backspace(&mut self) {
        self.input_buffer.pop();
    }

    fn on_clear_search(&mut self) {
        self.clear_search();
        self.status_message.clear();
    }

    // Selection handlers

    fn on_toggle_selection(&mut self) {
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

    fn on_clear_selection(&mut self) {
        self.selection.clear();
        self.status_message.clear();
    }

    fn on_yank(&mut self) {
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

    // Filter list handlers

    fn on_filter_list_down(&mut self) {
        let total = self.filters.len();
        if self.filter_list_selected + 1 < total {
            self.filter_list_selected += 1;
        }
    }

    fn on_filter_list_up(&mut self) {
        self.filter_list_selected = self.filter_list_selected.saturating_sub(1);
    }

    fn on_delete_selected_filter(&mut self) {
        let includes = self.filters.includes().len();
        if self.filter_list_selected < includes {
            self.filters.remove_include(self.filter_list_selected);
        } else {
            self.filters
                .remove_exclude(self.filter_list_selected - includes);
        }
        // Ensure selection stays valid after deletion
        let total = self.filters.len();
        if self.filter_list_selected >= total && total > 0 {
            self.filter_list_selected = total - 1;
        }
        self.update_filtered_logs();
        self.clear_search_on_refilter();
        if self.filters.is_empty() {
            self.mode = Mode::Normal;
        }
    }

    fn on_close_filter_list(&mut self) {
        self.mode = Mode::Normal;
    }

    // View option handlers

    fn on_toggle_wrap(&mut self) {
        self.wrap_mode = !self.wrap_mode;
        self.visual_cache.set_wrap_mode(self.wrap_mode);
        self.status_message = if self.wrap_mode {
            "Wrap mode enabled".to_string()
        } else {
            "Wrap mode disabled".to_string()
        };
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
