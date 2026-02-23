use crate::model::FilterKind;
use chrono::Local;

const COMMANDS: &[&str] = &[
    "filter",
    "filter-clear",
    "filter-out",
    "list-filters",
    "quit",
    "write",
];

#[derive(Debug, Clone, PartialEq)]
pub enum CommandEffect {
    Quit,
    AddFilter { kind: FilterKind, pattern: String },
    ClearFilters,
    WriteFilteredLogs { filename: String },
    ListFilters,
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub effect: Option<CommandEffect>,
    pub status: String,
}

pub fn parse(input: &str) -> CommandResult {
    let (cmd, arg) = split_command(input);

    match cmd {
        "q" | "quit" => CommandResult {
            effect: Some(CommandEffect::Quit),
            status: String::new(),
        },
        "w" | "write" => {
            let filename = arg.map(|s| s.to_string()).unwrap_or_else(|| {
                let timestamp = Local::now().format("%Y%m%d-%H%M%S");
                format!("filtered-logs-{}.log", timestamp)
            });
            CommandResult {
                effect: Some(CommandEffect::WriteFilteredLogs { filename }),
                status: String::new(),
            }
        }
        "filter" => match arg {
            Some(pattern) if !pattern.is_empty() => CommandResult {
                effect: Some(CommandEffect::AddFilter {
                    kind: FilterKind::Include,
                    pattern: pattern.to_string(),
                }),
                status: format!("Added filter: {}", pattern),
            },
            _ => CommandResult {
                effect: None,
                status: "Usage: filter <pattern>".to_string(),
            },
        },
        "filter-out" => match arg {
            Some(pattern) if !pattern.is_empty() => CommandResult {
                effect: Some(CommandEffect::AddFilter {
                    kind: FilterKind::Exclude,
                    pattern: pattern.to_string(),
                }),
                status: format!("Added filter-out: {}", pattern),
            },
            _ => CommandResult {
                effect: None,
                status: "Usage: filter-out <pattern>".to_string(),
            },
        },
        "filter-clear" => CommandResult {
            effect: Some(CommandEffect::ClearFilters),
            status: "Filters cleared".to_string(),
        },
        "list-filters" => CommandResult {
            effect: Some(CommandEffect::ListFilters),
            status: String::new(),
        },
        "" => CommandResult {
            effect: None,
            status: String::new(),
        },
        _ => CommandResult {
            effect: None,
            status: format!("Unknown command: {}", cmd),
        },
    }
}

fn split_command(input: &str) -> (&str, Option<&str>) {
    let input = input.trim();
    let mut parts = input.splitn(2, ' ');
    let cmd = parts.clone().next().unwrap_or("");
    let arg = parts.nth(1).map(|s| s.trim()).filter(|s| !s.is_empty());
    (cmd, arg)
}

pub fn complete(prefix: &str, index: usize) -> Option<(String, usize)> {
    let lower_prefix = prefix.to_lowercase();
    let matches: Vec<&str> = COMMANDS
        .iter()
        .filter(|&&cmd| cmd.to_lowercase().starts_with(&lower_prefix))
        .copied()
        .collect();

    if matches.is_empty() {
        return None;
    }

    let match_idx = index % matches.len();
    Some((matches[match_idx].to_string(), match_idx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_empty() {
        let (result, _) = complete("", 0).unwrap();
        assert_eq!(result, "filter");
    }

    #[test]
    fn test_complete_partial() {
        let (result, idx) = complete("fi", 0).unwrap();
        assert_eq!(result, "filter");
        assert_eq!(idx, 0);

        let (result, idx) = complete("fi", 1).unwrap();
        assert_eq!(result, "filter-clear");
        assert_eq!(idx, 1);

        let (result, idx) = complete("fi", 2).unwrap();
        assert_eq!(result, "filter-out");
        assert_eq!(idx, 2);
    }

    #[test]
    fn test_complete_wraps() {
        let matches: Vec<_> = (0..4).filter_map(|i| complete("fi", i)).collect();
        assert_eq!(matches.len(), 4);

        let (result, _) = complete("fi", 3).unwrap();
        assert_eq!(result, "filter");

        let (result, _) = complete("fi", 0).unwrap();
        assert_eq!(result, "filter");
    }

    #[test]
    fn test_complete_no_match() {
        assert!(complete("xyz", 0).is_none());
    }

    #[test]
    fn test_complete_shortcut_expands() {
        // "q" should complete to "quit" (shortcut expands to full command)
        let result = complete("q", 0);
        let (text, _) = result.unwrap();
        assert_eq!(text, "quit");
    }

    #[test]
    fn test_parse_quit() {
        let result = parse("quit");
        assert_eq!(result.effect, Some(CommandEffect::Quit));
        assert!(result.status.is_empty());

        let result = parse("q");
        assert_eq!(result.effect, Some(CommandEffect::Quit));
    }

    #[test]
    fn test_parse_write() {
        let result = parse("write test.log");
        assert_eq!(
            result.effect,
            Some(CommandEffect::WriteFilteredLogs {
                filename: "test.log".to_string()
            })
        );

        let result = parse("w");
        assert!(
            matches!(
                result.effect,
                Some(CommandEffect::WriteFilteredLogs { ref filename })
                if filename.starts_with("filtered-logs-") && filename.ends_with(".log")
            ),
            "Expected timestamped filename, got {:?}",
            result.effect
        );
    }

    #[test]
    fn test_parse_filter() {
        let result = parse("filter error");
        assert_eq!(
            result.effect,
            Some(CommandEffect::AddFilter {
                kind: FilterKind::Include,
                pattern: "error".to_string()
            })
        );
        assert_eq!(result.status, "Added filter: error");

        let result = parse("filter");
        assert_eq!(result.effect, None);
        assert_eq!(result.status, "Usage: filter <pattern>");
    }

    #[test]
    fn test_parse_filter_out() {
        let result = parse("filter-out debug");
        assert_eq!(
            result.effect,
            Some(CommandEffect::AddFilter {
                kind: FilterKind::Exclude,
                pattern: "debug".to_string()
            })
        );

        let result = parse("filter-out");
        assert_eq!(result.effect, None);
        assert_eq!(result.status, "Usage: filter-out <pattern>");
    }

    #[test]
    fn test_parse_filter_clear() {
        let result = parse("filter-clear");
        assert_eq!(result.effect, Some(CommandEffect::ClearFilters));
        assert_eq!(result.status, "Filters cleared");
    }

    #[test]
    fn test_parse_list_filters() {
        let result = parse("list-filters");
        assert_eq!(result.effect, Some(CommandEffect::ListFilters));
    }

    #[test]
    fn test_parse_unknown() {
        let result = parse("unknown");
        assert_eq!(result.effect, None);
        assert_eq!(result.status, "Unknown command: unknown");
    }

    #[test]
    fn test_parse_empty() {
        let result = parse("");
        assert_eq!(result.effect, None);
        assert!(result.status.is_empty());
    }

    #[test]
    fn test_split_command() {
        assert_eq!(split_command("filter error"), ("filter", Some("error")));
        assert_eq!(split_command("filter"), ("filter", None));
        assert_eq!(split_command("filter  "), ("filter", None));
        assert_eq!(
            split_command("  filter  error  "),
            ("filter", Some("error"))
        );
        assert_eq!(split_command(""), ("", None));
    }
}
