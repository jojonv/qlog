## ADDED Requirements

### Requirement: Filter modal can be opened with keyboard shortcut
The system SHALL open the filter modal dialog when user presses the `f` key in normal mode.

#### Scenario: Opening filter modal
- **WHEN** user presses `f` key
- **THEN** the filter modal appears centered on screen
- **AND** the modal dims the background content
- **AND** the modal shows a dropdown for filter type selection

### Requirement: Filter modal supports all filter types
The system SHALL support creating filters of types: Level, Text, Date, and Source Context.

#### Scenario: Creating Level filter
- **WHEN** user selects "Level" from filter type dropdown
- **THEN** the modal displays radio buttons for Error, Warning, and Information
- **AND** user can select one or multiple levels

#### Scenario: Creating Text filter
- **WHEN** user selects "Text" from filter type dropdown
- **THEN** the modal displays a text input field
- **AND** user can enter search text
- **AND** the modal shows a checkbox for "Case sensitive" (default off)
- **AND** the modal shows a checkbox for "Regex" (default off)

#### Scenario: Creating Date filter
- **WHEN** user selects "Date" from filter type dropdown
- **THEN** the modal displays two date input fields: "From" and "To"
- **AND** each field accepts ISO 8601 format (YYYY-MM-DD)
- **AND** fields are optional (blank means no constraint)

#### Scenario: Creating Date filter with relative expressions
- **WHEN** user enters relative date expression in date field
- **THEN** the system converts to absolute timestamp
- **AND** supported expressions include:
  - "now" - current timestamp
  - "today" - start of current day (00:00:00)
  - "yesterday" - start of previous day
  - "last week" - 7 days ago
  - "2 hours ago" - relative hours
  - "3 days ago" - relative days
  - "-1h" - shorthand for hours ago
  - "-30m" - shorthand for minutes ago

#### Scenario: Creating Source Context filter
- **WHEN** user selects "Source Context" from filter type dropdown
- **THEN** the modal displays a text input field with autocomplete
- **AND** autocomplete suggests source contexts found in loaded logs

### Requirement: Filter modal validates input before creation
The system SHALL validate filter input before creating the filter.

#### Scenario: Valid text filter input
- **WHEN** user enters text "failed" and presses Enter
- **THEN** the modal closes
- **AND** a new text filter is added to active filters

#### Scenario: Invalid regex pattern
- **WHEN** user enables regex mode and enters invalid pattern "["
- **THEN** the modal shows error message "Invalid regex pattern"
- **AND** the filter is not created

#### Scenario: Empty filter rejected
- **WHEN** user leaves all fields empty and presses Enter
- **THEN** the modal shows error message "Filter cannot be empty"
- **AND** the filter is not created

### Requirement: Filter modal can be cancelled
The system SHALL allow closing the modal without creating a filter.

#### Scenario: Cancelling with Escape
- **WHEN** user presses Escape key in filter modal
- **THEN** the modal closes
- **AND** no filter is created
- **AND** focus returns to the main view

#### Scenario: Cancelling with 'q'
- **WHEN** user presses 'q' key in filter modal
- **THEN** the modal closes
- **AND** no filter is created

### Requirement: Filter modal shows existing filters
The system SHALL display currently active filters in the modal for reference.

#### Scenario: Viewing active filters in modal
- **WHEN** user opens the filter modal
- **THEN** a sidebar shows current active filters
- **AND** each filter displays its type and value
- **AND** pressing Delete on a filter removes it
