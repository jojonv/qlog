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
