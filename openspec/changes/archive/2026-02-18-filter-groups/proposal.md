## Why

The current filter system is too rigid: it only supports AND logic between filters, and filter types are hardcoded to Serilog JSON structure (Level, SourceContext, DateRange). Users cannot compose filters with OR logic, and the viewer cannot work with arbitrary log formats. This limits debugging workflows where you often need "show me errors OR warnings, but only with 'timeout'".

## What Changes

- **BREAKING**: Remove structure-specific filter types (`Filter::Level`, `Filter::SourceContext`, `Filter::DateRange`)
- Replace flat `FilterSet` with group-based composition: filters are OR'd within groups, groups are AND'd together
- Simplify `Filter` to single `Contains` query type (case-insensitive text match)
- Make `LogEntry` generic: store raw text line with auto-detected timestamp (no required JSON fields)
- Implement Helix-style keybindings for filter management (`f` add, `d` delete, `Space` toggle, `h/l` navigate groups)
- Add command-line input at bottom for filter text entry (Helix-style `:` prompt)
- Remove auto-filters on startup (currently hardcodes Error + Warning)

## Capabilities

### New Capabilities

- `filter-groups`: Group-based filter composition with OR-within-group, AND-between-groups semantics, plus Helix-style filter management UI

### Modified Capabilities

- None (no existing specs to modify)

## Impact

- `src/model/filter.rs`: Complete rewrite for Filter/FilterGroup/FilterSet model
- `src/model/log_entry.rs`: Simplify to raw text + optional timestamp
- `src/storage/loader.rs`: Update to new LogEntry structure, add timestamp detection
- `src/main.rs` or `src/app.rs`: Implement filter group UI, keybindings, command-line input
- `src/ui/mod.rs`: Render filter groups with visual separation
