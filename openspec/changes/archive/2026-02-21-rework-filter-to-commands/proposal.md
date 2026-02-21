## Why

The current visual filter mode (activated by 't') requires complex navigation between filter groups and individual filters, creating a steep learning curve. Users must remember h/l for group switching, j/k for filter navigation, and understand the implicit OR-within-group AND-between-groups logic. A command-based approach aligned with lnav's pattern will provide a more intuitive, discoverable interface where users explicitly construct filter expressions.

## What Changes

- **BREAKING**: Remove visual Filter mode (pressing 't') and FilterInput mode entirely
- **BREAKING**: Remove filter group navigation (h/l for groups, j/k for filters)
- **BREAKING**: Remove filter bar UI that displays groups and enabled/disabled states
- Add `:filter <text>` command to add include filters (AND logic between multiple filters)
- Add `:filter-out <text>` command to add exclude filters
- Add `:filter-clear` command to remove all filters
- Add `:list-filters` command to view and manage active filters interactively
- Implement FilterList struct with flat stack architecture (replaces FilterSet/FilterGroup hierarchy)
- Display active filter count in status bar (e.g., "Filters: 3 active")
- **BREAKING**: Remove ability to toggle individual filters on/off - filters are either active or deleted
- Add comprehensive unit tests for FilterList matching logic
- Refactor filter system following SOLID principles (separate matching, storage, and UI concerns)

## Capabilities

### New Capabilities
- `command-based-filtering`: Filter management via command-line interface with :filter, :filter-out, :filter-clear, and :list-filters commands

### Modified Capabilities
- `filter-groups`: **BREAKING CHANGE** - Remove visual filter group management, group navigation, and filter bar UI. Replace with command-based flat filter list.

## Impact

- src/model/filter.rs - Replace FilterSet/FilterGroup/Filter hierarchy with FilterList
- src/app.rs - Remove handle_filter_key and handle_filter_input_key handlers
- src/ui/mod.rs - Remove draw_filter_bar and draw_filter_input functions
- Status bar will show filter count instead of visual filter bar
- Mode enum simplified (remove Filter and FilterInput variants)
- All existing filter tests need updating to use FilterList API
