## Purpose

Memory-efficient log file storage using memory-mapped I/O with zero-copy access patterns and lazy visual line calculation.

## Requirements

### Requirement: Memory-mapped log storage

The system SHALL store log files using memory-mapped I/O with a line index for O(1) random access.

#### Scenario: Load log file
- **WHEN** a log file is loaded
- **THEN** the system creates a memory mapping for the file
- **AND** builds a line index with offset and length for each line
- **AND** does NOT copy log content into heap-allocated strings

#### Scenario: Random line access
- **WHEN** a specific line index is requested
- **THEN** the system returns a view into the memory-mapped data
- **AND** the view is returned in O(1) time
- **AND** no heap allocation occurs for the line content

#### Scenario: Large file handling
- **WHEN** a 2GB log file is loaded
- **THEN** memory usage SHALL NOT exceed 2.5GB
- **AND** the file loads successfully
- **AND** all lines are accessible

### Requirement: Zero-copy string views

The system SHALL provide string views into memory-mapped data without copying.

#### Scenario: Access line content
- **WHEN** a line's content is accessed for display
- **THEN** the system provides a view referencing the mmap
- **AND** no String allocation occurs
- **AND** the view is valid as long as the storage exists

#### Scenario: UTF-8 lossy conversion
- **WHEN** a line contains invalid UTF-8 bytes
- **THEN** the system replaces invalid bytes with the replacement character (�)
- **AND** no panic occurs
- **AND** the line is still displayable

### Requirement: Case-insensitive filtering without allocation

The system SHALL perform case-insensitive filtering without allocating temporary strings.

#### Scenario: Filter text caching
- **WHEN** a filter is created or modified
- **THEN** the system converts the filter text to lowercase bytes once
- **AND** caches the lowercase bytes for reuse

#### Scenario: Filter matching
- **WHEN** a line is checked against a filter
- **THEN** the system compares bytes directly without creating new strings
- **AND** matching is case-insensitive for ASCII characters
- **AND** no heap allocation occurs during matching

### Requirement: Lazy visual line calculation

The system SHALL calculate visual line offsets only for lines near the viewport.

#### Scenario: Initial viewport
- **WHEN** the application displays logs
- **THEN** visual line offsets are calculated only for visible lines
- **AND** a small buffer of lines above and below
- **AND** not for all filtered lines

#### Scenario: Scroll to new area
- **WHEN** the user scrolls to a new area
- **THEN** visual line offsets are calculated on-demand for the new visible area
- **AND** previously calculated offsets may be discarded from cache

### Requirement: Unified mmap approach

The system SHALL use memory-mapped files for all log files regardless of size.

#### Scenario: Small file loading
- **WHEN** a log file smaller than 100MB is loaded
- **THEN** the system uses the same mmap approach as large files
- **AND** no special in-memory path is used

#### Scenario: Empty file handling
- **WHEN** an empty log file is loaded
- **THEN** the system creates an empty line index
- **AND** no errors occur
