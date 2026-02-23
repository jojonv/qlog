## ADDED Requirements

### Requirement: Tab triggers command completion

The system SHALL provide tab-based autocomplete for command names in Command mode.

#### Scenario: Complete partial command
- **WHEN** user types "fil" in command mode and presses Tab
- **THEN** input buffer updates to "filter" (first matching command alphabetically)

#### Scenario: Cycle through matches
- **WHEN** user has "fil" and presses Tab twice
- **THEN** input buffer shows "filter-out" (second match)

#### Scenario: Wrap around on cycle
- **WHEN** user cycles past the last matching command and presses Tab again
- **THEN** input buffer wraps back to the first matching command

#### Scenario: No match does nothing
- **WHEN** user types "xyz" and presses Tab
- **THEN** input buffer remains unchanged

### Requirement: Completion applies to command portion only

The system SHALL only complete the command name (text before first space).

#### Scenario: Existing arguments preserved
- **WHEN** user types "filter some text" and presses Tab
- **THEN** input buffer remains unchanged (space detected, no completion)

#### Scenario: Empty input shows all commands
- **WHEN** user presses Tab with empty input buffer
- **THEN** input buffer shows "filter" (first command alphabetically)

### Requirement: Completion resets on non-Tab input

The system SHALL reset the completion cycle when user types any non-Tab character.

#### Scenario: Typing resets cycle
- **WHEN** user cycles to "filter-out" then types "a"
- **THEN** completion index resets and next Tab starts fresh cycle

#### Scenario: Backspace resets cycle
- **WHEN** user cycles to "filter-clear" then presses Backspace
- **THEN** completion index resets
