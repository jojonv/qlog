## Why

The log viewer currently lacks a way to find specific text within filtered results. Users must manually scan through potentially thousands of lines to locate occurrences of keywords. A search mode would enable quick navigation to specific text patterns, significantly improving the efficiency of log analysis and debugging workflows.

## What Changes

- **New mode**: Add `SearchInput` mode triggered by `/` key in Normal mode
- **Search navigation**: Implement `n` (next) and `N` (previous) keys to jump between matches in Normal mode
- **Dual highlighting**: Highlight all search matches in visible lines, with the "current" match (where cursor is) using distinct styling
- **Unified configuration**: Extend existing config system to support search highlight colors and styles (foreground, background, style modifiers)
- **Horizontal auto-scroll**: Automatically adjust horizontal scroll position to bring matches into view
- **Automatic cleanup**: Clear search state when filters change to maintain consistency
- **Case-insensitive search**: Search is always case-insensitive using ASCII lowercase matching
- **Empty query clears**: Pressing `/` then Enter clears the current search

## Capabilities

### New Capabilities
- `search-mode`: Text search within filtered log results with navigation and highlighting
- `unified-config`: Consolidated configuration system for colors and search settings

### Modified Capabilities
- `configurable-log-coloring`: Extend to support search highlight configuration (colors and styles for matches)

## Impact

- **App struct**: Add search state fields (query, matcher, current match index, match positions)
- **Mode enum**: Add `SearchInput` variant
- **Key handlers**: Add `/` binding in Normal mode, `n`/`N` for navigation, new handler for SearchInput mode
- **Config system**: Extend ColorConfig or create unified AppConfig with search highlight settings
- **UI rendering**: Modify log line rendering to highlight search matches with configurable styles
- **Horizontal scroll**: Update logic to auto-scroll when navigating to off-screen matches
