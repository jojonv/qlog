use chrono::{FixedOffset, TimeZone};
use como_log_viewer::model::{Filter, LogEntry, LogLevel};

fn create_test_entry(level: LogLevel, message: &str, source: &str) -> LogEntry {
    LogEntry {
        timestamp: FixedOffset::west_opt(5 * 3600)
            .unwrap()
            .with_ymd_and_hms(2026, 2, 13, 14, 23, 1)
            .unwrap(),
        level,
        message: message.to_string(),
        message_template: message.to_string(),
        properties: serde_json::json!({"SourceContext": source}),
        exception: None,
    }
}

#[test]
fn test_level_filter() {
    let entry = create_test_entry(LogLevel::Error, "test", "App.Service");

    assert!(Filter::Level(LogLevel::Error).matches(&entry));
    assert!(!Filter::Level(LogLevel::Warning).matches(&entry));
}

#[test]
fn test_text_filter() {
    let entry = create_test_entry(LogLevel::Information, "Connection failed", "App.Service");

    assert!(Filter::Text("failed".to_string()).matches(&entry));
    assert!(!Filter::Text("success".to_string()).matches(&entry));
}

#[test]
fn test_date_range_filter() {
    let entry = create_test_entry(LogLevel::Information, "test", "App.Service");

    let start = FixedOffset::west_opt(5 * 3600)
        .unwrap()
        .with_ymd_and_hms(2026, 2, 13, 0, 0, 0)
        .unwrap();
    let end = FixedOffset::west_opt(5 * 3600)
        .unwrap()
        .with_ymd_and_hms(2026, 2, 13, 23, 59, 59)
        .unwrap();

    assert!(Filter::DateRange(start, end).matches(&entry));
}

#[test]
fn test_source_filter() {
    let entry = create_test_entry(LogLevel::Information, "test", "Kistler.AkvisIO.Service");

    assert!(Filter::SourceContext("Kistler".to_string()).matches(&entry));
    assert!(!Filter::SourceContext("Other".to_string()).matches(&entry));
}

#[test]
fn test_combined_filters() {
    let entry = create_test_entry(LogLevel::Error, "Connection failed", "App.Service");

    let filters = vec![
        Filter::Level(LogLevel::Error),
        Filter::Text("failed".to_string()),
    ];

    assert!(filters.iter().all(|f| f.matches(&entry)));
}
