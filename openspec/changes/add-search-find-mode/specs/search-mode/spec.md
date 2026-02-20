## ADDED Requirements

### Requirement: Search mode activation
The system SHALL allow users to enter search input mode from Normal mode.

#### Scenario: Activate search mode
- **WHEN** the user presses `/` in Normal mode
- **THEN** the system SHALL enter SearchInput mode
- **AND** display a search input prompt at the bottom of the screen
- **AND** clear any previous search query from the input buffer

#### Scenario: Cancel search input
- **WHEN** the user presses `Esc` in SearchInput mode
- **THEN** the system SHALL return to Normal mode
- **AND** discard any input in the search buffer
- **AND** preserve the previous search state (if any)

### Requirement: Search query input
The system SHALL accept text input for the search query in SearchInput mode.

#### Scenario: Type search query
- **WHEN** the user types characters in SearchInput mode
- **THEN** the system SHALL append characters to the input buffer
- **AND** display the current query in the input prompt

#### Scenario: Backspace in search input
- **WHEN** the user presses `Backspace` in SearchInput mode
- **THEN** the system SHALL remove the last character from the input buffer

#### Scenario: Execute search
- **WHEN** the user presses `Enter` in SearchInput mode
- **AND** the input buffer is not empty
- **THEN** the system SHALL execute the search
- **AND** return to Normal mode
- **AND** highlight all matches in the filtered results
- **AND** jump to the first match

#### Scenario: Clear search with empty query
- **WHEN** the user presses `Enter` in SearchInput mode
- **AND** the input buffer is empty
- **THEN** the system SHALL clear the current search state
- **AND** remove all highlighting
- **AND** return to Normal mode

### Requirement: Case-insensitive search
The system SHALL perform case-insensitive searches using ASCII lowercase matching.

#### Scenario: Case-insensitive match
- **WHEN** the search query is "error"
- **AND** a log line contains "ERROR", "Error", or "error"
- **THEN** the system SHALL consider it a match

### Requirement: Search navigation
The system SHALL allow navigation between search matches using keyboard shortcuts.

#### Scenario: Jump to next match
- **WHEN** the user presses `n` in Normal mode
- **AND** a search is active
- **THEN** the system SHALL jump to the next match in the filtered results
- **AND** update the current match highlighting
- **AND** scroll vertically to bring the match into view

#### Scenario: Jump to previous match
- **WHEN** the user presses `N` in Normal mode
- **AND** a search is active
- **THEN** the system SHALL jump to the previous match in the filtered results
- **AND** update the current match highlighting
- **AND** scroll vertically to bring the match into view

#### Scenario: Wrap around at end
- **WHEN** the user presses `n` at the last match
- **THEN** the system SHALL wrap to the first match

#### Scenario: Wrap around at beginning
- **WHEN** the user presses `N` at the first match
- **THEN** the system SHALL wrap to the last match

### Requirement: Match highlighting
The system SHALL highlight search matches in the filtered log lines.

#### Scenario: Highlight all visible matches
- **WHEN** a search is active
- **THEN** all occurrences of the search query in visible lines SHALL be highlighted
- **AND** the highlight color SHALL be configurable

#### Scenario: Highlight current match distinctly
- **WHEN** a search is active
- **AND** the cursor is positioned at a match
- **THEN** that specific match SHALL be highlighted with distinct styling
- **AND** the current match styling SHALL be configurable

### Requirement: Horizontal auto-scroll
The system SHALL automatically adjust horizontal scroll to bring matches into view.

#### Scenario: Auto-scroll to match
- **WHEN** the user navigates to a match that is off-screen horizontally
- **THEN** the system SHALL adjust the horizontal scroll offset
- **AND** ensure the match is visible with a margin of at least 10 characters

### Requirement: Clear search on filter change
The system SHALL automatically clear search state when filters change.

#### Scenario: Clear on filter modification
- **WHEN** the user adds, removes, or toggles a filter
- **THEN** the system SHALL clear the search query
- **AND** clear all match highlighting
- **AND** reset the current match index
