## Why

Filters are visually flat - all groups look the same, making it hard to understand filter structure at a glance. Additionally, there's no visual feedback showing which log lines match which filters, forcing users to mentally parse each line. Color-coded groups and match highlighting would provide immediate visual comprehension of filter organization and matching behavior.

## What Changes

- **Group color assignment**: Each filter group gets assigned a color from a fixed palette (cycling through cyan, magenta, yellow, green, etc.)
- **Filter bar coloring**: Filters in the filter bar are rendered in their group's color
- **Selection indicator**: Selected filter shows a dark background highlight + bold; active group shows subtle background tint on all its filters
- **Log match highlighting**: Each filter's matching text in log lines is highlighted with the filter's group color (first match wins for overlaps)
- **Group separator styling**: Group separators (`|`) remain neutral to let group colors pop

## Capabilities

### New Capabilities

- `filter-colors`: Color assignment and rendering for filter groups with selection indicators
- `match-highlighting`: Color-coded highlighting of filter matches in log content

### Modified Capabilities

- None (this is additive visual enhancement, no spec-level behavior changes)

## Impact

- `src/ui/mod.rs`: Filter bar rendering, log line rendering
- New color palette constant or function
- Performance: substring searches per visible log line (should be acceptable for typical TUI usage)
