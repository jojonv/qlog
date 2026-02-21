## MODIFIED Requirements

### Requirement: Filter groups
The system SHALL organize filters into groups where filters within a group are combined with OR logic, and groups are combined with AND logic.

#### Scenario: OR within group
- **WHEN** a log line contains "error" OR "warning"
- **THEN** the line matches a group containing both filters

#### Scenario: AND between groups
- **WHEN** a log line matches all groups
- **THEN** the line passes the entire filter set

### Requirement: Filter UI Display
The system SHALL display active filters in a visual filter bar at the top of the screen.

#### Scenario: Filter bar visibility
- **WHEN** filter mode is active
- **THEN** the filter bar shows all groups and filters

#### Scenario: Navigation feedback
- **WHEN** user navigates between groups or filters
- **THEN** the UI highlights the current selection

### Requirement: Filter Management Keys
The system SHALL support keyboard shortcuts for managing filters.

#### Scenario: Group navigation
- **WHEN** user presses 'h' or 'l'
- **THEN** selection moves between filter groups

#### Scenario: Filter navigation
- **WHEN** user presses 'j' or 'k'
- **THEN** selection moves between filters within current group

#### Scenario: Toggle filter
- **WHEN** user presses Space
- **THEN** the selected filter toggles enabled/disabled state

#### Scenario: Add filter
- **WHEN** user presses 'f'
- **THEN** system enters filter input mode for current group

#### Scenario: Add group
- **WHEN** user presses 'F'
- **THEN** system creates new group and enters filter input mode

#### Scenario: Delete filter
- **WHEN** user presses 'd'
- **THEN** the selected filter is removed from its group

### Requirement: Filter Input Mode
The system SHALL provide a mode for entering new filter text.

#### Scenario: Enter filter text
- **WHEN** user types text and presses Enter
- **THEN** a new filter is created with the entered text

## ADDED Requirements

### Requirement: Command-based filter management
The system SHALL support adding filters via command interface.

#### Scenario: Add include filter
- **WHEN** user enters `:filter <text>`
- **THEN** a new include filter is added to the active filter list

#### Scenario: Add exclude filter
- **WHEN** user enters `:filter-out <text>`
- **THEN** a new exclude filter is added to the active filter list

#### Scenario: Clear all filters
- **WHEN** user enters `:filter-clear`
- **THEN** all active filters are removed

### Requirement: Filter list display
The system SHALL provide a command to view and manage active filters.

#### Scenario: List filters
- **WHEN** user enters `:list-filters`
- **THEN** an interactive overlay displays all active filters with index numbers

#### Scenario: Delete from list
- **WHEN** user navigates to a filter and presses 'd'
- **THEN** the selected filter is removed from the active list

#### Scenario: Close list view
- **WHEN** user presses 'q' or Enter
- **THEN** the filter list overlay closes

### Requirement: Status bar filter indicator
The system SHALL display active filter count in the status bar.

#### Scenario: Show filter count
- **WHEN** filters are active
- **THEN** status bar shows "Filters: N active"

## REMOVED Requirements

### Requirement: Visual filter mode
**Reason**: Replaced by command-based filter interface
**Migration**: Use `:filter <text>` to add filters, `:list-filters` to view/manage

### Requirement: Filter group navigation
**Reason**: Flat filter list replaces hierarchical groups
**Migration**: Filters are now a single list; use `:filter` and `:filter-out` commands

### Requirement: Filter toggle
**Reason**: Command-based filters are active until deleted
**Migration**: Use `:list-filters` and delete individual filters with 'd'
