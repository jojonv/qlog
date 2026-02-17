## Context

The log viewer currently has a rigid filter system:
- `FilterSet` uses pure AND logic between all filters
- Filter types (`Level`, `SourceContext`, `DateRange`) are hardcoded to Serilog JSON structure
- `LogEntry` requires JSON parsing with specific fields (timestamp, level, message, properties)
- Auto-filters (Error, Warning) are hardcoded on startup, hiding Information logs
- Filter keybindings documented in README are empty stubs

This prevents:
- OR composition (e.g., "error OR warning")
- Working with non-Serilog/non-JSON logs
- User control over what's filtered

## Goals / Non-Goals

**Goals:**
- Group-based filter composition: filters OR'd within group, groups AND'd together
- Generic log entry model: raw text + optional auto-detected timestamp
- Single filter type: `Contains` (case-insensitive text match)
- Helix-style keybindings for filter management
- Command-line input at bottom for filter text entry
- No auto-filters on startup

**Non-Goals:**
- Regex filters (future consideration)
- Date/time range filters (future consideration)
- JSON field-specific filters (over-engineering for now)
- Match counts on filter chips (future consideration)
- Persisting filters between sessions

## Decisions

### D1: Filter Group Model

**Decision:** Groups of filters where filters within a group are OR'd, groups are AND'd together.

```
FilterSet { groups: Vec<FilterGroup> }     // AND between groups
FilterGroup { filters: Vec<Filter> }       // OR within group
Filter { text: String, enabled: bool }
```

**Rationale:** Matches user mental model: "I want (error OR warning) AND (timeout OR retry)". Alternative was flat list with per-filter AND/OR toggle, but evaluation semantics were ambiguous (left-to-right vs. operator precedence).

### D2: Contains-Only Filter Query

**Decision:** Single `Contains` query type, case-insensitive substring match against raw log line.

**Rationale:** Covers 80% of debugging use cases. Regex adds UI complexity (prompt for type selection). Can add later without breaking model.

### D3: Generic Log Entry

**Decision:** `LogEntry { raw: String, timestamp: Option<DateTime> }`. No required JSON fields.

**Rationale:** Viewer should work with any text logs. Timestamp detection is opportunistic—falls back to file order if not detected. Removes coupling to Serilog format.

**Alternative considered:** Keep JSON parsing for when available, but this adds complexity without clear benefit. Users can still search for JSON fields via Contains.

### D4: Helix-Style Keybindings

**Decision:** Modal-ish keybindings with command-line input:

| Key | Action |
|-----|--------|
| `f` | Add filter to current group (opens command line) |
| `Shift+f` | Create new group, add filter to it |
| `d` | Delete selected filter |
| `Space` | Toggle selected filter on/off |
| `j/k` | Move selection within group |
| `h/l` | Move to prev/next group |
| `Enter` | Confirm command-line input |
| `Esc` | Cancel input / exit filter mode |

**Rationale:** Matches Helix's selection→action pattern. Command-line at bottom is familiar from `:` in Helix/vim.

### D5: No Auto-Filters

**Decision:** Start with empty filter set. No hardcoded Error/Warning filters.

**Rationale:** User should control what they see. Auto-filters were hiding Information logs with no way to disable.

## Risks / Trade-offs

**[R1] Timestamp detection may miss formats** → Use chrono-english or similar library for fuzzy parsing. Fall back gracefully to file order.

**[R2] Contains-only may feel limiting** → Document as design choice; regex is future work.

**[R3] Group model may be confusing at first** → Visual UI with separators (`|`) between groups; hover/selection highlights group membership.
