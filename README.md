# Como Log Viewer

A TUI (Terminal User Interface) application for viewing and filtering como-data-center log files with Helix-style keybindings.

## Features

- **Unified Log View**: Merge all log files chronologically with auto-detected timestamps
- **Filter Groups**: Compose filters with OR-within-group, AND-between-groups logic
- **Helix Keybindings**: Modal editing with hjkl navigation and filter management
- **Generic Log Support**: Works with any text log format (not just JSON)
- **Async Loading**: Efficient loading for large datasets
- **Virtual Scrolling**: Handle millions of lines with only visible lines rendered
- **Horizontal Scroll**: View wide log content with wrap mode toggle
- **Configurable Log Coloring**: Customize line colors via TOML config files

## Installation

```bash
cd como-log-viewer
cargo build --release
```

The binary will be at `target/release/como-log-viewer`.

## Usage

```bash
# Automatically find *.log files in current directory
./como-log-viewer

# Or specify specific log files
./como-log-viewer /path/to/*.log
```

## Keybindings

### Navigation (Normal Mode)
- `j/k` or `Arrow Up/Down` - Scroll through logs
- `h/l` or `Arrow Left/Right` - Horizontal scroll
- `g` - Go to top
- `G` - Go to bottom
- `w` - Toggle wrap mode
- `t` - Enter filter mode
- `q` - Quit application

### Filter Mode
- `f` - Add filter to current group (opens input)
- `F` (Shift+f) - Create new group and add filter
- `j/k` - Select filter within group
- `h/l` - Switch between groups
- `Space` - Toggle filter on/off
- `d` - Delete selected filter
- `t` or `Esc` - Return to normal mode

### Filter Input Mode
- `Enter` - Confirm filter text
- `Esc` - Cancel input
- `Backspace` - Delete character

## Filter Groups

Filters are organized into groups with powerful composition:

- **Filters within a group** are combined with **OR** logic
- **Groups** are combined with **AND** logic

Example: `("error" OR "warning") AND ("timeout" OR "retry")`

This creates two groups:
1. Group 1: `error` OR `warning`
2. Group 2: `timeout` OR `retry`

A log line must match at least one filter from each group to be shown.

Filter matching is **case-insensitive** substring search against the raw log line.

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
├── main.rs           # Entry point and CLI args
├── lib.rs            # Library exports
├── app.rs            # Application state and key handling
├── config.rs         # Log coloring configuration
├── model/
│   ├── log_entry.rs  # Log entry (raw text + optional timestamp)
│   ├── filter.rs     # Filter/FilterGroup/FilterSet with OR/AND logic
│   └── timestamp.rs  # Timestamp detection from log lines
├── storage/
│   └── loader.rs     # Log file loading
└── ui/
    └── mod.rs        # TUI rendering (filter bar, log list, status)
```

## Testing

```bash
cargo test
```

Tests cover:
- Filter matching logic (OR within groups, AND between groups)
- Case-insensitive text matching
- Timestamp detection from various formats
- Log loading from files

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
