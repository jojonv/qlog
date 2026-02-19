# Optimized String Matching

## Purpose

The optimized string matching capability provides high-performance substring searching operations for log file filtering. It uses the Boyer-Moore-Horspool algorithm for efficient pattern matching while avoiding memory allocations through byte-level operations.

## Requirements

### Requirement: String matching SHALL use Boyer-Moore-Horspool algorithm
The system SHALL implement Boyer-Moore-Horspool algorithm for all substring matching operations in filter evaluation, replacing naive character-by-character comparison. The pattern SHALL be preprocessed to build a skip table, and patterns SHALL be cached as lowercase bytes for case-insensitive matching.

#### Scenario: Pattern matching with BMH algorithm
- **WHEN** a filter pattern is evaluated against a log line
- **THEN** the system uses Boyer-Moore-Horspool algorithm with O(n/m) average-case complexity instead of O(n×m) naive search
- **AND** the pattern is preprocessed once to build a skip table (O(m) time)
- **AND** for case-insensitive matching, the pattern is lowercased once during preprocessing

#### Scenario: Skip table preprocessing
- **WHEN** a filter is created or modified
- **THEN** the system builds a skip table mapping each byte to its skip distance
- **AND** for case-insensitive patterns, the table uses lowercase byte values
- **AND** the skip table enables O(n/m) average-case search performance

### Requirement: Pattern matching SHALL operate on bytes without allocation
The system SHALL perform all pattern matching operations on raw byte slices without converting log lines to UTF-8 strings, eliminating per-line heap allocations. Pattern matching SHALL use a thread-local buffer for case folding to avoid per-line allocation.

#### Scenario: Byte-based matching without String conversion
- **WHEN** a filter is applied to a log line
- **THEN** the system matches the pattern against the raw byte slice of the line
- **AND** no String allocation occurs during the comparison
- **AND** a thread-local byte buffer is reused for case folding operations

#### Scenario: Thread-local buffer for case folding
- **WHEN** case-insensitive matching is performed
- **THEN** the system uses a thread-local buffer with 8KB initial capacity
- **AND** the buffer is cleared and reused for each line
- **AND** no per-line heap allocation occurs

### Requirement: Case-insensitive matching SHALL use byte-level case folding
The system SHALL implement case-insensitive matching using ASCII byte-level case folding (A-Z ↔ a-z) without requiring UTF-8 conversion. The system SHALL pre-lowercase the entire line once using the thread-local buffer before running Boyer-Moore-Horspool search.

#### Scenario: ASCII case-insensitive matching
- **WHEN** a filter with case-insensitive matching is applied
- **THEN** the system matches patterns ignoring ASCII case (A-Z matches a-z)
- **AND** the matching operates directly on bytes without string conversion
- **AND** the entire line is lowercased once before BMH search begins
- **AND** pure BMH search runs on the pre-lowercased buffer

#### Scenario: Two-phase case folding and search
- **WHEN** case-insensitive pattern matching is performed
- **THEN** Phase 1: The line is lowercased into the thread-local buffer (O(n))
- **AND** Phase 2: Boyer-Moore-Horspool algorithm runs on the buffer (O(n/m))
- **AND** Total complexity is O(n) + O(n/m) instead of O(n×m) for naive approach

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
