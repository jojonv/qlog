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
