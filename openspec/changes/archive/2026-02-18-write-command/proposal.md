## Why

Users need to export filtered log data for external analysis, sharing, or archival. Currently, there's no way to save the filtered view - users can only view logs within the TUI. This is a fundamental feature gap for any log viewer.

## What Changes

- Add `:w` and `:write` command support in command mode
- Save all currently filtered logs to a user-specified file
- Support default filename (timestamped) when no filename provided
- Support quoted filenames for paths with spaces
- Output format: raw text (original log lines as parsed)
- Create new file or overwrite existing
- Show success/error feedback in status bar

## Capabilities

### New Capabilities

- `write-command`: Export filtered logs to file via command mode (`:w`/`:write`)

### Modified Capabilities

None. This is a new feature that extends command mode without changing existing behavior.

## Impact

- **Affected code**:
  - `src/app.rs`: Extend `handle_command_key` to parse and execute write commands
  - Potentially new module for command parsing (`src/command.rs`)
- **No API changes**: Internal TUI feature only
- **No dependencies**: Uses standard library `std::fs` for file I/O
