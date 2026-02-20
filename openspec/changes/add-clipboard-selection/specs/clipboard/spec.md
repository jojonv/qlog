## ADDED Requirements

### Requirement: User can copy selected lines to system clipboard
The system SHALL support copying selected log lines to the system clipboard.

#### Scenario: Copy requires explicit selection
- **WHEN** user has no active selection
- **AND** user presses `y`
- **THEN** no copy operation is performed
- **AND** no feedback is shown

#### Scenario: Copy selected lines
- **WHEN** user has selected lines 3-5
- **AND** user presses `y`
- **THEN** lines 3, 4, and 5 SHALL be copied to the system clipboard
- **AND** the lines SHALL be joined with newline characters
- **AND** a status message SHALL display "Copied 3 lines to clipboard"

#### Scenario: Copy raw log content
- **WHEN** user copies selected lines
- **THEN** the copied content SHALL be the raw log line text
- **AND** SHALL NOT include any visual formatting or wrapping

#### Scenario: Clipboard unavailable error
- **WHEN** the system clipboard is unavailable (e.g., headless system)
- **AND** user presses `y` with an active selection
- **THEN** an error message SHALL display "Clipboard unavailable"
- **AND** the selection SHALL remain active

### Requirement: Clipboard handles initialization gracefully
The system SHALL gracefully handle clipboard initialization failures.

#### Scenario: App starts without clipboard
- **WHEN** the application starts on a system without display/clipboard support
- **THEN** the application SHALL continue to function normally
- **AND** clipboard-related features SHALL be unavailable
- **AND** pressing `y` SHALL show the unavailable error

## MODIFIED Requirements

*No existing requirements are being modified for this capability*

## REMOVED Requirements

*No requirements are being removed for this capability*
