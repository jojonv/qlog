use qlog::model::timestamp::detect_timestamp;
use qlog::model::LogEntry;

#[test]
fn test_parse_log_entry_with_timestamp() {
    let line = r#"2026-02-13T14:23:01.1234567-05:00 [Information] Test message"#;

    let entry = LogEntry::new(line.to_string(), None);
    assert!(entry.timestamp.is_some());
    assert!(entry.raw.contains("Test message"));
}

#[test]
fn test_parse_raw_line() {
    let line = "Some raw log line without special format";
    let entry = LogEntry::new(line.to_string(), None);

    assert_eq!(entry.raw, line);
    assert!(entry.timestamp.is_none());
}

#[test]
fn test_timestamp_detection_iso8601() {
    let line = "2026-02-13T14:23:01+00:00 Some message";
    let ts = detect_timestamp(line);
    assert!(ts.is_some());
}

#[test]
fn test_timestamp_detection_common_log() {
    let line = "[13/Feb/2026:14:23:01 +0000] Some message";
    let ts = detect_timestamp(line);
    assert!(ts.is_some());
}
