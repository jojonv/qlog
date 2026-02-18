## MODIFIED Requirements

### Requirement: Filter navigation only in Filter mode

Filter navigation keybindings (j/k for filter selection, h/l for group switching, f/F for adding, d for delete, Space for toggle) SHALL only be active when in Filter mode, not in the default Content mode.

#### Scenario: j/k navigate filters only in Filter mode
- **WHEN** user is in Content mode and presses `j` or `k`
- **THEN** system SHALL scroll log content, NOT navigate filters

#### Scenario: h/l switch groups only in Filter mode
- **WHEN** user is in Content mode and presses `h` or `l`
- **THEN** system SHALL scroll horizontally, NOT switch filter groups

#### Scenario: f/F/d/Space operate filters only in Filter mode
- **WHEN** user is in Content mode and presses `f`, `F`, `d`, or `Space`
- **THEN** system SHALL NOT perform filter operations

#### Scenario: All filter operations work in Filter mode
- **WHEN** user is in Filter mode
- **THEN** all filter keybindings (j/k/h/l/f/F/d/Space) SHALL work as defined

---

## ADDED Requirements

### Requirement: Filter groups support OR-within-group AND-between-groups composition

The system SHALL organize filters into groups where filters within a single group are combined with OR logic, and groups are combined with AND logic.

#### Scenario: Single group with multiple filters matches any filter
- **WHEN** group contains filters "error" and "warning" (both enabled)
- **AND** a log line contains "error" but not "warning"
- **THEN** the log line SHALL match the group

#### Scenario: Multiple groups require match in each group
- **WHEN** filter set has two groups: group 1 with "error", group 2 with "timeout"
- **AND** a log line contains "error" but not "timeout"
- **THEN** the log line SHALL NOT match the filter set

#### Scenario: Disabled filters are ignored in matching
- **WHEN** group contains filter "error" (enabled) and "warning" (disabled)
- **AND** a log line contains "warning" but not "error"
- **THEN** the log line SHALL NOT match the group

---

### Requirement: Filters use case-insensitive Contains text matching

The system SHALL match filters against the raw log line text using case-insensitive substring search.

#### Scenario: Contains match is case-insensitive
- **WHEN** filter text is "ERROR"
- **AND** log line contains "error" (lowercase)
- **THEN** the filter SHALL match the log line

#### Scenario: Contains match finds substring anywhere
- **WHEN** filter text is "timeout"
- **AND** log line is "Connection timeout after 30s"
- **THEN** the filter SHALL match the log line

---

### Requirement: Log entries store raw text with optional detected timestamp

The system SHALL store each log entry as raw text with an optional auto-detected timestamp.

#### Scenario: Timestamp detected from common formats
- **WHEN** log line starts with "2026-02-17T14:30:00"
- **THEN** the entry SHALL have timestamp parsed as DateTime

#### Scenario: Missing timestamp results in None
- **WHEN** log line has no recognizable timestamp pattern
- **THEN** the entry SHALL have timestamp of None

---

### Requirement: Helix-style keybindings for filter management

The system SHALL provide the following keybindings for filter interaction:

#### Scenario: Add filter opens command-line input
- **WHEN** user presses `f` in filter mode
- **THEN** system SHALL display command-line prompt at bottom
- **AND** user SHALL be able to type filter text

#### Scenario: Confirm input creates filter
- **WHEN** user types text in command-line and presses Enter
- **THEN** system SHALL add new filter with that text to current group

#### Scenario: Shift+f creates new group
- **WHEN** user presses Shift+f
- **THEN** system SHALL create a new filter group
- **AND** subsequent filter addition goes to new group

#### Scenario: Delete removes selected filter
- **WHEN** user presses `d` with a filter selected
- **THEN** system SHALL remove that filter from its group
- **AND** if group becomes empty, remove the group

#### Scenario: Space toggles filter enabled state
- **WHEN** user presses Space with a filter selected
- **THEN** system SHALL toggle the filter's enabled state

#### Scenario: h/l navigate between groups
- **WHEN** user presses `h` or `l`
- **THEN** system SHALL move selection to previous or next group

#### Scenario: j/k navigate within group
- **WHEN** user presses `j` or `k`
- **THEN** system SHALL move selection up or down within current group

---

### Requirement: No auto-filters on startup

The system SHALL start with an empty filter set, applying no automatic filters.

#### Scenario: Fresh start shows all logs
- **WHEN** application starts
- **THEN** filter set SHALL be empty
- **AND** all log entries SHALL be visible

---

### Requirement: Filter bar displays groups with visual separation

The system SHALL render the filter bar with visual distinction between groups.

#### Scenario: Groups visually separated
- **WHEN** filter set contains multiple groups
- **THEN** filter bar SHALL display groups with separators (e.g., `|` or spacing)

#### Scenario: Disabled filters shown grayed
- **WHEN** a filter is disabled
- **THEN** filter chip SHALL be displayed with muted/strikethrough styling
