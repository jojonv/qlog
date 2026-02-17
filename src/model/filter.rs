use super::log_entry::LogEntry;

#[derive(Debug, Clone, PartialEq)]
pub struct Filter {
    pub text: String,
    pub enabled: bool,
}

impl Filter {
    pub fn new(text: String) -> Self {
        Self {
            text,
            enabled: true,
        }
    }

    pub fn matches(&self, line: &str) -> bool {
        if !self.enabled {
            return false;
        }
        line.to_lowercase().contains(&self.text.to_lowercase())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilterGroup {
    pub filters: Vec<Filter>,
}

impl FilterGroup {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    pub fn matches(&self, line: &str) -> bool {
        if self.filters.is_empty() {
            return true;
        }
        self.filters.iter().any(|f| f.matches(line))
    }

    pub fn add_filter(&mut self, text: String) {
        self.filters.push(Filter::new(text));
    }

    pub fn is_empty(&self) -> bool {
        self.filters.is_empty() || self.filters.iter().all(|f| !f.enabled)
    }
}

impl Default for FilterGroup {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct FilterSet {
    pub groups: Vec<FilterGroup>,
}

impl FilterSet {
    pub fn new() -> Self {
        Self { groups: Vec::new() }
    }

    pub fn matches(&self, entry: &LogEntry) -> bool {
        self.groups
            .iter()
            .filter(|g| !g.is_empty())
            .all(|g| g.matches(&entry.raw))
    }

    pub fn add_group(&mut self) -> usize {
        self.groups.push(FilterGroup::new());
        self.groups.len() - 1
    }

    pub fn add_filter_to_group(&mut self, group_idx: usize, text: String) {
        if let Some(group) = self.groups.get_mut(group_idx) {
            group.add_filter(text);
        }
    }

    pub fn remove_filter(&mut self, group_idx: usize, filter_idx: usize) {
        if let Some(group) = self.groups.get_mut(group_idx) {
            if filter_idx < group.filters.len() {
                group.filters.remove(filter_idx);
            }
        }
        self.groups.retain(|g| !g.filters.is_empty());
    }

    pub fn toggle_filter(&mut self, group_idx: usize, filter_idx: usize) {
        if let Some(group) = self.groups.get_mut(group_idx) {
            if let Some(filter) = group.filters.get_mut(filter_idx) {
                filter.enabled = !filter.enabled;
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty() || self.groups.iter().all(|g| g.is_empty())
    }

    pub fn total_filters(&self) -> usize {
        self.groups.iter().map(|g| g.filters.len()).sum()
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

    #[test]
    fn test_filter_matches_case_insensitive() {
        let filter = Filter::new("ERROR".to_string());
        assert!(filter.matches("This is an error message"));
        assert!(filter.matches("ERROR: something failed"));
        assert!(!filter.matches("This is fine"));
    }

    #[test]
    fn test_filter_disabled() {
        let mut filter = Filter::new("error".to_string());
        filter.enabled = false;
        assert!(!filter.matches("This is an error message"));
    }

    #[test]
    fn test_filter_group_or_logic() {
        let mut group = FilterGroup::new();
        group.add_filter("error".to_string());
        group.add_filter("warning".to_string());

        assert!(group.matches("This is an error"));
        assert!(group.matches("This is a warning"));
        assert!(!group.matches("This is fine"));
    }

    #[test]
    fn test_filter_set_and_logic() {
        let mut set = FilterSet::new();

        let mut g1 = FilterGroup::new();
        g1.add_filter("error".to_string());
        set.groups.push(g1);

        let mut g2 = FilterGroup::new();
        g2.add_filter("database".to_string());
        set.groups.push(g2);

        assert!(set.matches(&LogEntry {
            raw: "error in database connection".to_string(),
            timestamp: None
        }));
        assert!(!set.matches(&LogEntry {
            raw: "error in cache".to_string(),
            timestamp: None
        }));
        assert!(!set.matches(&LogEntry {
            raw: "database is fine".to_string(),
            timestamp: None
        }));
    }
}
