# qlog

A TUI (Terminal User Interface) application for viewing and filtering como-data-center log files with Helix-style keybindings.

## Features

- **Unified Log View**: Merge all log files chronologically with auto-detected timestamps
- **Filters**: Include/exclude filters with case-insensitive substring matching
- **Search**: Incremental search with `n`/`N` navigation and match highlighting
- **Selection & Yank**: Select lines with `x` and copy to clipboard with `y`
- **Helix Keybindings**: Modal editing with hjkl navigation and `:` command mode
- **Generic Log Support**: Works with any text log format (not just JSON)
- **Async Loading**: Efficient loading for large datasets
- **Virtual Scrolling**: Handle millions of lines with only visible lines rendered
- **Horizontal Scroll**: View wide log content with wrap mode toggle
- **Configurable Log Coloring**: Customize line colors via TOML config files
- **Export**: Save filtered results to file with `:write` command

## Installation

```bash
cd qlog
cargo build --release
```

The binary will be at `target/release/qlog`.

## Usage

```bash
# Automatically find *.log files in current directory
./qlog

# Or specify specific log files
./qlog /path/to/*.log
```

## Keybindings

### Navigation (Normal Mode)
- `j/k` or `Arrow Up/Down` - Scroll through logs
- `h/l` or `Arrow Left/Right` - Horizontal scroll
- `g` - Go to top
- `G` - Go to bottom
- `w` - Toggle wrap mode
- `:` - Enter command mode
- `/` - Enter search mode
- `n` - Next search match
- `N` - Previous search match
- `x` - Start/extend line selection
- `y` - Yank (copy) selected lines to clipboard
- `Esc` - Clear selection
- `q` - Quit application (or `:q` / `:quit` in command mode)

### Command Mode (`:`)
- `filter <text>` - Add include filter
- `filter-out <text>` - Add exclude filter
- `filter-clear` - Clear all filters
- `list-filters` - Show filter list view
- `write [filename]` or `w [filename]` - Save filtered logs to file
- `quit` or `q` - Quit application
- `Enter` - Execute command
- `Esc` - Cancel and return to normal mode
- `Backspace` - Delete character

### Filter List Mode
- `j/k` or `Arrow Up/Down` - Select filter
- `d` - Delete selected filter
- `Enter` / `Esc` / `q` - Return to normal mode

### Search Input Mode (`/`)
- `Enter` - Execute search
- `Esc` - Cancel and return to normal mode
- `Backspace` - Delete character

## Filters

qlog supports include and exclude filters:

- **Include filters**: Lines must contain at least one include filter text (OR logic between multiple includes)
- **Exclude filters**: Lines containing any exclude filter text are hidden
- Filters are combined as: `(include1 OR include2) AND NOT (exclude1 OR exclude2)

Filter matching is **case-insensitive** substring search against the raw log line.

Add filters via command mode (`:`):
- `:filter <text>` - Add include filter
- `:filter-out <text>` - Add exclude filter
- `:filter-clear` - Remove all filters
- `:list-filters` - View and manage active filters

## Log Coloring

Log lines can be colored based on pattern matching. Create a configuration file at:

1. `./.qlog/qlog.toml` (current directory) - takes precedence
2. `~/.qlog/qlog.toml` (home directory) - fallback

### Example Configuration

```toml
[colors]
error = "red"
warn = "yellow"
success = "green"
"*TODO*" = "magenta"
```

### Pattern Matching

- `error` - matches lines containing "error" (case-insensitive)
- `*error` - matches lines ending with "error"
- `error*` - matches lines starting with "error"
- `*error*` - matches lines containing "error"

First match wins based on config file order. Timestamps remain cyan regardless of line color.

### Supported Colors

Basic: `red`, `green`, `blue`, `yellow`, `magenta`, `cyan`, `white`, `black`, `gray`

Extended: `dark_gray`, `light_red`, `light_green`, `light_blue`, `light_yellow`, `light_magenta`, `light_cyan`

## Architecture

```
src/
├── main.rs              # Entry point and CLI args
├── lib.rs               # Library exports
├── app.rs               # Application state and key handling
├── clipboard.rs         # Clipboard integration for copy operations
├── config.rs            # Log coloring configuration
├── model/
│   ├── log_entry.rs     # Log entry (raw text + optional timestamp)
│   ├── filter.rs        # FilterList with include/exclude logic
│   ├── timestamp.rs     # Timestamp detection from log lines
│   ├── mmap_str.rs      # Memory-mapped string wrapper
│   ├── line_info.rs     # Line position tracking for log files
│   ├── log_storage.rs   # Memory-mapped log storage with indexing
│   ├── visual_line_cache.rs  # Visual line calculation caching
│   ├── selection.rs     # Line selection state management
│   └── mod.rs           # Model module exports
├── storage/
│   ├── loader.rs        # Log file loading
│   └── mod.rs           # Storage module exports
└── ui/
    └── mod.rs           # TUI rendering (filter bar, log list, status)
```

## Testing

```bash
cargo test
```

Tests cover:
- Filter matching logic (include/exclude filters)
- Case-insensitive text matching
- Timestamp detection from various formats
- Log loading from files
- Memory-mapped string operations
- Visual line calculation caching
- Selection state management
- Configuration parsing
- Search matching with Boyer-Moore-Horspool algorithm

## Performance

- Memory-mapped files for efficient reading
- Virtual scrolling renders only visible lines
- Async file loading keeps UI responsive
- **Optimized filtering with Boyer-Moore-Horspool algorithm** - 10-100x faster substring matching
- Zero-allocation, byte-level case-insensitive matching (ASCII-only)
- Early termination for AND-combined filter groups
- Optimized for 2.5GB+ datasets

## License

MIT
