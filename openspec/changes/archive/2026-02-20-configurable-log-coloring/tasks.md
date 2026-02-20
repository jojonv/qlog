## 1. Setup Dependencies

- [x] 1.1 Add `toml = "0.8"` to Cargo.toml dependencies
- [x] 1.2 Add `dirs = "5.0"` to Cargo.toml dependencies
- [x] 1.3 Run `cargo check` to verify dependencies resolve

## 2. Create Configuration Module

- [x] 2.1 Create `src/config.rs` with `ColorConfig` struct containing `colors: HashMap<String, String>`
- [x] 2.2 Implement `ColorConfig::load() -> Option<ColorConfig>` that checks `./.qlog/qlog.toml` then `~/.qlog/qlog.toml`
- [x] 2.3 Implement TOML parsing with error handling (log errors to stderr, return None on failure)
- [x] 2.4 Create `PatternMatcher` struct that compiles patterns with wildcards into matchers
- [x] 2.5 Implement `PatternMatcher::is_match(line: &str) -> bool` with case-insensitive matching
- [x] 2.6 Add unit tests for config loading, TOML parsing, and pattern matching

## 3. Integrate Config into App

- [x] 3.1 Add `config: Option<ColorConfig>` field to `App` struct in `src/app.rs`
- [x] 3.2 Load config in `App::new()` using `ColorConfig::load()`
- [x] 3.3 Create `App::get_line_color(line: &str) -> Option<Color>` method

## 4. Modify UI Rendering

- [x] 4.1 In `src/ui/mod.rs`, modify `draw_main_view()` to check for line color
- [x] 4.2 Apply color to entire line when pattern matches (use color for foreground)
- [x] 4.3 Ensure selection highlight (dark gray background) still works and takes precedence
- [x] 4.4 Verify timestamps remain cyan regardless of line color

## 5. Testing and Validation

- [x] 5.1 Create sample `.qlog/qlog.toml` with test patterns:
  ```toml
  [colors]
  error = "red"
  warn = "yellow"
  success = "green"
  "*TODO*" = "magenta"
  ```
- [x] 5.2 Test case-insensitive matching: "error" matches "ERROR", "Error", "ApiError"
- [x] 5.3 Test wildcard patterns: "*error" matches lines ending with error
- [x] 5.4 Test first-match-wins: line "error warning" uses error's color (first in config)
- [x] 5.5 Test selection override: colored line still shows selection highlight
- [x] 5.6 Test fallback to home directory: move config to `~/.qlog/qlog.toml`
- [x] 5.7 Test no coloring when no config: ensure app runs normally without config

## 6. Documentation

- [x] 6.1 Add CONFIGURATION.md with examples and color name reference
- [x] 6.2 Update README.md with brief mention of coloring feature
- [x] 6.3 Add inline code documentation for public config module methods
