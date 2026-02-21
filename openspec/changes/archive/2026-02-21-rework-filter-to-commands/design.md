## Context

The current filter system uses a hierarchical structure: FilterSet (AND between groups) → FilterGroup (OR within group) → Filter (individual patterns). This creates complexity for users who must understand the implicit logic and navigate through groups and filters separately. The UI displays this as a visual filter bar with group separators.

Current implementation in src/model/filter.rs:
- Filter struct with BMHMatcher for substring matching
- FilterGroup with Vec<Filter> combined via OR logic
- FilterSet with Vec<FilterGroup> combined via AND logic
- Mode::Filter with complex navigation (h/l for groups, j/k for filters)

## Goals / Non-Goals

**Goals:**
- Replace hierarchical FilterSet/FilterGroup/Filter with flat FilterList architecture
- Implement command-based filter management (:filter, :filter-out, :filter-clear, :list-filters)
- Simplify UI by removing visual filter mode and filter bar
- Follow SOLID principles: separate concerns between matching, storage, and UI
- Maintain case-insensitive substring matching behavior
- Provide comprehensive unit tests for FilterList

**Non-Goals:**
- Regex support (deferred to future change)
- Date/time filtering (deferred to future change)
- Persistent filters across sessions
- Individual filter toggle (filters are active or deleted)
- OR logic between filters (requires regex)

## Decisions

### Decision: Flat FilterList replaces hierarchical FilterSet
**Rationale**: The hierarchical structure (AND groups of OR filters) is overkill for the lnav-style command interface. A simple flat list with separate include/exclude filters is easier to understand and matches the user's mental model of "filter IN this AND NOT that".

**Structure**:
```rust
pub struct FilterList {
    includes: Vec<FilterRule>,  // All must match (AND)
    excludes: Vec<FilterRule>,  // None must match (AND NOT)
}

pub struct FilterRule {
    pattern: String,
    matcher: BMHMatcher,  // Pre-built for performance
}
```

### Decision: Command parsing integrated into existing Command mode
**Rationale**: The app already has Mode::Command for :write/:quit. Extending this to handle filter commands is more consistent than creating a new mode. Command parsing will dispatch to FilterList operations.

**Alternative considered**: Separate FilterCommand mode - rejected as unnecessary complexity.

### Decision: BMHMatcher reused but wrapped in FilterRule
**Rationale**: The existing Boyer-Moore-Horspool implementation is efficient. We keep it but extract from Filter struct into a reusable component that FilterRule owns.

### Decision: :list-filters provides interactive management
**Rationale**: Users need a way to see and delete individual filters. This will be a modal overlay (similar to help) rather than a full mode, keeping the architecture simple.

**Navigation**: j/k to move cursor, d to delete selected, q/Enter to close

### Decision: Status bar shows filter count only
**Rationale**: Removing the visual filter bar reduces complexity. The status bar will show "Filters: N active" or similar. Users can :list-filters for details.

### Decision: SOLID refactoring approach
**Rationale**: Current Filter struct mixes concerns (matching logic, UI state like enabled flag, display formatting).

**Separation**:
- **FilterRule**: Pure matching logic (Single Responsibility)
- **FilterList**: Collection management, provides matching interface
- **CommandHandler**: Parses :filter commands, delegates to FilterList
- **FilterListView**: Renders :list-filters UI (separate from FilterList)

## Risks / Trade-offs

**Risk**: Users accustomed to visual filter mode will be confused
**Mitigation**: Update README with new command reference; keep :help command updated

**Risk**: No OR logic limits filter expressiveness
**Mitigation**: Document limitation; plan regex support for future OR capability

**Risk**: Performance regression from removing BMH optimization
**Mitigation**: Keep BMHMatcher, just restructure ownership; profile after implementation

**Risk**: Breaking change affects existing workflows
**Mitigation**: This is intentional per proposal; communicate clearly in release notes

**Trade-off**: Status bar is less informative than filter bar
**Mitigation**: Users can :list-filters anytime for detailed view; this is acceptable for simplicity gain

## Migration Plan

1. Create FilterList with FilterRule structs (parallel to existing)
2. Implement command handlers for :filter/:filter-out/:filter-clear
3. Wire FilterList into App state
4. Remove FilterSet/FilterGroup from App
5. Remove Mode::Filter and Mode::FilterInput
6. Remove filter bar UI rendering
7. Add :list-filters UI overlay
8. Update all tests
9. Archive change

## Open Questions

None - design is complete.
