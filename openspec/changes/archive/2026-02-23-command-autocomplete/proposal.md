## Why

Users currently must type full command names (`:filter`, `:filter-out`, `:list-filters`, etc.) without any assistance. This slows down workflow and requires memorization. Tab completion is a standard TUI/shell pattern that would significantly improve command entry speed and discoverability.

## What Changes

- Add Tab key handling in Command mode to trigger autocomplete
- Implement cycling completion through matching commands
- Commands: `filter`, `filter-clear`, `filter-out`, `list-filters`, `quit`, `write` (and aliases `w`, `q`)
- Completion only applies to the command portion (text before first space)
- Typing any non-Tab character resets the completion cycle

## Capabilities

### New Capabilities

- `command-completion`: Tab-based cycling autocomplete for command names in command mode

### Modified Capabilities

None - this is a new feature with no spec-level changes to existing capabilities.

## Impact

- `src/app.rs`: Add `completion_index` state field, Tab key handler in `handle_command_key`, completion logic
- UI behavior only - no API or storage changes
