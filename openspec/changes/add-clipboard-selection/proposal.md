## Why

Users need to copy log lines from the viewer to use in bug reports, documentation, or external tools. Currently, there's no way to extract text from the TUI interface, forcing users to switch to terminal selection or external tools.

## What Changes

- **New model**: Add `Selection` struct to track line selection state (helix-style)
- **New infrastructure**: Add `Clipboard` wrapper around `arboard` crate for system clipboard integration
- **New keybindings**: `x` to select line/extend selection, `j`/`k` to extend selection when active, `y` to copy selected lines, `Escape` to clear selection
- **UI updates**: Highlight all lines in selection range with DarkGray background
- **Error handling**: Graceful degradation when clipboard unavailable

## Capabilities

### New Capabilities
- `line-selection`: Helix-style multi-line selection with keyboard controls
- `system-clipboard`: Copy selected text to system clipboard with platform abstraction

### Modified Capabilities
- (none - this is purely additive functionality)

## Impact

- New dependency: `arboard` crate added to Cargo.toml
- New modules: `src/model/selection.rs`, `src/clipboard.rs`
- Modified: `src/app.rs` (add selection/clipboard fields, key handlers)
- Modified: `src/ui/mod.rs` (render selection range)
- Modified: `src/model/mod.rs`, `src/lib.rs` (export new modules)
