## Why

After implementing filter groups, the j/k/h/l keys were bound to filter navigation in the default mode, breaking log scrolling. Users cannot scroll content because the keybindings conflict: they navigate filters instead of scrolling logs.

## What Changes

- Introduce two distinct focus modes: **Content Mode** (default) and **Filter Mode**
- **BREAKING**: j/k/h/l behavior changes based on active mode
- `t` toggles between Content and Filter modes
- `Esc` returns to Content mode from Filter mode
- Content Mode: j/k scroll logs, h/l scroll horizontally
- Filter Mode: j/k select filter, h/l switch groups, f/F/d/Space for filter operations

## Capabilities

### New Capabilities

- `focus-modes`: Two-mode navigation system where j/k/h/l behavior depends on active focus (Content vs Filter), with `t` to toggle and `Esc` to return to Content mode

### Modified Capabilities

- `filter-groups`: Requirement change - filter navigation (j/k within group, h/l between groups, f/F/d/Space) only works in Filter Mode, not the default mode

## Impact

- `src/app.rs`: Mode enum semantics change, keybindings reassigned per mode
- `src/ui/mod.rs`: Help text in status bar needs to reflect current mode
- User experience: Clearer separation between content interaction and filter management
