## ADDED Requirements

### Requirement: Log lines highlight filter matches

The system SHALL highlight substrings in log lines that match active filters, using the color of the filter's group.

#### Scenario: Single filter match highlighted

- **WHEN** a log line contains text matching an active filter
- **THEN** the matched substring is rendered in the filter's group color

#### Scenario: Multiple filter matches from same group

- **WHEN** a log line contains multiple matches from filters in the same group
- **THEN** each matched substring is rendered in the group's color

#### Scenario: Matches from different groups

- **WHEN** a log line contains matches from filters in different groups
- **THEN** each match uses its respective group's color

#### Scenario: Overlapping matches

- **WHEN** two filters match overlapping text in a log line
- **THEN** the first match (by start position) takes precedence and the overlapping portion is not double-highlighted

#### Scenario: Case-insensitive matching

- **WHEN** a filter is "error" and a log line contains "ERROR" or "Error"
- **THEN** the matched text is highlighted regardless of case

#### Scenario: Disabled filter no highlight

- **WHEN** a filter is disabled
- **THEN** its matches are not highlighted in log lines

### Requirement: Group color palette

The system SHALL assign distinct colors to each filter group from a fixed palette.

#### Scenario: First group color

- **WHEN** a filter belongs to group index 0
- **THEN** it uses the first palette color (Cyan)

#### Scenario: Palette cycles for many groups

- **WHEN** there are more than 6 groups
- **THEN** colors cycle back through the palette (group 6 uses Cyan, etc.)
