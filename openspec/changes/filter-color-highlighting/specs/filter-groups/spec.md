## ADDED Requirements

### Requirement: Filter chips display group colors

The system SHALL render each filter chip using the color assigned to its group.

#### Scenario: Filter chip color matches group

- **WHEN** a filter is rendered in the filter bar
- **THEN** its text color matches the assigned group color

### Requirement: Selected filter visual indicator

The system SHALL display a clear visual indicator on the currently selected filter.

#### Scenario: Selected filter background highlight

- **WHEN** a filter is selected (cursor on it)
- **THEN** it displays with a dark background and bold styling in addition to its group color

### Requirement: Active group context indicator

The system SHALL visually indicate all filters in the currently active group during navigation.

#### Scenario: Group navigation shows group context

- **WHEN** user navigates between groups with h/l keys
- **THEN** all filters in the active group display with a subtle background tint

### Requirement: Group separator styling

The system SHALL render group separators distinctly to enhance visual grouping.

#### Scenario: Group separator neutral color

- **WHEN** group separators are rendered
- **THEN** they use a neutral color (white or gray) that does not compete with group colors
