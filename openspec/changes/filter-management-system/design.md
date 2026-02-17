## Context

The como-log-viewer is a TUI application built with Rust using ratatui for rendering. It currently has a basic structure with App state, UI components, and model definitions. The filter system exists in `src/model/filter.rs` but lacks:
1. UI components for filter management
2. Filter persistence between sessions
3. Complex filter logic (AND/OR)
4. Visual feedback showing active filters

The app uses a model-view architecture where:
- `App` holds state in `src/app.rs`
- UI rendering happens in `src/ui/mod.rs`
- Events are handled in the main event loop
- Filters are applied to the log entries for display

## Goals / Non-Goals

**Goals:**
- Implement modal dialog for adding/editing filters with type selection
- Create visual filter bar showing active filters as chips with match counts
- Support AND/OR logic operators between filters with visual indicators
- Enable toggling filters on/off without removing them
- Implement "Clear All Filters" functionality
- Save filter configurations to disk and restore on startup
- Maintain Helix-style keyboard shortcuts
- Support NOT filters (negation)
- Support relative date expressions ("2 hours ago", "yesterday", etc.)
- Track and restore filter history (last 20 combinations)

**Non-Goals:**
- SQL-like query language for filters
- Advanced regex editor with syntax highlighting
- Filter sharing between users
- Cloud sync of filter presets
- Mouse-only interaction (keyboard-first design)

## Decisions

### 1. Filter Logic: Explicit Operators vs Type-Based
**Decision:** Use explicit AND/OR operators between filters with visual toggle

**Rationale:** 
- Type-based (Option C from brainstorm) seemed elegant but is too magical
- Users won't understand why [Level:Error] OR [Level:Warning] AND [Text:"failed"] behaves differently than [Text:"failed"] AND [Level:Error] OR [Level:Warning]
- Explicit operators give users full control and clarity
- Visual indicators (● for AND, ○ for OR) between filter chips make logic obvious

**Alternatives considered:**
- Type-based logic: Rejected - too implicit, confusing behavior
- Always AND: Rejected - too limiting for real use cases
- SQL syntax: Rejected - overkill for this tool

### 2. Modal Dialog vs Command Palette
**Decision:** Use modal dialog for filter creation

**Rationale:**
- Modal dialogs are more discoverable for new users
- Can show input validation and autocomplete inline
- Better for complex filter types (date ranges with visual pickers)
- Command palette is better for power users but harder to discover

**Implementation:**
- `f` key opens modal with dropdown for filter type
- Form fields change based on selected type
- Enter confirms, Esc cancels
- Modal centered on screen with dimmed background

### 3. Filter Persistence Format
**Decision:** JSON file in `~/.config/como-log-viewer/filters.json`

**Rationale:**
- JSON is human-readable and editable
- Easy to backup and version control
- No external dependencies needed
- Standard XDG config location follows conventions

**Structure:**
```json
{
  "active_filters": [...],
  "presets": [...],
  "last_used": [...]
}
```

### 4. Filter Chip UI Design
**Decision:** Horizontal bar at top with toggleable chips

**Rationale:**
- Always visible - users know what filters are active
- Compact representation using abbreviations
- Toggle state (dimmed when off) provides visual feedback
- Arrow keys navigate chips, Space toggles, d deletes

**Visual design:**
```
Filters: [Error ● "failed" ○ Today ▲]
          │     │         │
          │     │         └── Chip 3
          │     └──────────── Chip 2 with OR operator
          └────────────────── Chip 1
```

### 5. Keyboard Shortcuts
**Decision:** Helix-inspired shortcuts

**Rationale:**
- User specifically requested Helix-like keybindings
- Consistent with existing navigation (hjkl, gg, G)
- Power-user friendly while being mnemonic

**Shortcuts:**
- `f` - Open filter modal
- `F` - Clear all filters
- `t` - Toggle filter under cursor
- `d` - Delete filter under cursor
- Arrow keys - Navigate between filter chips
- Space - Toggle selected filter
- `:` - Command mode (future expansion)

## Risks / Trade-offs

### Risk: Complex filter logic may confuse users
**Mitigation:** 
- Default to AND for first two filters
- Show visual indicators prominently
- Add "Logic Help" in modal
- Consider "Simple Mode" with only AND

### Risk: Too many filters = poor performance
**Mitigation:**
- Filter evaluation is lazy - only check visible entries
- Cache filter results
- Limit number of active filters (soft limit with warning)

### Risk: Modal dialog blocks the view
**Mitigation:**
- Modal is compact (40% of screen)
- Background remains visible but dimmed
- Esc instantly closes
- Consider "quick filter" mode for text-only filters

### Trade-off: JSON persistence vs Database
- **JSON:** Simple, no deps, human-readable, but slower for large filter sets
- **Database:** Faster queries, complex relations, but adds SQLite dependency
- **Decision:** JSON is sufficient for typical use (10-20 filters max)

## Migration Plan

1. **Phase 1:** Implement modal dialog and basic CRUD (no persistence)
2. **Phase 2:** Add filter bar UI with chips
3. **Phase 3:** Implement AND/OR logic operators
4. **Phase 4:** Add persistence layer
5. **Phase 5:** Polish and edge cases

**Rollback:** Simply revert to previous git commit - no database migrations or external state

### 6. NOT Filters
**Decision:** Add negation toggle to filters
- Press `!` on focused filter to negate
- Visual: red "NOT" prefix or strikethrough
- Example: NOT [Level: Debug] shows everything except debug
- Implementation: invert Filter.matches() result

### 7. Relative Dates
**Decision:** Support natural language date expressions
- Parse: "2 hours ago", "yesterday", "today", "last week"
- Store as both expression + computed timestamp
- Recalculate on app start (relative to current time)

### 8. Match Counts
**Decision:** Display match counts in filter chips
- Format: [Error (1,234)] or [contains: "failed" (89)]
- Live update during filter application
- Disable if > 100k matches (performance)
- Configurable in settings

### 9. Filter History
**Decision:** Track last 20 unique filter combinations
- Access via `:history` command or `Ctrl+h`
- Shows: timestamp + filter summary + match count
- Click to restore filter set
- Persist to `~/.config/como-log-viewer/filter-history.json`

1. Should we support NOT filters (exclude logs matching criteria)?
2. Should date filters support relative dates ("last 2 hours") or only absolute?
3. Should filter presets have keyboard shortcuts (e.g., `1`, `2`, `3`)?
4. Should we show match count per filter (e.g., "Error (152 matches)")?
5. Should we add a "filter history" to quickly re-apply previous filters?
