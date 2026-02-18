## Why

The log viewer currently duplicates all filtered log entries in memory by storing complete clones in `filtered_logs: Vec<LogEntry>`. With large log files (10GB+), this causes excessive memory consumption (15GB+ RAM), making the application unsustainable for production use. Competing tools like klogg and lnav achieve low memory footprints by using index-based references instead of data duplication.

## What Changes

- Replace `filtered_logs: Vec<LogEntry>` with `filtered_indices: Vec<usize>` to store references to original entries
- Remove `.cloned().collect()` pattern in `update_filtered_logs()` 
- Update all consumers of `filtered_logs` to access via index into `logs` vector
- **BREAKING**: Internal API changes - `filtered_logs` field removed from `App` struct

## Capabilities

### New Capabilities
- `index-based-filtering`: Store filtered results as indices into the main log vector instead of cloned entries

### Modified Capabilities
- `filter-groups`: Requirement change - filtered results must be accessible via index lookup rather than direct ownership

## Impact

- **Memory**: 30-50% reduction in memory usage for filtered views
- **Performance**: Faster filter updates (no cloning), slightly slower individual entry access (one indirection)
- **Files affected**: `src/app.rs`, `src/ui.rs` (any code accessing `filtered_logs`)
- **Backward compatibility**: Internal change only, no user-facing API changes
