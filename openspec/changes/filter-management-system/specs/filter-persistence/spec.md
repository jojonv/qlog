## ADDED Requirements

### Requirement: Filters are saved on exit
The system SHALL automatically save active filter configuration when the application exits.

#### Scenario: Auto-save on quit
- **GIVEN** user has active filters: [Level: Error], [Text: "failed"]
- **WHEN** user presses 'q' to quit the application
- **THEN** the filter configuration is saved to disk
- **AND** the save location is ~/.config/como-log-viewer/filters.json

### Requirement: Filters are restored on startup
The system SHALL restore the last active filter configuration when the application starts.

#### Scenario: Restore on startup
- **GIVEN** previous session had filters: [Level: Error], [Date: Today]
- **WHEN** user launches the application
- **THEN** the filter bar shows the restored filters
- **AND** the filters are applied to the loaded logs

#### Scenario: Restore with missing log files
- **GIVEN** a saved filter references a specific Source Context
- **WHEN** the new log files don't contain that source
- **THEN** the filter is still restored and shown
- **AND** it shows 0 matches but remains active

### Requirement: Filter configuration file format
The system SHALL use JSON format for persistence with defined schema.

#### Scenario: JSON structure
- **GIVEN** filters are saved
- **THEN** the JSON file contains:
  - `active_filters`: array of filter objects
  - `operators`: array of "AND" or "OR" strings (N-1 entries for N filters)
  - `version`: file format version number
  - `saved_at`: ISO 8601 timestamp

#### Scenario: Filter object structure
- **GIVEN** a text filter is saved
- **THEN** the filter object contains:
  - `type`: "text"
  - `value`: the search string
  - `case_sensitive`: boolean
  - `regex`: boolean
  - `enabled`: boolean

### Requirement: Save location follows XDG conventions
The system SHALL save filter configuration in the appropriate XDG directory.

#### Scenario: Linux save location
- **WHEN** the application saves filters on Linux
- **THEN** the file is saved to `$XDG_CONFIG_HOME/como-log-viewer/filters.json`
- **OR** if XDG_CONFIG_HOME is unset, `~/.config/como-log-viewer/filters.json`

#### Scenario: Directory creation
- **GIVEN** the config directory does not exist
- **WHEN** the application attempts to save
- **THEN** it creates the directory recursively
- **AND** sets appropriate permissions (700)

### Requirement: Manual save and load commands
The system SHALL support explicit save/load commands for filter configurations.

#### Scenario: Manual save command
- **WHEN** user presses `:save` in command mode
- **THEN** a dialog appears asking for preset name
- **AND** user enters "production-errors"
- **THEN** the current filters are saved as a named preset

#### Scenario: Manual load command
- **WHEN** user presses `:load` in command mode
- **THEN** a dialog shows list of saved presets
- **AND** user selects "production-errors"
- **THEN** those filters are loaded and applied

### Requirement: Handle corrupted save files gracefully
The system SHALL handle corrupted or malformed save files without crashing.

#### Scenario: Corrupted JSON
- **GIVEN** the filters.json file is corrupted (invalid JSON)
- **WHEN** application starts
- **THEN** it logs a warning about corrupted config
- **AND** starts with no filters (empty state)
- **AND** does not crash

#### Scenario: Missing version field
- **GIVEN** the filters.json is missing the version field
- **WHEN** application starts
- **THEN** it attempts to load assuming current version
- **AND** ignores unrecognized fields gracefully

### Requirement: Filter history tracking
The system SHALL track the last 20 unique filter combinations for quick re-application.

#### Scenario: Automatic history tracking
- **WHEN** user applies a new filter combination
- **THEN** it is added to the history if different from the last entry
- **AND** history is limited to 20 entries (oldest removed when full)
- **AND** history is persisted to ~/.config/como-log-viewer/history.json

#### Scenario: Accessing history with Ctrl+h
- **WHEN** user presses Ctrl+h
- **THEN** a history popup appears showing recent filter combinations
- **AND** each entry shows: timestamp, filter summary, match count
- **AND** user can navigate with j/k and press Enter to apply

#### Scenario: Accessing history with :history command
- **WHEN** user enters `:history` in command mode
- **THEN** the same history popup appears

#### Scenario: History entry format
- **GIVEN** history entries are saved
- **THEN** each entry contains:
  - `timestamp`: ISO 8601 when filter was applied
  - `filters`: array of filter objects
  - `operators`: array of AND/OR operators
  - `summary`: short description (e.g., "Error + failed")
  - `match_count`: number of matching logs (if available)
