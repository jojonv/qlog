## ADDED Requirements

### Requirement: FD exhaustion detection
The system SHALL detect file descriptor exhaustion errors (EMFILE, ENFILE) during file operations.

#### Scenario: Detect too many open files
- **WHEN** opening a file or directory fails with EMFILE error
- **THEN** system identifies the error as FD exhaustion
- **AND** system does not crash

#### Scenario: Detect system-wide FD limit
- **WHEN** opening a file or directory fails with ENFILE error
- **THEN** system identifies the error as system-wide FD limit reached

### Requirement: Graceful degradation on FD exhaustion
The system SHALL retry file operations with exponential backoff when FD exhaustion is detected.

#### Scenario: Retry with backoff
- **WHEN** FD exhaustion is detected
- **THEN** system waits 100ms before first retry
- **AND** system doubles wait time on each subsequent failure (100ms, 200ms, 400ms, 800ms)
- **AND** system gives up after 3 failed retries

#### Scenario: User notification on exhaustion
- **WHEN** file loading fails after retries due to FD exhaustion
- **THEN** system displays clear error message: "File descriptor limit reached. Try: ulimit -n 65536"
- **AND** system continues processing remaining files if possible

### Requirement: Non-blocking error handling
The system SHALL continue processing remaining files when individual files fail due to FD exhaustion.

#### Scenario: Continue after single file failure
- **WHEN** one file fails to open due to FD exhaustion after retries
- **THEN** system logs the error
- **AND** system proceeds to next file
- **AND** system displays count of failed files in final summary

#### Scenario: Aggregate failure reporting
- **WHEN** loading completes with some failed files
- **THEN** system reports total files loaded successfully
- **AND** system reports total files that failed
- **AND** system lists first 5 failed file paths as examples

### Requirement: Preventive FD monitoring
The system SHALL monitor FD usage and warn before hitting limits.

#### Scenario: Warning at 80% FD usage
- **WHEN** process FD count exceeds 80% of system limit (e.g., 820 of 1024)
- **THEN** system logs warning message
- **AND** system suggests increasing FD limit

#### Scenario: No warning at safe levels
- **WHEN** process FD count is below 50% of system limit
- **THEN** system does not emit FD-related warnings
