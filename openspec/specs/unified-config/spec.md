## ADDED Requirements

### Requirement: Unified configuration structure
The system SHALL support a unified configuration structure that consolidates all settings.

#### Scenario: Load unified configuration
- **WHEN** the application starts
- **THEN** the system SHALL load configuration from `AppConfig::load()`
- **AND** the configuration SHALL include both color and search settings

#### Scenario: Backward compatibility
- **WHEN** loading existing configurations
- **THEN** the system SHALL maintain backward compatibility with `ColorConfig`
- **AND** gracefully handle missing search configuration sections

### Requirement: Configuration sections
The system SHALL support distinct sections for different configuration concerns.

#### Scenario: Colors section
- **WHEN** a configuration file contains a `[colors]` section
- **THEN** the system SHALL parse it as log line color mappings

#### Scenario: Search section
- **WHEN** a configuration file contains a `[search]` section
- **THEN** the system SHALL parse it as search highlight settings

### Requirement: Search highlight configuration
The system SHALL support configurable search highlight colors and styles.

#### Scenario: Configure match highlight colors
- **WHEN** a configuration file contains `match_fg` and `match_bg` in `[search]`
- **THEN** the system SHALL use these colors for all match highlighting

#### Scenario: Configure current match styling
- **WHEN** a configuration file contains `current_fg`, `current_bg`, or `current_style` in `[search]`
- **THEN** the system SHALL use these for the current match highlight

#### Scenario: Default search colors
- **WHEN** no search configuration is provided
- **THEN** the system SHALL use sensible defaults (yellow background for matches, bright yellow for current)
