## ADDED Requirements

### Requirement: Index-based filtering storage

The system SHALL store filtered log results as indices into the main log vector rather than cloned entries.

#### Scenario: Filter update creates indices not clones
- **WHEN** user applies or modifies a filter
- **THEN** the system stores matching entry indices in `filtered_indices: Vec<usize>`
- **AND** no LogEntry clones are created

#### Scenario: Filtered entry access via index
- **WHEN** UI or command needs to access a filtered entry
- **THEN** the system retrieves the entry via `logs[filtered_indices[position]]`
- **AND** returns a reference to the original LogEntry

### Requirement: Memory-efficient filter operations

The system SHALL provide O(n) filter application with O(1) per-entry memory overhead (8 bytes per index).

#### Scenario: Filter memory overhead
- **WHEN** filtering 1 million log entries
- **THEN** filtered storage uses approximately 8MB for indices
- **AND** does NOT duplicate log content strings

#### Scenario: Filter update performance
- **WHEN** user toggles a filter on/off
- **THEN** filter indices are recalculated without cloning
- **AND** memory usage remains stable (no spikes)

### Requirement: Backward-compatible API surface

The system SHALL maintain semantic compatibility for all `filtered_logs` operations via accessor methods.

#### Scenario: Length queries work identically
- **WHEN** code queries `filtered_logs.len()`
- **THEN** equivalent result is available via `filtered_len()` method

#### Scenario: Iteration works via accessor
- **WHEN** code iterates filtered entries
- **THEN** iteration yields references to original LogEntry instances
- **AND** entry order matches filter result order
