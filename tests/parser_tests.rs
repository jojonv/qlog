use chrono::FixedOffset;
use como_log_viewer::model::{LogEntry, LogLevel};

#[test]
fn test_parse_log_entry_with_properties() {
    let json = r#"{"Timestamp":"2026-02-13T14:23:01.1234567-05:00","Level":"Information","MessageTemplate":"Test message","RenderedMessage":"Test message","Properties":{"SourceContext":"MyApp.Service"}}"#;

    let entry = LogEntry::from_line(json).unwrap();

    assert_eq!(entry.level, LogLevel::Information);
    assert_eq!(entry.message, "Test message");
    assert_eq!(entry.source(), "MyApp.Service");
}

#[test]
fn test_parse_error_log() {
    let json = r#"{"Timestamp":"2026-02-13T14:23:01.1234567-05:00","Level":"Error","MessageTemplate":"Error occurred","RenderedMessage":"Error occurred","Properties":{"SourceContext":"MyApp.Service"},"Exception":"System.Exception: test"}"#;

    let entry = LogEntry::from_line(json).unwrap();
    assert_eq!(entry.level, LogLevel::Error);
    assert!(entry.exception.is_some());
}

#[test]
fn test_parse_warning_log() {
    let json = r#"{"Timestamp":"2026-02-13T14:23:01.1234567-05:00","Level":"Warning","MessageTemplate":"Warning message","RenderedMessage":"Warning message","Properties":{}}"#;

    let entry = LogEntry::from_line(json).unwrap();
    assert_eq!(entry.level, LogLevel::Warning);
    assert_eq!(entry.source(), "");
}

#[test]
fn test_parse_invalid_json() {
    let json = r#"not valid json"#;
    assert!(LogEntry::from_line(json).is_err());
}

#[test]
fn test_level_from_str() {
    assert_eq!(
        LogLevel::from_str("Information"),
        Some(LogLevel::Information)
    );
    assert_eq!(LogLevel::from_str("Warning"), Some(LogLevel::Warning));
    assert_eq!(LogLevel::from_str("Error"), Some(LogLevel::Error));
    assert_eq!(LogLevel::from_str("UNKNOWN"), None);
}
