## ADDED Requirements

### Requirement: Streaming file discovery
The system SHALL discover log files incrementally using a streaming iterator with bounded directory handle usage.

#### Scenario: Directory traversal with FD limit
- **WHEN** user specifies a directory pattern with thousands of subdirectories
- **THEN** system opens at most 10 concurrent directory handles during traversal

#### Scenario: Memory-efficient path handling
- **WHEN** discovering files matching a pattern
- **THEN** system does not collect all paths into memory before processing

### Requirement: Incremental file loading
The system SHALL load and parse files as they are discovered without waiting for full directory traversal to complete.

#### Scenario: Progressive loading feedback
- **WHEN** loading a directory with 1000+ log files
- **THEN** system displays progress updates after each file loads
- **AND** user sees logs appearing incrementally

#### Scenario: Single file handle at a time
- **WHEN** processing log files
- **THEN** system holds at most 1 open file handle at any time per loading thread

### Requirement: Memory-mapped large files
The system SHALL use memory-mapped I/O for files exceeding 10MB in size.

#### Scenario: Large file handling
- **WHEN** loading a log file larger than 10MB
- **THEN** system uses mmap for file access
- **AND** system parses the file without reading entire contents into memory

#### Scenario: Small file handling
- **WHEN** loading a log file smaller than 10MB
- **THEN** system uses BufReader for file access

#### Scenario: mmap fallback on error
- **WHEN** memory mapping fails for any reason
- **THEN** system falls back to BufReader automatically

### Requirement: Configurable concurrency limits
The system SHALL allow configuration of maximum concurrent directory handles via environment variable.

#### Scenario: Default FD limit
- **WHEN** user does not specify COMO_MAX_OPEN_DIRS
- **THEN** system limits concurrent directory handles to 10

#### Scenario: Custom FD limit
- **WHEN** user sets COMO_MAX_OPEN_DIRS=5
- **THEN** system limits concurrent directory handles to 5

### Requirement: Bounded resource usage
The system SHALL maintain total file descriptor usage below 50 during normal operation.

#### Scenario: Total FD count verification
- **WHEN** loading any directory structure
- **THEN** total open file descriptors (dirs + files) SHALL NOT exceed 50
- **AND** system remains stable at system FD limits as low as 1024
