## Why

Log files are hard to scan visually, especially when searching for specific patterns like errors, warnings, or custom markers. Currently, como-log-viewer only colors timestamps (cyan) and selected lines (dark gray). Users need a configurable way to highlight entire log lines based on patterns, making it easier to visually identify important information at a glance.

## What Changes

- Add TOML-based configuration system for log coloring
- Support `.qlog/qlog.toml` in current directory or home directory (cross-platform via `dirs` crate)
- Pattern-based matching with wildcard support (e.g., `error*`, `*error`, `*error*`)
- Case-insensitive partial matching (e.g., "error" matches "ERROR", "ApiError", "[error]")
- Entire line coloring using ratatui color names (red, yellow, green, blue, cyan, magenta, etc.)
- **BREAKING**: No coloring if configuration file is not found (removing default assumptions)
- Add new dependencies: `toml` for config parsing, `dirs` for home directory detection

## Capabilities

### New Capabilities
- `configurable-log-coloring`: Pattern-based log line coloring system with TOML configuration support

### Modified Capabilities
- (none - this is a new visual enhancement capability)

## Impact

- **UI Rendering Layer** (`src/ui/mod.rs`): Modified to apply color styles based on pattern matches
- **New Config Module** (`src/config.rs`): New module for loading and parsing TOML configuration
- **Dependencies**: Add `toml` and `dirs` crates to Cargo.toml
- **User Experience**: Users can now create `.qlog/qlog.toml` files to customize log highlighting
