## Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Search Mode Architecture                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   App Config (Unified)                                           │
│   ├─ ColorConfig (existing patterns → colors)                   │
│   └─ SearchConfig (NEW: match/current highlight settings)       │
│                                                                  │
│   App State                                                      │
│   ├─ mode: Normal | Filter | FilterInput | SearchInput | ...    │
│   ├─ search_query: Option<String>                               │
│   ├─ search_state: Option<SearchState>                          │
│   │   ├─ matcher: BMHMatcher                                    │
│   │   ├─ current_match: Option<(usize, usize)>                  │
│   │   └─ match_cache: LruCache<usize, Vec<(usize, usize)>>     │
│   ├─ horizontal_scroll: usize                                   │
│   └─ (existing fields)                                          │
│                                                                  │
│   Key Handlers                                                   │
│   ├─ handle_normal_key: '/', 'n', 'N' bindings                  │
│   └─ handle_search_input_key: typing, Enter, Esc, Backspace    │
│                                                                  │
│   UI Rendering                                                   │
│   ├─ draw_search_input: bottom input box like command mode      │
│   └─ draw_main_view: highlight matches in log lines             │
│       ├─ Normal text: log color from config                     │
│       ├─ All matches: search.match_fg/bg + match_style         │
│       └─ Current match: search.current_fg/bg + current_style   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Data Structures

#### SearchConfig
```rust
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Foreground color for all match highlights
    pub match_fg: Color,
    /// Background color for all match highlights
    pub match_bg: Color,
    /// Style modifiers for all matches (bold, underline, reverse)
    pub match_style: Style,
    /// Foreground color for the current (active) match
    pub current_fg: Color,
    /// Background color for the current (active) match
    pub current_bg: Color,
    /// Style modifiers for current match
    pub current_style: Style,
}
```

#### AppConfig (Unified)
```rust
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Existing log line color configuration
    pub colors: ColorConfig,
    /// NEW: Search highlight configuration
    pub search: SearchConfig,
}
```

#### SearchState
```rust
#[derive(Debug)]
pub struct SearchState {
    /// The search query string (lowercase for case-insensitive matching)
    pub query: String,
    /// BMH matcher for efficient searching
    pub matcher: BMHMatcher,
    /// Index of the current match in the flattened match list
    pub current_idx: usize,
    /// Cache of matches per line index (filtered_indices index)
    /// Key: filtered line index, Value: Vec of (byte_start, byte_end)
    pub match_cache: LruCache<usize, Vec<(usize, usize)>>,
}
```

### State Management

#### Refresh Strategy (Event-Driven)
Instead of direct coupling between filters and search, use explicit refresh events:

```rust
pub enum RefreshKind {
    /// Rebuild filtered_indices from storage + filters
    Refilter,
    /// Rebuild search matches for new query
    Research,
}

pub struct App {
    pending_refresh: Option<RefreshKind>,
    // ...
}
```

Main loop (run_app) handles cascade:
```rust
if let Some(kind) = app.pending_refresh.take() {
    match kind {
        RefreshKind::Refilter => {
            app.update_filtered_indices();
            // Automatic cascade: clear search state
            app.search_query = None;
            app.search_state = None;
        }
        RefreshKind::Research => {
            if let Some(query) = &app.search_query {
                app.init_search_state(query.clone());
            }
        }
    }
}
```

### Search Algorithm

#### Lazy Match Finding
Only search visible lines plus a small margin to avoid memory explosion on large datasets:

1. **Match finding per line**: Use BMHMatcher with `find_all()` extension
2. **Caching**: LRU cache stores matches for recently accessed lines
3. **Cache key**: `usize` (index into filtered_indices)
4. **Cache value**: `Vec<(usize, usize)>` (byte start/end positions)

#### BMHMatcher.find_all() Extension
```rust
impl BMHMatcher {
    /// Find all match positions in text (not just first)
    /// Returns vector of (start, end) byte positions
    pub fn find_all(&self, text: &[u8]) -> Vec<(usize, usize)> {
        let mut matches = Vec::new();
        let pattern_len = self.pattern.len();
        
        // O(n×m) scan is fine since we only scan visible lines (~50)
        for i in 0..=text.len().saturating_sub(pattern_len) {
            if self.matches_at(text, i) {
                matches.push((i, i + pattern_len));
            }
        }
        matches
    }
}
```

### UI Rendering

#### Match Highlighting in Log Lines
For each visible line:
1. Get line text from storage
2. Check cache for match positions
3. If not cached, compute matches and store in cache
4. Split line into spans around match boundaries
5. Apply styles:
   - Regular text: log color from ColorConfig + selection bg if selected
   - Match spans: SearchConfig.match_fg/bg + match_style
   - Current match: SearchConfig.current_fg/bg + current_style (overrides)

#### Horizontal Auto-Scroll
When navigating to a match:
```rust
fn jump_to_match(&mut self, match_idx: usize) {
    // Update current match
    self.search_state.current_idx = match_idx;
    
    // Get match position
    let (filtered_idx, byte_offset, match_len) = self.get_match_position(match_idx);
    
    // Vertical scroll (existing logic)
    self.selected_line = filtered_idx;
    self.clamp_scroll();
    
    // Horizontal auto-scroll
    let match_char_pos = self.byte_to_char_offset(filtered_idx, byte_offset);
    let viewport_width = self.viewport_width.get();
    let margin = 10;
    
    if match_char_pos < self.horizontal_scroll {
        // Match is left of viewport
        self.horizontal_scroll = match_char_pos;
    } else if match_char_pos + match_len > self.horizontal_scroll + viewport_width - margin {
        // Match is right of viewport
        self.horizontal_scroll = match_char_pos.saturating_sub(viewport_width - margin);
    }
}
```

## Interface Definitions

### App Public API Additions
```rust
impl App {
    /// Initialize or update search state with new query
    pub fn init_search_state(&mut self, query: String);
    
    /// Clear current search state
    pub fn clear_search(&mut self);
    
    /// Navigate to next match (n key)
    pub fn next_match(&mut self);
    
    /// Navigate to previous match (N key)
    pub fn prev_match(&mut self);
    
    /// Get matches for a specific line (with caching)
    pub fn get_line_matches(&mut self, filtered_idx: usize) -> Vec<(usize, usize)>;
    
    /// Get total match count
    pub fn total_matches(&self) -> usize;
    
    /// Get current match index (for status display)
    pub fn current_match_display(&self) -> Option<String>; // "3/42"
}
```

### UI Module Additions
```rust
/// Draw search input box (similar to command input)
pub fn draw_search_input(frame: &mut Frame, app: &App, area: Rect);

/// Check if line has current match (for styling)
pub fn is_current_match(app: &App, filtered_idx: usize, byte_offset: usize) -> bool;
```

## Data Flow

### Search Flow
```
User presses '/'
    │
    ▼
Mode changes to SearchInput
    │
    ▼
User types query, presses Enter
    │
    ▼
init_search_state() called
    │
    ├─ Create BMHMatcher from query (lowercase)
    ├─ Set current_idx = 0
    ├─ Clear match cache
    │
    ▼
Mode returns to Normal
    │
    ▼
UI renders with highlighted matches
    │
    ▼
User presses 'n' or 'N'
    │
    ▼
next_match() or prev_match() called
    │
    ├─ Update current_idx with wrap-around
    ├─ Auto-scroll horizontally to match
    │
    ▼
UI re-renders with updated current match
```

### Filter Change Flow
```
Filter modified (add/toggle/delete)
    │
    ▼
pending_refresh = Some(Refilter)
    │
    ▼
Main loop processes refresh
    │
    ├─ Rebuild filtered_indices
    ├─ search_query = None (clear)
    ├─ search_state = None (clear)
    │
    ▼
No stale search state remains
```

## Testing Strategy

### Unit Tests
- `SearchConfig::default()` returns expected defaults
- `BMHMatcher::find_all()` finds all occurrences including overlapping
- `SearchState` cache hit/miss behavior
- `next_match()`/`prev_match()` wrap-around logic
- Horizontal auto-scroll calculations

### Integration Tests
- Full search workflow: / → type query → Enter → n → N → Esc
- Filter change clears search state
- Match highlighting renders correctly in UI
- Case-insensitive matching works
- Configurable colors load from TOML

## Configuration

### TOML Format
```toml
[colors]
"*ERROR*" = "red"
"*WARN*" = "yellow"
"*INFO*" = "green"

[search]
match_fg = "black"
match_bg = "yellow"
match_style = "bold"
current_fg = "black"
current_bg = "light_yellow"
current_style = "bold"
```

### Default Values
- `match_fg`: Color::Black
- `match_bg`: Color::Yellow
- `match_style`: Style::default().add_modifier(Modifier::BOLD)
- `current_fg`: Color::Black
- `current_bg`: Color::LightYellow
- `current_style`: Style::default().add_modifier(Modifier::BOLD)
