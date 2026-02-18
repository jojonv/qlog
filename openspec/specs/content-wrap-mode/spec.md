## ADDED Requirements

### Requirement: Wrap mode toggle
The system SHALL allow users to toggle between wrapped and non-wrapped content display modes.

#### Scenario: Toggle wrap mode on
- **WHEN** user presses `w` key in Normal mode while wrap mode is off
- **THEN** system enables wrap mode and long lines wrap to next visual line

#### Scenario: Toggle wrap mode off
- **WHEN** user presses `w` key in Normal mode while wrap mode is on
- **THEN** system disables wrap mode and long lines are truncated with horizontal scroll available

### Requirement: Vertical scrollbar
The system SHALL display a vertical scrollbar when content exceeds the viewport height.

#### Scenario: Show vertical scrollbar when content overflows
- **WHEN** number of log entries exceeds viewport height
- **THEN** system displays a vertical scrollbar on the right edge

#### Scenario: Hide vertical scrollbar when content fits
- **WHEN** number of log entries fits within viewport height
- **THEN** system does not display a vertical scrollbar

### Requirement: Horizontal scrollbar in non-wrapped mode
The system SHALL display a horizontal scrollbar when wrap mode is off and content exceeds viewport width.

#### Scenario: Show horizontal scrollbar when wrap off and content wide
- **WHEN** wrap mode is off AND max line width exceeds viewport width
- **THEN** system displays a horizontal scrollbar at the bottom

#### Scenario: Hide horizontal scrollbar when wrap is on
- **WHEN** wrap mode is on
- **THEN** system does not display a horizontal scrollbar

### Requirement: Wrap mode indicator
The system SHALL display the current wrap mode state in the status bar.

#### Scenario: Display wrap mode indicator
- **WHEN** content view is active
- **THEN** status bar shows `[WRAP]` when wrap is on or `[nowrap]` when wrap is off

### Requirement: Dynamic viewport height
The system SHALL calculate viewport height dynamically based on actual terminal size.

#### Scenario: Scroll correctly on large terminal
- **WHEN** terminal height is greater than hardcoded default
- **THEN** system uses actual viewport height for scroll calculations

#### Scenario: Scroll correctly on small terminal
- **WHEN** terminal height is smaller than content
- **THEN** system correctly limits scroll to prevent overscrolling
