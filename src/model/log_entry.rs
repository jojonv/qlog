use super::timestamp::detect_timestamp;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub struct LogEntry {
    pub raw: String,
    pub timestamp: Option<DateTime<Utc>>,
}

impl LogEntry {
    pub fn new(raw: String, timestamp: Option<DateTime<Utc>>) -> Self {
        let timestamp = timestamp.or_else(|| detect_timestamp(&raw));
        Self { raw, timestamp }
    }
}
