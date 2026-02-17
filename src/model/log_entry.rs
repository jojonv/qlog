use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    #[serde(rename = "Timestamp")]
    pub timestamp: DateTime<FixedOffset>,
    #[serde(rename = "Level")]
    pub level: LogLevel,
    #[serde(rename = "RenderedMessage", alias = "Message")]
    pub message: String,
    #[serde(rename = "MessageTemplate")]
    pub message_template: String,
    #[serde(rename = "Properties", default)]
    pub properties: serde_json::Value,
    #[serde(rename = "Exception", default)]
    pub exception: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum LogLevel {
    Information,
    Warning,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Information => "Information",
            LogLevel::Warning => "Warning",
            LogLevel::Error => "Error",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Information" | "INFO" => Some(LogLevel::Information),
            "Warning" | "WARN" => Some(LogLevel::Warning),
            "Error" | "ERROR" => Some(LogLevel::Error),
            _ => None,
        }
    }
}

impl LogEntry {
    pub fn from_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
    }

    pub fn source(&self) -> String {
        if let Some(props) = self.properties.as_object() {
            if let Some(source) = props.get("SourceContext") {
                return source.as_str().unwrap_or("").to_string();
            }
        }
        String::new()
    }

    pub fn matches_level(&self, level: LogLevel) -> bool {
        self.level == level
    }

    pub fn matches_text(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        self.message.to_lowercase().contains(&text_lower)
            || self.message_template.to_lowercase().contains(&text_lower)
            || self.source().to_lowercase().contains(&text_lower)
            || self
                .properties
                .to_string()
                .to_lowercase()
                .contains(&text_lower)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(
            LogLevel::from_str("Information"),
            Some(LogLevel::Information)
        );
        assert_eq!(LogLevel::from_str("Warning"), Some(LogLevel::Warning));
        assert_eq!(LogLevel::from_str("Error"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("Unknown"), None);
    }

    #[test]
    fn test_log_entry_from_line() {
        let json = r#"{"Timestamp":"2026-02-13T10:00:00+00:00","Level":"Error","MessageTemplate":"Test","RenderedMessage":"Test","Properties":{"SourceContext":"TestSource"}}"#;
        let entry = LogEntry::from_line(json);
        assert!(entry.is_ok());
        let entry = entry.unwrap();
        assert_eq!(entry.level, LogLevel::Error);
        assert_eq!(entry.source(), "TestSource");
    }
}
