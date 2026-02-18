## 1. Data Model

- [x] 1.1 Create new `Filter` struct with `text: String` and `enabled: bool` in `src/model/filter.rs`
- [x] 1.2 Create `FilterGroup` struct with `filters: Vec<Filter>`
- [x] 1.3 Update `FilterSet` to use `groups: Vec<FilterGroup>` instead of flat filters
- [x] 1.4 Implement `Filter::matches(&self, line: &str) -> bool` with case-insensitive contains
- [x] 1.5 Implement `FilterGroup::matches(&self, line: &str) -> bool` with OR logic
- [x] 1.6 Implement `FilterSet::matches(&self, entry: &LogEntry) -> bool` with AND logic between groups
- [x] 1.7 Simplify `LogEntry` to `raw: String` and `timestamp: Option<DateTime<Utc>>`
- [x] 1.8 Remove `LogLevel` enum and related match methods from `LogEntry`

## 2. Timestamp Detection

- [x] 2.1 Create `src/model/timestamp.rs` module for detection logic
- [x] 2.2 Implement `detect_timestamp(line: &str) -> Option<DateTime<Utc>>` trying common formats
- [x] 2.3 Add unit tests for timestamp detection (ISO 8601, common log formats, no timestamp)

## 3. Loader Updates

- [x] 3.1 Update `LogLoader` to create `LogEntry` with raw line text
- [x] 3.2 Call `detect_timestamp` during load, store result in entry
- [x] 3.3 Remove JSON parsing requirement from loader
- [x] 3.4 Update sorting to handle entries without timestamp (file order fallback)

## 4. Filter UI State

- [x] 4.1 Add `selected_group: usize` and `selected_filter: usize` to app state
- [x] 4.2 Add `input_mode: InputMode` enum (Normal, FilterInput)
- [x] 4.3 Add `input_buffer: String` for command-line text entry
- [x] 4.4 Add `pending_new_group: bool` flag for Shift+f behavior

## 5. Filter Bar Rendering

- [x] 5.1 Render filter groups with visual separators (`|` between groups)
- [x] 5.2 Show each filter as a chip with text content
- [x] 5.3 Highlight selected filter differently
- [x] 5.4 Render disabled filters with muted/gray styling
- [x] 5.5 Show command-line input at bottom when in FilterInput mode

## 6. Keybindings

- [x] 6.1 Implement `f` to enter FilterInput mode (add to current group)
- [x] 6.2 Implement `Shift+f` to create new group then enter FilterInput mode
- [x] 6.3 Implement `Enter` to confirm filter text and add to appropriate group
- [x] 6.4 Implement `Esc` to cancel input mode
- [x] 6.5 Implement `d` to delete selected filter (remove empty groups)
- [x] 6.6 Implement `Space` to toggle selected filter enabled state
- [x] 6.7 Implement `j/k` to move selection within current group
- [x] 6.8 Implement `h/l` to move selection to prev/next group

## 7. Integration

- [x] 7.1 Remove auto-filters from `App::new()` initialization
- [x] 7.2 Wire up `FilterSet::matches` in log filtering logic
- [x] 7.3 Update log list rendering to use `entry.raw` instead of formatted fields

## 8. Cleanup

- [x] 8.1 Remove old `Filter` enum variants (Level, Text, DateRange, SourceContext)
- [x] 8.2 Remove old `matches_level`, `matches_text` methods from LogEntry
- [x] 8.3 Remove JSON serde derives from LogEntry if no longer needed
- [x] 8.4 Update or remove outdated README documentation about filters
