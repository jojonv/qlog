## ADDED Requirements

### Requirement: String matching SHALL use Boyer-Moore-Horspool algorithm
The system SHALL implement Boyer-Moore-Horspool algorithm for all substring matching operations in filter evaluation, replacing naive character-by-character comparison.

#### Scenario: Pattern matching with BMH algorithm
- **WHEN** a filter pattern is evaluated against a log line
- **THEN** the system uses Boyer-Moore-Horspool algorithm with O(n/m) average-case complexity instead of O(n×m) naive search

### Requirement: Pattern matching SHALL operate on bytes without allocation
The system SHALL perform all pattern matching operations on raw byte slices without converting log lines to UTF-8 strings, eliminating per-line heap allocations.

#### Scenario: Byte-based matching without String conversion
- **WHEN** a filter is applied to a log line
- **THEN** the system matches the pattern against the raw byte slice of the line
- **AND** no String allocation occurs during the comparison

### Requirement: Case-insensitive matching SHALL use byte-level case folding
The system SHALL implement case-insensitive matching using ASCII byte-level case folding (A-Z ↔ a-z) without requiring UTF-8 conversion.

#### Scenario: ASCII case-insensitive matching
- **WHEN** a filter with case-insensitive matching is applied
- **THEN** the system matches patterns ignoring ASCII case (A-Z matches a-z)
- **AND** the matching operates directly on bytes without string conversion

## MODIFIED Requirements

### Requirement: Filter evaluation SHALL use optimized matching algorithm
Filter groups SHALL use the optimized Boyer-Moore-Horspool string matching algorithm for evaluating Contains predicates, maintaining existing behavior while improving performance.

#### Scenario: Filter with substring matching
- **WHEN** a filter group is evaluated with Contains predicate
- **THEN** the system uses Boyer-Moore-Horspool algorithm to check if the pattern exists in the log line
- **AND** the result is identical to naive substring search

#### Scenario: Case-insensitive filter matching
- **WHEN** a filter with case-insensitive Contains is evaluated
- **THEN** the system matches the lowercase pattern against lowercase log line content
- **AND** the result matches the previous case-insensitive behavior

### Requirement: AND-combined filters SHALL implement early termination
When evaluating filters with AND-combined groups, the system SHALL terminate evaluation early upon first non-matching group, avoiding unnecessary pattern comparisons.

#### Scenario: Early termination on first mismatch
- **WHEN** a filter with multiple AND-combined groups is evaluated
- **AND** the first group does not match the log line
- **THEN** evaluation terminates immediately
- **AND** subsequent groups are not evaluated

#### Scenario: Full evaluation when all groups match
- **WHEN** a filter with multiple AND-combined groups is evaluated
- **AND** all groups match the log line
- **THEN** evaluation continues through all groups
- **AND** the filter reports a match
