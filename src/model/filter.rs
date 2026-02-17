use super::log_entry::{LogEntry, LogLevel};
use chrono::{DateTime, FixedOffset};

#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    Level(LogLevel),
    Text(String),
    DateRange(DateTime<FixedOffset>, DateTime<FixedOffset>),
    SourceContext(String),
}

impl Filter {
    pub fn matches(&self, entry: &LogEntry) -> bool {
        match self {
            Filter::Level(level) => entry.matches_level(*level),
            Filter::Text(text) => entry.matches_text(text),
            Filter::DateRange(start, end) => entry.timestamp >= *start && entry.timestamp <= *end,
            Filter::SourceContext(source) => entry
                .source()
                .to_lowercase()
                .contains(&source.to_lowercase()),
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            Filter::Level(level) => format!("Level:{}", level.as_str()),
            Filter::Text(text) => format!("Text:{}", text),
            Filter::DateRange(start, end) => {
                format!(
                    "Date:{} to {}",
                    start.format("%Y-%m-%d"),
                    end.format("%Y-%m-%d")
                )
            }
            Filter::SourceContext(source) => format!("Source:{}", source),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FilterSet {
    pub filters: Vec<(Filter, bool)>,
}

impl FilterSet {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    pub fn add(&mut self, filter: Filter) {
        self.filters.push((filter, true));
    }

    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }

    pub fn toggle(&mut self, index: usize) {
        if let Some((_, enabled)) = self.filters.get_mut(index) {
            *enabled = !*enabled;
        }
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.filters.len() {
            self.filters.remove(index);
        }
    }

    pub fn matches(&self, entry: &LogEntry) -> bool {
        self.filters
            .iter()
            .filter(|(_, enabled)| *enabled)
            .all(|(filter, _)| filter.matches(entry))
    }

    pub fn active_count(&self) -> usize {
        self.filters.iter().filter(|(_, enabled)| *enabled).count()
    }
}

impl Default for FilterSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_entry() -> LogEntry {
        LogEntry {
            timestamp: Utc::now().into(),
            level: LogLevel::Error,
            message: "Test error message".to_string(),
            message_template: "Test error message".to_string(),
            exception: None,
            properties: serde_json::json!({"SourceContext": "TestSource"}),
        }
    }

    #[test]
    fn test_filter_level() {
        let entry = create_test_entry();
        let filter = Filter::Level(LogLevel::Error);
        assert!(filter.matches(&entry));

        let filter = Filter::Level(LogLevel::Warning);
        assert!(!filter.matches(&entry));
    }

    #[test]
    fn test_filter_text() {
        let entry = create_test_entry();
        let filter = Filter::Text("error".to_string());
        assert!(filter.matches(&entry));

        let filter = Filter::Text("warning".to_string());
        assert!(!filter.matches(&entry));
    }

    #[test]
    fn test_filter_set() {
        let mut filter_set = FilterSet::new();
        let entry = create_test_entry();

        filter_set.add(Filter::Level(LogLevel::Error));
        assert!(filter_set.matches(&entry));

        filter_set.add(Filter::Text("warning".to_string()));
        assert!(!filter_set.matches(&entry));
    }
}
