## ADDED Requirements

### Requirement: Filters support explicit AND/OR operators
The system SHALL support combining filters with AND or OR logic operators.

#### Scenario: Default AND between filters
- **WHEN** user creates two filters: [Level: Error] and [Text: "failed"]
- **THEN** the filter bar shows: [Error] ● ["failed"]
- **AND** the ● symbol indicates AND logic
- **AND** only logs matching BOTH filters are displayed

#### Scenario: OR operator between filters
- **GIVEN** filters: [Level: Error] ● [Text: "failed"]
- **WHEN** user clicks on the ● symbol
- **THEN** it changes to ○ indicating OR logic
- **AND** the filter bar shows: [Error] ○ ["failed"]
- **AND** logs matching EITHER filter are displayed

#### Scenario: Mixed operators
- **WHEN** user has three filters with mixed logic: [Error] ● ["failed"] ○ [Today]
- **THEN** the system evaluates as: (Error AND "failed") OR Today
- **AND** matching logs are displayed accordingly

### Requirement: Operator symbols are clickable/toggleable
The system SHALL allow changing operators by interacting with the symbols.

#### Scenario: Toggling operator with click
- **WHEN** user clicks on ● symbol between two filters
- **THEN** the symbol changes to ○
- **AND** the logic switches from AND to OR

#### Scenario: Toggling operator with Enter
- **WHEN** user navigates to an operator symbol with arrow keys
- **AND** user presses Enter
- **THEN** the symbol toggles between ● and ○

### Requirement: Logic operators are visually distinct
The system SHALL clearly distinguish AND and OR operators.

#### Scenario: AND operator appearance
- **WHEN** operator is AND
- **THEN** it displays as filled circle: ●
- **AND** it uses color matching the first filter

#### Scenario: OR operator appearance
- **WHEN** operator is OR
- **THEN** it displays as hollow circle: ○
- **AND** it uses neutral/gray color

### Requirement: Filter logic affects results immediately
The system SHALL update the log display immediately when logic changes.

#### Scenario: Changing logic updates results
- **GIVEN** [Error] ● ["failed"] shows 100 matches
- **WHEN** user changes to [Error] ○ ["failed"]
- **THEN** the match count updates immediately
- **AND** the log list refreshes to show expanded results
