//! Configuration module for log line coloring.
//!
//! This module provides TOML-based configuration for log line coloring.
//! Users can create `.qlog/qlog.toml` in the current directory or home directory
//! with pattern-color mappings.
//!
//! # Example Configuration
//!
//! ```toml
//! [colors]
//! error = "red"
//! warn = "yellow"
//! success = "green"
//! "*TODO*" = "magenta"
//! ```
//!
//! # Pattern Matching
//!
//! - `error` = contains "error" (case-insensitive)
//! - `*error` = ends with "error"
//! - `error*` = starts with "error"
//! - `*error*` = contains "error"
//!
//! First match wins based on config file order.

use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use ratatui::style::{Color, Modifier, Style};

/// Configuration for search highlight colors.
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Foreground color for all match highlights
    pub match_fg: Color,
    /// Background color for all match highlights
    pub match_bg: Color,
    /// Style modifiers for all matches (bold, underline, reverse)
    pub match_style: Style,
    /// Foreground color for the current (active) match
    pub current_fg: Color,
    /// Background color for the current (active) match
    pub current_bg: Color,
    /// Style modifiers for current match
    pub current_style: Style,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            match_fg: Color::Black,
            match_bg: Color::Yellow,
            match_style: Style::default().add_modifier(Modifier::BOLD),
            current_fg: Color::Black,
            current_bg: Color::LightYellow,
            current_style: Style::default().add_modifier(Modifier::BOLD),
        }
    }
}

/// Unified application configuration.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Log line color configuration
    pub colors: ColorConfig,
    /// Search highlight configuration
    pub search: SearchConfig,
}

/// Configuration for log line coloring.
#[derive(Debug, Clone)]
pub struct ColorConfig {
    /// List of pattern-color pairs in order (for first-match-wins semantics)
    patterns: Vec<(PatternMatcher, Color)>,
}

impl ColorConfig {
    /// Load configuration from file.
    ///
    /// Checks `./.qlog/qlog.toml` first, then falls back to `~/.qlog/qlog.toml`.
    /// Returns `None` if no config file is found or if parsing fails.
    pub fn load() -> Option<Self> {
        // Try current directory first
        let local_config = PathBuf::from(".qlog/qlog.toml");
        if local_config.exists() {
            return Self::load_from_path(&local_config);
        }

        // Fall back to home directory
        if let Some(home_dir) = dirs::home_dir() {
            let home_config = home_dir.join(".qlog/qlog.toml");
            if home_config.exists() {
                return Self::load_from_path(&home_config);
            }
        }

        None
    }

    /// Load configuration from a specific path.
    fn load_from_path(path: &PathBuf) -> Option<Self> {
        match fs::read_to_string(path) {
            Ok(content) => Self::parse_toml(&content),
            Err(e) => {
                let _ = writeln!(
                    io::stderr(),
                    "Error reading config file {}: {}",
                    path.display(),
                    e
                );
                None
            }
        }
    }

    /// Parse TOML configuration content.
    fn parse_toml(content: &str) -> Option<Self> {
        // Parse as generic TOML value to preserve order
        let doc = content.parse::<toml::Table>().ok()?;

        let colors_table = doc.get("colors")?.as_table()?;

        let mut patterns = Vec::new();

        for (pattern, color_value) in colors_table {
            let color_str = match color_value.as_str() {
                Some(s) => s,
                None => {
                    let _ = writeln!(
                        io::stderr(),
                        "Invalid color value for pattern '{}': expected string",
                        pattern
                    );
                    continue;
                }
            };

            let color = match parse_color(color_str) {
                Some(c) => c,
                None => {
                    let _ = writeln!(
                        io::stderr(),
                        "Unknown color '{}' for pattern '{}'",
                        color_str,
                        pattern
                    );
                    continue;
                }
            };

            let matcher = PatternMatcher::new(pattern);
            patterns.push((matcher, color));
        }

        if patterns.is_empty() {
            None
        } else {
            Some(Self { patterns })
        }
    }

    /// Get the color for a log line.
    ///
    /// Returns the color of the first matching pattern, or `None` if no patterns match.
    pub fn get_line_color(&self, line: &str) -> Option<Color> {
        for (matcher, color) in &self.patterns {
            if matcher.is_match(line) {
                return Some(*color);
            }
        }
        None
    }
}

/// Pattern matcher for log lines.
///
/// Supports wildcards:
/// - No wildcards: contains match (case-insensitive)
/// - `*` at start: ends with match
/// - `*` at end: starts with match
/// - `*` at both ends: contains match
#[derive(Debug, Clone)]
pub struct PatternMatcher {
    /// The pattern to match
    pattern: String,
    /// Match type
    match_type: MatchType,
}

#[derive(Debug, Clone, Copy)]
enum MatchType {
    /// Pattern must be contained in the line
    Contains,
    /// Line must start with pattern
    StartsWith,
    /// Line must end with pattern
    EndsWith,
    /// Line must exactly match pattern
    Exact,
}

impl PatternMatcher {
    /// Create a new pattern matcher from a pattern string.
    pub fn new(pattern: &str) -> Self {
        let has_leading_wildcard = pattern.starts_with('*');
        let has_trailing_wildcard = pattern.ends_with('*');

        // Strip wildcards and convert to lowercase for case-insensitive matching
        let inner = pattern.trim_matches('*').to_lowercase();

        let (match_type, normalized_pattern) = match (has_leading_wildcard, has_trailing_wildcard) {
            (true, true) => (MatchType::Contains, inner),
            (true, false) => (MatchType::EndsWith, inner),
            (false, true) => (MatchType::StartsWith, inner),
            (false, false) => (MatchType::Contains, inner),
        };

        // Handle edge case of just "*"
        let normalized_pattern = if normalized_pattern.is_empty() {
            pattern.to_lowercase()
        } else {
            normalized_pattern
        };

        // If the normalized pattern is empty (just wildcards), match everything
        let match_type = if normalized_pattern.is_empty() {
            MatchType::Contains
        } else {
            match_type
        };

        Self {
            pattern: normalized_pattern,
            match_type,
        }
    }

    /// Check if a line matches this pattern (case-insensitive).
    pub fn is_match(&self, line: &str) -> bool {
        let line_lower = line.to_lowercase();

        match self.match_type {
            MatchType::Contains => line_lower.contains(&self.pattern),
            MatchType::StartsWith => line_lower.starts_with(&self.pattern),
            MatchType::EndsWith => line_lower.ends_with(&self.pattern),
            MatchType::Exact => line_lower == self.pattern,
        }
    }
}

impl AppConfig {
    /// Load configuration from file.
    ///
    /// Checks `./.qlog/qlog.toml` first, then falls back to `~/.qlog/qlog.toml`.
    /// Returns default configuration if no config file is found.
    pub fn load() -> Option<Self> {
        // Try current directory first
        let local_config = PathBuf::from(".qlog/qlog.toml");
        if local_config.exists() {
            return Self::load_from_path(&local_config);
        }

        // Fall back to home directory
        if let Some(home_dir) = dirs::home_dir() {
            let home_config = home_dir.join(".qlog/qlog.toml");
            if home_config.exists() {
                return Self::load_from_path(&home_config);
            }
        }

        None
    }

    /// Load configuration from a specific path.
    fn load_from_path(path: &PathBuf) -> Option<Self> {
        match fs::read_to_string(path) {
            Ok(content) => Self::parse_toml(&content),
            Err(e) => {
                let _ = writeln!(
                    io::stderr(),
                    "Error reading config file {}: {}",
                    path.display(),
                    e
                );
                None
            }
        }
    }

    /// Parse TOML configuration content.
    fn parse_toml(content: &str) -> Option<Self> {
        let doc = content.parse::<toml::Table>().ok()?;

        // Parse colors section
        let colors = if let Some(colors_table) = doc.get("colors").and_then(|v| v.as_table()) {
            let mut patterns = Vec::new();
            for (pattern, color_value) in colors_table {
                let color_str = match color_value.as_str() {
                    Some(s) => s,
                    None => {
                        let _ = writeln!(
                            io::stderr(),
                            "Invalid color value for pattern '{}': expected string",
                            pattern
                        );
                        continue;
                    }
                };

                let color = match parse_color(color_str) {
                    Some(c) => c,
                    None => {
                        let _ = writeln!(
                            io::stderr(),
                            "Unknown color '{}' for pattern '{}'",
                            color_str,
                            pattern
                        );
                        continue;
                    }
                };

                let matcher = PatternMatcher::new(pattern);
                patterns.push((matcher, color));
            }
            ColorConfig { patterns }
        } else {
            ColorConfig {
                patterns: Vec::new(),
            }
        };

        // Parse search section
        let mut search = SearchConfig::default();
        if let Some(search_table) = doc.get("search").and_then(|v| v.as_table()) {
            if let Some(fg) = search_table.get("match_fg").and_then(|v| v.as_str()) {
                if let Some(color) = parse_color(fg) {
                    search.match_fg = color;
                }
            }
            if let Some(bg) = search_table.get("match_bg").and_then(|v| v.as_str()) {
                if let Some(color) = parse_color(bg) {
                    search.match_bg = color;
                }
            }
            if let Some(style) = search_table.get("match_style").and_then(|v| v.as_str()) {
                search.match_style = parse_style(style);
            }
            if let Some(fg) = search_table.get("current_fg").and_then(|v| v.as_str()) {
                if let Some(color) = parse_color(fg) {
                    search.current_fg = color;
                }
            }
            if let Some(bg) = search_table.get("current_bg").and_then(|v| v.as_str()) {
                if let Some(color) = parse_color(bg) {
                    search.current_bg = color;
                }
            }
            if let Some(style) = search_table.get("current_style").and_then(|v| v.as_str()) {
                search.current_style = parse_style(style);
            }
        }

        Some(Self { colors, search })
    }
}

/// Parse a style string to a ratatui Style.
fn parse_style(style_str: &str) -> Style {
    let mut style = Style::default();
    for modifier in style_str.split_whitespace() {
        style = match modifier.to_lowercase().as_str() {
            "bold" => style.add_modifier(Modifier::BOLD),
            "underline" => style.add_modifier(Modifier::UNDERLINED),
            "reverse" => style.add_modifier(Modifier::REVERSED),
            "italic" => style.add_modifier(Modifier::ITALIC),
            "dim" => style.add_modifier(Modifier::DIM),
            _ => style,
        };
    }
    style
}

/// Parse a color name to a ratatui Color.
fn parse_color(name: &str) -> Option<Color> {
    let color = match name.to_lowercase().as_str() {
        "red" => Color::Red,
        "green" => Color::Green,
        "blue" => Color::Blue,
        "yellow" => Color::Yellow,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "black" => Color::Black,
        "gray" | "grey" => Color::Gray,
        "dark_gray" | "dark_grey" => Color::DarkGray,
        "light_red" => Color::LightRed,
        "light_green" => Color::LightGreen,
        "light_blue" => Color::LightBlue,
        "light_yellow" => Color::LightYellow,
        "light_magenta" => Color::LightMagenta,
        "light_cyan" => Color::LightCyan,
        _ => return None,
    };
    Some(color)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_pattern_matcher_contains() {
        let matcher = PatternMatcher::new("error");
        assert!(matcher.is_match("This is an error message"));
        assert!(matcher.is_match("ERROR: something failed"));
        assert!(matcher.is_match("ApiError occurred"));
        assert!(!matcher.is_match("This is fine"));
    }

    #[test]
    fn test_pattern_matcher_starts_with() {
        let matcher = PatternMatcher::new("error*");
        assert!(matcher.is_match("error occurred"));
        assert!(matcher.is_match("ERROR: something failed"));
        assert!(!matcher.is_match("This is an error"));
    }

    #[test]
    fn test_pattern_matcher_ends_with() {
        let matcher = PatternMatcher::new("*error");
        assert!(matcher.is_match("This is an error"));
        assert!(matcher.is_match("got ERROR"));
        assert!(!matcher.is_match("error occurred"));
    }

    #[test]
    fn test_pattern_matcher_case_insensitive() {
        let matcher = PatternMatcher::new("error");
        assert!(matcher.is_match("ERROR"));
        assert!(matcher.is_match("Error"));
        assert!(matcher.is_match("ErRoR"));
        assert!(matcher.is_match("some ERROR here"));
    }

    #[test]
    fn test_color_config_first_match_wins() {
        let mut patterns = Vec::new();
        patterns.push((PatternMatcher::new("error"), Color::Red));
        patterns.push((PatternMatcher::new("warning"), Color::Yellow));

        let config = ColorConfig { patterns };

        // Line with "error" should get red (first match)
        assert_eq!(config.get_line_color("error warning"), Some(Color::Red));

        // Line with only "warning" should get yellow
        assert_eq!(
            config.get_line_color("warning message"),
            Some(Color::Yellow)
        );
    }

    #[test]
    fn test_parse_color() {
        assert_eq!(parse_color("red"), Some(Color::Red));
        assert_eq!(parse_color("RED"), Some(Color::Red));
        assert_eq!(parse_color("Green"), Some(Color::Green));
        assert_eq!(parse_color("dark_gray"), Some(Color::DarkGray));
        assert_eq!(parse_color("light_cyan"), Some(Color::LightCyan));
        assert_eq!(parse_color("invalid"), None);
    }

    #[test]
    fn test_load_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let qlog_dir = temp_dir.path().join(".qlog");
        fs::create_dir(&qlog_dir).unwrap();

        let config_path = qlog_dir.join("qlog.toml");
        let mut file = fs::File::create(&config_path).unwrap();
        writeln!(
            file,
            r#"[colors]
error = "red"
warn = "yellow"
success = "green""#
        )
        .unwrap();

        let config = ColorConfig::load_from_path(&config_path).unwrap();

        assert_eq!(config.get_line_color("this is an error"), Some(Color::Red));
        assert_eq!(
            config.get_line_color("warning: something"),
            Some(Color::Yellow)
        );
        assert_eq!(config.get_line_color("success!"), Some(Color::Green));
        assert_eq!(config.get_line_color("nothing matches"), None);
    }

    #[test]
    fn test_invalid_toml() {
        let result = ColorConfig::parse_toml("this is not valid toml [");
        assert!(result.is_none());
    }

    #[test]
    fn test_empty_colors() {
        let result = ColorConfig::parse_toml("[colors]");
        assert!(result.is_none());
    }

    #[test]
    fn test_no_colors_section() {
        let result = ColorConfig::parse_toml("[other]\nkey = \"value\"");
        assert!(result.is_none());
    }

    #[test]
    fn test_wildcard_pattern() {
        let matcher = PatternMatcher::new("*TODO*");
        // *TODO* should match lines containing "todo"
        assert!(matcher.is_match("TODO: fix this"));
        assert!(matcher.is_match("fix this TODO"));
        assert!(matcher.is_match("a TODO is here"));
        assert!(!matcher.is_match("nothing here"));
    }
}
