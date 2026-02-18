# Como Log Viewer

A TUI (Terminal User Interface) application for viewing and filtering como-data-center log files with Helix-style keybindings.

## Features

- **Unified Log View**: Merge all log files chronologically
- **Flexible Filtering**: Apply/Remove multiple filters
- **Helix Keybindings**: Navigation using hjkl, gg/G for goto top/bottom
- **Color Coding**: Errors (Red), Warnings (Yellow), Information (Green)
- **Async Loading**: Efficient memory-mapped file loading for large datasets
- **Virtual Scrolling**: Handle millions of lines with only visible lines rendered
- **Horizontal Scroll**: View wide JSON content

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

### Navigation (Helix-style)
- `h/j/k/l` - Navigate left/down/up/right (scroll)
- `Ctrl+f` / `Ctrl+b` - Page down/up
- `gg` / `G` - Go to top/bottom
- `Arrow keys` - Alternative navigation

### Filters
- `f` - Open filter dialog
- `Space` - Toggle filter on/off
- `d` - Delete filter under cursor
- `:` - Command mode for date ranges

### General
- `q` - Quit application
- `Esc` - Cancel current operation

## Filter Types

1. **Level Filter**: Filter by Information, Warning, or Error
2. **Text Filter**: Search for text in message, source, or properties
3. **Date Range Filter**: Filter by start/end timestamps
4. **Source Context Filter**: Filter by specific source context

Multiple filters combine with AND logic.

## Architecture

```
src/
├── main.rs        # Entry point and CLI args
├── lib.rs         # Library exports
├── app.rs         # Application state and event handling
├── model/         # Data models
│   ├── log_entry.rs  # Log entry structure
│   └── filter.rs     # Filter definitions
├── storage/       # File I/O
│   └── loader.rs     # Async log loading
└── ui/            # UI components
    └── mod.rs        # Main UI rendering
```

## Testing

```bash
cargo test
```

Tests cover:
- Log entry parsing from JSON
- Filter matching logic
- Date range handling
- Log loading from files

## Performance

- Memory-mapped files for efficient reading
- Virtual scrolling renders only visible lines
- Async file loading keeps UI responsive
- Optimized for 2.5GB+ datasets

## License

MIT
