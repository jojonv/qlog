## ADDED Requirements

### Requirement: Content mode for log navigation

The system SHALL provide a Content mode where j/k/h/l keys control log content scrolling, not filter navigation.

#### Scenario: Scroll down with j
- **WHEN** user is in Content mode and presses `j`
- **THEN** selected_line SHALL increment by 1 (scrolling down)

#### Scenario: Scroll up with k
- **WHEN** user is in Content mode and presses `k`
- **THEN** selected_line SHALL decrement by 1 (scrolling up)

#### Scenario: Scroll right with l
- **WHEN** user is in Content mode and presses `l`
- **THEN** horizontal_scroll SHALL increment by some amount

#### Scenario: Scroll left with h
- **WHEN** user is in Content mode and presses `h`
- **THEN** horizontal_scroll SHALL decrement by some amount

### Requirement: Filter mode for filter management

The system SHALL provide a Filter mode where j/k/h/l keys navigate and manage filters.

#### Scenario: Select next filter with j
- **WHEN** user is in Filter mode and presses `j`
- **THEN** selected_filter SHALL increment by 1 within current group

#### Scenario: Select previous filter with k
- **WHEN** user is in Filter mode and presses `k`
- **THEN** selected_filter SHALL decrement by 1 within current group

#### Scenario: Switch to next group with l
- **WHEN** user is in Filter mode and presses `l`
- **THEN** selected_group SHALL increment by 1

#### Scenario: Switch to previous group with h
- **WHEN** user is in Filter mode and presses `h`
- **THEN** selected_group SHALL decrement by 1

### Requirement: Mode switching with t

The system SHALL allow toggling between Content and Filter modes using the `t` key.

#### Scenario: Enter Filter mode
- **WHEN** user is in Content mode and presses `t`
- **THEN** system SHALL switch to Filter mode

#### Scenario: Return to Content mode with t
- **WHEN** user is in Filter mode and presses `t`
- **THEN** system SHALL switch to Content mode

### Requirement: Return to Content mode with Escape

The system SHALL allow returning to Content mode from Filter mode using the `Esc` key.

#### Scenario: Escape returns to Content mode
- **WHEN** user is in Filter mode and presses `Esc`
- **THEN** system SHALL switch to Content mode

### Requirement: Content mode is default

The system SHALL start in Content mode when the application launches.

#### Scenario: Application starts in Content mode
- **WHEN** application launches
- **THEN** mode SHALL be Content mode (Normal)

### Requirement: Current line highlighting

The system SHALL visually highlight the currently selected line in the content view.

#### Scenario: Selected line is highlighted
- **WHEN** user is viewing logs in Content mode
- **THEN** the line at selected_line SHALL be visually distinct from other lines
