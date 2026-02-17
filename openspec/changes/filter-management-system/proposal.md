## Why

The como-log-viewer currently has placeholder filter functionality - the UI exists but users cannot actually add, remove, or manage filters. To make the tool useful for debugging and log analysis, we need a complete filter management system that allows users to combine multiple filters with flexible logic (AND/OR) to narrow down the 7.6M log entries to relevant subsets.

## What Changes

- Implement filter modal dialog UI with input fields and type selection
- Add filter CRUD operations (Create, Read, Update, Delete)
- Implement filter toggle functionality (enable/disable without removing)
- Add visual filter bar showing active filters as interactive chips
- **BREAKING**: Redesign filter logic system to support explicit AND/OR operators between filters
- Add filter persistence to save/restore filter configurations
- Implement keyboard shortcuts for filter management (add, toggle, delete, clear all)
- Add "Clear All Filters" functionality
- Support filter presets for common filter combinations

## Capabilities

### New Capabilities
- `filter-modal`: Modal dialog for creating and editing filters with type selection and input
- `filter-bar`: Interactive filter bar displaying active filters as toggleable chips
- `filter-logic`: AND/OR operator system for combining filters with visual indicators
- `filter-persistence`: Save and restore filter configurations between sessions

### Modified Capabilities
- None (this is a new feature implementation)

## Impact

- `como-log-viewer/src/app.rs`: Handle filter-related events and state updates
- `como-log-viewer/src/ui/`: New filter modal, filter bar, and filter logic components
- `como-log-viewer/src/model/filter.rs`: Extend filter model with logic operators
- `como-log-viewer/Cargo.toml`: May add dependencies for modal/dialog handling
- User workflows: New keyboard shortcuts to learn, but existing navigation remains unchanged
