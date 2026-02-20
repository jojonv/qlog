## ADDED Requirements

### Requirement: User can select lines using Helix-style selection
The system SHALL support Helix-style line selection with single and multi-line capabilities.

#### Scenario: Starting selection
- **WHEN** user presses `x` with no active selection
- **THEN** the current line under cursor becomes selected
- **AND** the selection is visually highlighted

#### Scenario: Extending selection downward
- **WHEN** user has an active selection on line 5
- **AND** user presses `j` to move cursor down
- **THEN** the selection range extends to include line 6

#### Scenario: Extending selection upward
- **WHEN** user has an active selection on line 5
- **AND** user presses `k` to move cursor up
- **THEN** the selection range extends to include line 4

#### Scenario: Extending selection with repeated x
- **WHEN** user has an active selection on line 5
- **AND** user presses `x` again
- **THEN** the selection extends to line 6 in the same direction as the previous extension

#### Scenario: Clearing selection with Escape
- **WHEN** user has an active selection on lines 3-7
- **AND** user presses `Escape`
- **THEN** the selection is cleared
- **AND** only the cursor position remains (single line)

### Requirement: Selection is cleared when filters change
The system SHALL clear any active selection when filters are modified.

#### Scenario: Filter change clears selection
- **WHEN** user has selected lines 10-15
- **AND** user changes the filter criteria
- **THEN** the selection is cleared
- **AND** the cursor is positioned at the first matching line

### Requirement: Selection persists during search navigation
The system SHALL maintain the selection when using search navigation.

#### Scenario: Search navigation preserves selection
- **WHEN** user has selected lines 5-10
- **AND** user presses `n` to jump to next match
- **THEN** the selection range remains active

### Requirement: Selection is visually indicated
The system SHALL visually distinguish selected lines from unselected lines.

#### Scenario: Visual selection highlighting
- **WHEN** user selects lines 3-7
- **THEN** lines 3, 4, 5, 6, and 7 SHALL have a DarkGray background
- **AND** line 8 SHALL have the default background

## MODIFIED Requirements

*No existing requirements are being modified for this capability*

## REMOVED Requirements

*No requirements are being removed for this capability*
