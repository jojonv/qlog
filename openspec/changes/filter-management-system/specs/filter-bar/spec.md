## ADDED Requirements

### Requirement: Filter bar displays active filters as chips
The system SHALL display all active filters in a horizontal bar at the top of the main view.

#### Scenario: Multiple filters displayed
- **WHEN** user has active filters: Level=Error, Text="failed"
- **THEN** the filter bar shows: [Error] ● ["failed"]
- **AND** each filter appears as an interactive chip
- **AND** chips show abbreviated filter representation

### Requirement: Filter chips show visual state
The system SHALL visually distinguish between enabled and disabled filters.

#### Scenario: Enabled filter chip
- **WHEN** a filter is enabled
- **THEN** the chip displays with full opacity
- **AND** the chip has a border color matching filter type

#### Scenario: Disabled filter chip
- **WHEN** a filter is disabled (toggled off)
- **THEN** the chip displays at 40% opacity
- **AND** the chip shows strikethrough or dimmed text
- **AND** the chip maintains its position in the bar

### Requirement: Filter chips support keyboard navigation
The system SHALL allow navigating between filter chips using keyboard.

#### Scenario: Navigating filter chips
- **WHEN** user presses Tab or Right Arrow key
- **THEN** selection moves to the next filter chip
- **AND** the selected chip is highlighted with a border

#### Scenario: Reverse navigation
- **WHEN** user presses Shift+Tab or Left Arrow key
- **THEN** selection moves to the previous filter chip

### Requirement: Filter chips can be toggled
The system SHALL allow toggling filter chips on/off without removing them.

#### Scenario: Toggling with Space
- **WHEN** a filter chip is selected
- **AND** user presses Space key
- **THEN** the filter toggles between enabled and disabled state
- **AND** the chip appearance updates immediately

#### Scenario: Toggling with 't'
- **WHEN** user presses 't' key (Helix-style)
- **THEN** the currently selected filter toggles state

### Requirement: Filter chips support NOT (negation) toggle
The system SHALL allow negating any filter to show logs that do NOT match.

#### Scenario: Enabling NOT filter with '!'
- **WHEN** a filter chip is selected
- **AND** user presses '!' key
- **THEN** the filter becomes a NOT filter
- **AND** the chip shows "NOT" prefix or red border
- **AND** the chip displays with red color accent

#### Scenario: Enabling NOT filter with Ctrl+n
- **WHEN** a filter chip is selected
- **AND** user presses Ctrl+n
- **THEN** the filter toggles NOT state

#### Scenario: Disabling NOT filter
- **WHEN** a NOT filter is selected
- **AND** user presses '!' or Ctrl+n again
- **THEN** the filter reverts to normal (positive) matching

### Requirement: Filter chips display match count
The system SHALL show how many log entries match each individual filter.

#### Scenario: Match count displayed
- **WHEN** filter is active and logs are loaded
- **THEN** each filter chip shows: [Error (152)]
- **AND** the count updates live during log loading

#### Scenario: Large match counts
- **WHEN** match count exceeds 100,000
- **THEN** display "99k+" instead of exact count
- **AND** show exact count in tooltip on hover/focus

#### Scenario: Match count disabled
- **WHEN** loading more than 1M logs
- **THEN** match counts are temporarily disabled
- **AND** chips show only filter value without count

### Requirement: Filter chips can be deleted
The system SHALL allow removing filter chips.

#### Scenario: Deleting with 'd'
- **WHEN** a filter chip is selected
- **AND** user presses 'd' key
- **THEN** the filter is removed from active filters
- **AND** the filter bar updates immediately

#### Scenario: Deleting with Delete key
- **WHEN** a filter chip is selected
- **AND** user presses Delete key
- **THEN** the filter is removed
- **AND** focus moves to the next chip (or previous if last)

### Requirement: Filter bar shows "Clear All" option
The system SHALL provide a way to remove all filters at once.

#### Scenario: Clearing all filters
- **WHEN** user presses 'F' (shift+f)
- **THEN** all active filters are removed
- **AND** the filter bar becomes empty
- **AND** a confirmation message "All filters cleared" appears briefly

#### Scenario: Clearing with confirmation
- **GIVEN** there are more than 3 active filters
- **WHEN** user presses 'F'
- **THEN** a confirmation dialog appears: "Clear all 5 filters? (y/n)"
- **AND** only after 'y' are filters removed
