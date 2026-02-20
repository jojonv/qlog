## ADDED Requirements

### Requirement: Configuration file discovery
The system SHALL discover configuration files from specific locations in a defined order.

#### Scenario: Current directory config takes precedence
- **WHEN** a file exists at `./.qlog/qlog.toml`
- **THEN** the system SHALL use this configuration

#### Scenario: Home directory config as fallback
- **WHEN** no config exists in the current directory
- **AND** a file exists at `~/.qlog/qlog.toml` (or Windows equivalent)
- **THEN** the system SHALL use the home directory configuration

#### Scenario: No coloring when no config found
- **WHEN** no configuration file exists in either location
- **THEN** the system SHALL apply no coloring to log lines

### Requirement: TOML configuration format
The system SHALL parse TOML configuration files containing color mappings.

#### Scenario: Parse color mappings from TOML
- **WHEN** a configuration file contains a `[colors]` section
- **AND** entries are in format `"pattern" = "color-name"` or `pattern = "color-name"`
- **THEN** the system SHALL extract pattern-color pairs

#### Scenario: Support wildcard patterns
- **WHEN** a pattern contains wildcard characters (`*`)
- **AND** the pattern is in format `"prefix*"`, `"*suffix"`, or `"*infix*"`
- **THEN** the system SHALL interpret wildcards for partial matching

#### Scenario: Validate color names
- **WHEN** a color value is specified
- **THEN** the system SHALL validate it against ratatui color names
- **AND** invalid color names SHALL be ignored

### Requirement: Pattern matching
The system SHALL match log line content against configured patterns using case-insensitive partial matching.

#### Scenario: Case-insensitive matching
- **WHEN** a pattern is "error"
- **AND** a log line contains "ERROR", "Error", or "error"
- **THEN** the system SHALL consider it a match

#### Scenario: Partial substring matching
- **WHEN** a pattern is "error"
- **AND** a log line contains "ApiError", "[error]", or "errorCode"
- **THEN** the system SHALL consider it a match

#### Scenario: Wildcard prefix matching
- **WHEN** a pattern is "*error"
- **AND** a log line ends with "error" or "ERROR"
- **THEN** the system SHALL consider it a match

#### Scenario: Wildcard suffix matching
- **WHEN** a pattern is "error*"
- **AND** a log line starts with "error" or "ERROR"
- **THEN** the system SHALL consider it a match

#### Scenario: Wildcard infix matching
- **WHEN** a pattern is "*error*"
- **AND** a log line contains "error" anywhere
- **THEN** the system SHALL consider it a match

### Requirement: Log line coloring
The system SHALL apply colors to entire log lines based on pattern matches.

#### Scenario: Apply matched pattern color to entire line
- **WHEN** a log line matches a configured pattern
- **THEN** the entire line SHALL be rendered in the corresponding color

#### Scenario: First match wins
- **WHEN** a log line matches multiple patterns
- **THEN** the system SHALL use the color from the first matching pattern in config file order

#### Scenario: No color for unmatched lines
- **WHEN** a log line does not match any configured pattern
- **THEN** the line SHALL be rendered with default styling (no coloring)

### Requirement: Ratatui color support
The system SHALL support standard ratatui color names.

#### Scenario: Basic color names
- **WHEN** a color value is "red", "green", "blue", "yellow", "magenta", "cyan", "white", "black", or "gray"
- **THEN** the system SHALL apply the corresponding ratatui color

#### Scenario: Extended color names
- **WHEN** a color value is "dark_gray", "light_red", "light_green", "light_blue", "light_yellow", "light_magenta", "light_cyan"
- **THEN** the system SHALL apply the corresponding ratatui color
## ADDED Requirements

### Requirement: Search highlight configuration
The system SHALL support configurable search highlight colors and styles in the TOML configuration.

#### Scenario: Parse search highlight colors from TOML
- **WHEN** a configuration file contains a `[search]` section
- **AND** entries include `match_fg`, `match_bg`, `current_fg`, `current_bg`
- **THEN** the system SHALL extract these color values
- **AND** validate them against ratatui color names

#### Scenario: Parse search highlight styles
- **WHEN** a configuration file contains `match_style` or `current_style` in `[search]`
- **AND** values are "bold", "underline", or "reverse"
- **THEN** the system SHALL apply the corresponding style modifiers

#### Scenario: Default search configuration
- **WHEN** no `[search]` section exists in the configuration
- **THEN** the system SHALL use default values:
  - `match_fg`: "black"
  - `match_bg`: "yellow"
  - `current_fg`: "black"
  - `current_bg`: "light_yellow"
  - `match_style`: "bold"
  - `current_style`: "bold"
