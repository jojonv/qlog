/// Boyer-Moore-Horspool string matcher for fast substring search.
/// Uses O(m) preprocessing and O(n/m) average-case search time.
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct BMHMatcher {
    /// The pattern to search for (lowercase bytes)
    pattern: Vec<u8>,
    /// Skip table: for each byte value, stores how far to shift
    skip_table: [usize; 256],
    /// Pattern length (cached for performance)
    pattern_len: usize,
}

impl BMHMatcher {
    /// Create a new BMH matcher for the given pattern.
    /// Pattern should already be in lowercase for case-insensitive matching.
    pub fn new(pattern: Vec<u8>) -> Self {
        let pattern_len = pattern.len();
        let mut skip_table = [pattern_len; 256];

        // Build skip table: for each byte in pattern (except last),
        // set the skip distance to the distance from the end
        if pattern_len > 0 {
            for i in 0..pattern_len - 1 {
                skip_table[pattern[i] as usize] = pattern_len - 1 - i;
            }
        }

        Self {
            pattern,
            skip_table,
            pattern_len,
        }
    }

    /// Find the pattern in the given text using BMH algorithm.
    /// Returns the starting position if found, None otherwise.
    /// Text should be converted to lowercase for case-insensitive matching.
    pub fn find(&self, text: &[u8]) -> Option<usize> {
        if self.pattern_len == 0 {
            return Some(0);
        }

        if self.pattern_len > text.len() {
            return None;
        }

        // Special case for single character patterns
        if self.pattern_len == 1 {
            let byte = self.pattern[0];
            for (i, &text_byte) in text.iter().enumerate() {
                if text_byte == byte {
                    return Some(i);
                }
            }
            return None;
        }

        let mut pos = self.pattern_len - 1;
        let last = self.pattern_len - 1;

        while pos < text.len() {
            // Check if pattern matches at current position
            let mut matched = true;
            let mut i = self.pattern_len;
            while i > 0 {
                i -= 1;
                if text[pos - (self.pattern_len - 1 - i)] != self.pattern[i] {
                    matched = false;
                    break;
                }
            }

            if matched {
                // Found match
                return Some(pos - last);
            }

            // Shift by skip table value for the character at current position
            let shift = self.skip_table[text[pos] as usize];
            pos += shift;
        }

        None
    }

    /// Check if pattern exists in text.
    pub fn contains(&self, text: &[u8]) -> bool {
        self.find(text).is_some()
    }

    /// Find all match positions in text.
    /// Returns vector of (start, end) byte positions.
    pub fn find_all(&self, text: &[u8]) -> Vec<(usize, usize)> {
        let mut matches = Vec::new();

        if self.pattern_len == 0 {
            return matches;
        }

        if self.pattern_len > text.len() {
            return matches;
        }

        // Special case for single character patterns
        if self.pattern_len == 1 {
            let byte = self.pattern[0];
            for (i, &text_byte) in text.iter().enumerate() {
                if text_byte == byte {
                    matches.push((i, i + 1));
                }
            }
            return matches;
        }

        let last = self.pattern_len - 1;
        let mut pos = last;

        while pos < text.len() {
            // Check if pattern matches at current position
            let mut matched = true;
            let mut i = self.pattern_len;
            while i > 0 {
                i -= 1;
                if text[pos - (self.pattern_len - 1 - i)] != self.pattern[i] {
                    matched = false;
                    break;
                }
            }

            if matched {
                // Found match
                let start = pos - last;
                matches.push((start, start + self.pattern_len));
                // Move past this match to find overlapping matches
                pos += 1;
            } else {
                // Shift by skip table value for the character at current position
                let shift = self.skip_table[text[pos] as usize];
                pos += shift;
            }
        }

        matches
    }
}

/// A single filter with cached lowercase bytes and BMH matcher for zero-allocation matching.
#[derive(Debug, Clone)]
pub struct Filter {
    /// Original filter text (for display/editing)
    text: String,
    /// Cached lowercase bytes for fast matching
    cached_lower: Vec<u8>,
    /// BMH matcher for optimized pattern matching
    matcher: BMHMatcher,
    /// Whether this filter is enabled
    enabled: bool,
}

impl Filter {
    /// Create a new filter.
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let cached_lower = Self::to_lower_bytes(&text);
        let matcher = BMHMatcher::new(cached_lower.clone());
        Self {
            text,
            cached_lower,
            matcher,
            enabled: true,
        }
    }

    /// Create a new filter with explicit enabled state.
    pub fn with_enabled(text: impl Into<String>, enabled: bool) -> Self {
        let text = text.into();
        let cached_lower = Self::to_lower_bytes(&text);
        let matcher = BMHMatcher::new(cached_lower.clone());
        Self {
            text,
            cached_lower,
            matcher,
            enabled,
        }
    }

    /// Convert string to lowercase ASCII bytes.
    fn to_lower_bytes(text: &str) -> Vec<u8> {
        text.bytes()
            .map(|b| {
                if b.is_ascii_uppercase() {
                    b.to_ascii_lowercase()
                } else {
                    b
                }
            })
            .collect()
    }

    /// ASCII lowercase a byte.
    #[inline]
    fn ascii_lower(b: u8) -> u8 {
        if b.is_ascii_uppercase() {
            b.to_ascii_lowercase()
        } else {
            b
        }
    }

    /// Get the filter text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set the filter text.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.cached_lower = Self::to_lower_bytes(&self.text);
        self.matcher = BMHMatcher::new(self.cached_lower.clone());
    }

    /// Check if the filter is enabled.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Set whether the filter is enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Toggle the enabled state.
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Check if the filter matches the given line bytes.
    /// Uses Boyer-Moore-Horspool algorithm with ASCII lowercase byte comparison.
    /// Pre-lowercases the text into a thread-local buffer for O(n/m) performance.
    pub fn matches(&self, line_bytes: &[u8]) -> bool {
        if !self.enabled || self.cached_lower.is_empty() {
            return true;
        }

        // Case-insensitive substring search using ASCII lowercase
        if self.cached_lower.len() > line_bytes.len() {
            return false;
        }

        let pattern_len = self.cached_lower.len();

        if pattern_len == 0 {
            return true;
        }

        // Use thread-local buffer to avoid allocation
        // Pre-lowercase the entire line once, then run pure BMH
        thread_local! {
            static LOWER_BUF: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(8192));
        }

        LOWER_BUF.with(|buf| {
            let mut buf = buf.borrow_mut();
            buf.clear();
            buf.extend(line_bytes.iter().map(|&b| Self::ascii_lower(b)));
            self.matcher.contains(&buf)
        })
    }
}

/// A group of filters (AND logic between filters).
#[derive(Debug, Clone)]
pub struct FilterGroup {
    filters: Vec<Filter>,
}

impl FilterGroup {
    /// Create a new empty filter group.
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    /// Create a new filter group from a list of filters.
    pub fn from_filters(filters: Vec<Filter>) -> Self {
        Self { filters }
    }

    /// Add a filter to the group.
    pub fn add_filter(&mut self, filter: Filter) {
        self.filters.push(filter);
    }

    /// Remove a filter by index.
    pub fn remove_filter(&mut self, index: usize) -> Option<Filter> {
        if index < self.filters.len() {
            Some(self.filters.remove(index))
        } else {
            None
        }
    }

    /// Get a filter by index.
    pub fn get_filter(&self, index: usize) -> Option<&Filter> {
        self.filters.get(index)
    }

    /// Get a mutable filter by index.
    pub fn get_filter_mut(&mut self, index: usize) -> Option<&mut Filter> {
        self.filters.get_mut(index)
    }

    /// Get all filters.
    pub fn filters(&self) -> &[Filter] {
        &self.filters
    }

    /// Get mutable filters.
    pub fn filters_mut(&mut self) -> &mut Vec<Filter> {
        &mut self.filters
    }

    /// Check if the group matches the given line bytes.
    /// At least one filter must match (OR logic).
    pub fn matches(&self, line_bytes: &[u8]) -> bool {
        if self.filters.is_empty() {
            return true;
        }

        for filter in &self.filters {
            if filter.matches(line_bytes) {
                return true;
            }
        }

        false
    }

    /// Clear all filters.
    pub fn clear(&mut self) {
        self.filters.clear();
    }

    /// Get the number of filters.
    pub fn len(&self) -> usize {
        self.filters.len()
    }

    /// Check if the group is empty.
    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }
}

impl Default for FilterGroup {
    fn default() -> Self {
        Self::new()
    }
}

/// A set of filter groups (OR logic between groups).
#[derive(Debug, Clone)]
pub struct FilterSet {
    groups: Vec<FilterGroup>,
}

impl FilterSet {
    /// Create a new empty filter set.
    pub fn new() -> Self {
        Self { groups: Vec::new() }
    }

    /// Create a new filter set with a default group.
    pub fn with_default_group() -> Self {
        let mut set = Self::new();
        set.add_group(FilterGroup::new());
        set
    }

    /// Add a filter group.
    pub fn add_group(&mut self, group: FilterGroup) {
        self.groups.push(group);
    }

    /// Remove a group by index.
    pub fn remove_group(&mut self, index: usize) -> Option<FilterGroup> {
        if index < self.groups.len() {
            Some(self.groups.remove(index))
        } else {
            None
        }
    }

    /// Get a group by index.
    pub fn get_group(&self, index: usize) -> Option<&FilterGroup> {
        self.groups.get(index)
    }

    /// Get a mutable group by index.
    pub fn get_group_mut(&mut self, index: usize) -> Option<&mut FilterGroup> {
        self.groups.get_mut(index)
    }

    /// Get all groups.
    pub fn groups(&self) -> &[FilterGroup] {
        &self.groups
    }

    /// Get mutable groups.
    pub fn groups_mut(&mut self) -> &mut Vec<FilterGroup> {
        &mut self.groups
    }

    /// Check if the set matches the given line bytes.
    /// All groups must match (AND logic).
    pub fn matches(&self, line_bytes: &[u8]) -> bool {
        if self.groups.is_empty() {
            return true;
        }

        for group in &self.groups {
            if !group.matches(line_bytes) {
                return false;
            }
        }

        true
    }

    /// Check if the set matches using an iterator over lines.
    pub fn matches_iter<'a, I>(&self, lines: I) -> impl Iterator<Item = bool> + use<'a, '_, I>
    where
        I: Iterator<Item = &'a [u8]>,
    {
        lines.map(|line| self.matches(line))
    }

    /// Clear all groups.
    pub fn clear(&mut self) {
        self.groups.clear();
    }

    /// Get the number of groups.
    pub fn len(&self) -> usize {
        self.groups.len()
    }

    /// Check if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }
}

impl Default for FilterSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Filter kind - include or exclude
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterKind {
    Include,
    Exclude,
}

/// New command-based filter system
/// Replaces FilterSet/FilterGroup/Filter with flat list
#[derive(Debug, Clone)]
pub struct FilterRule {
    pub pattern: String,
    pub kind: FilterKind,
    matcher: BMHMatcher,
}

impl FilterRule {
    pub fn new(pattern: impl Into<String>, kind: FilterKind) -> Self {
        let pattern = pattern.into();
        let pattern_lower = pattern.to_lowercase();
        let matcher = BMHMatcher::new(pattern_lower.into_bytes());
        Self {
            pattern,
            kind,
            matcher,
        }
    }

    /// ASCII lowercase a byte.
    #[inline]
    fn ascii_lower(b: u8) -> u8 {
        if b.is_ascii_uppercase() {
            b.to_ascii_lowercase()
        } else {
            b
        }
    }

    pub fn matches(&self, text: &[u8]) -> bool {
        // Use thread-local buffer to avoid allocation
        // Pre-lowercase the entire text once, then run pure BMH
        thread_local! {
            static LOWER_BUF: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(8192));
        }

        LOWER_BUF.with(|buf| {
            let mut buf = buf.borrow_mut();
            buf.clear();
            buf.extend(text.iter().map(|&b| Self::ascii_lower(b)));
            self.matcher.contains(&buf)
        })
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    pub fn kind(&self) -> FilterKind {
        self.kind
    }
}

#[derive(Debug, Clone)]
pub struct FilterList {
    includes: Vec<FilterRule>,
    excludes: Vec<FilterRule>,
}

impl FilterList {
    pub fn new() -> Self {
        Self {
            includes: Vec::new(),
            excludes: Vec::new(),
        }
    }

    pub fn add_include(&mut self, pattern: impl Into<String>) {
        self.includes
            .push(FilterRule::new(pattern, FilterKind::Include));
    }

    pub fn add_exclude(&mut self, pattern: impl Into<String>) {
        self.excludes
            .push(FilterRule::new(pattern, FilterKind::Exclude));
    }

    pub fn clear(&mut self) {
        self.includes.clear();
        self.excludes.clear();
    }

    pub fn remove_include(&mut self, index: usize) -> Option<FilterRule> {
        if index < self.includes.len() {
            Some(self.includes.remove(index))
        } else {
            None
        }
    }

    pub fn remove_exclude(&mut self, index: usize) -> Option<FilterRule> {
        if index < self.excludes.len() {
            Some(self.excludes.remove(index))
        } else {
            None
        }
    }

    pub fn includes(&self) -> &[FilterRule] {
        &self.includes
    }

    pub fn excludes(&self) -> &[FilterRule] {
        &self.excludes
    }

    pub fn is_empty(&self) -> bool {
        self.includes.is_empty() && self.excludes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.includes.len() + self.excludes.len()
    }

    /// Iterate over all filters (includes first, then excludes)
    /// Returns (index, &FilterRule) where index is the position in the combined list
    pub fn iter(&self) -> impl Iterator<Item = (usize, &FilterRule)> {
        self.includes.iter().enumerate().chain(
            self.excludes
                .iter()
                .enumerate()
                .map(|(i, f)| (i + self.includes.len(), f)),
        )
    }

    /// Returns true if the text matches all include filters and none of the exclude filters
    pub fn matches(&self, text: &[u8]) -> bool {
        // Must match ALL includes
        for include in &self.includes {
            if !include.matches(text) {
                return false;
            }
        }

        // Must NOT match ANY excludes
        for exclude in &self.excludes {
            if exclude.matches(text) {
                return false;
            }
        }

        true
    }
}

impl Default for FilterList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bmh_matcher_empty_pattern() {
        let matcher = BMHMatcher::new(vec![]);
        assert_eq!(matcher.find(b"hello"), Some(0));
        assert_eq!(matcher.find(b""), Some(0));
    }

    #[test]
    fn test_bmh_matcher_single_char() {
        let matcher = BMHMatcher::new(vec![b'a']);
        assert_eq!(matcher.find(b"hello"), None);
        assert_eq!(matcher.find(b"apple"), Some(0));
        assert_eq!(matcher.find(b"banana"), Some(1));
        assert_eq!(matcher.find(b"aa"), Some(0));
    }

    #[test]
    fn test_bmh_matcher_not_found() {
        let matcher = BMHMatcher::new(vec![b'x', b'y', b'z']);
        assert_eq!(matcher.find(b"hello"), None);
        assert_eq!(matcher.find(b"abc"), None);
        assert_eq!(matcher.find(b""), None);
    }

    #[test]
    fn test_bmh_matcher_multiple_matches() {
        let matcher = BMHMatcher::new(vec![b'a', b'b']);
        // Returns first match position
        assert_eq!(matcher.find(b"ababab"), Some(0));
        assert_eq!(matcher.find(b"cabcab"), Some(1));
    }

    #[test]
    fn test_bmh_matcher_long_pattern() {
        let pattern: Vec<u8> = (0..100).map(|i| b'a' + (i % 26) as u8).collect();
        let matcher = BMHMatcher::new(pattern.clone());
        assert_eq!(matcher.find(&pattern), Some(0));

        let mut text = vec![b'x'; 50];
        text.extend_from_slice(&pattern);
        text.extend_from_slice(&[b'x'; 50]);
        assert_eq!(matcher.find(&text), Some(50));
    }

    #[test]
    fn test_bmh_matcher_contains() {
        let matcher = BMHMatcher::new(vec![b't', b'e', b's', b't']);
        assert!(matcher.contains(b"this is a test"));
        assert!(matcher.contains(b"testing"));
        assert!(!matcher.contains(b"hello world"));
    }

    #[test]
    fn test_bmh_matcher_no_false_positive() {
        // Bug fix: "jeb" was incorrectly matching "0EB" because the
        // loop decremented i before checking, causing i==0 to be true
        // even when the first character didn't match.
        let matcher = BMHMatcher::new(vec![b'j', b'e', b'b']);
        assert!(!matcher.contains(b"0EB")); // j vs 0 should not match
        assert!(!matcher.contains(b"abc"));
        assert!(matcher.contains(b"jeb"));
        // Note: BMHMatcher is case-sensitive; caller handles case insensitivity
    }

    #[test]
    fn test_filter_matches() {
        let filter = Filter::new("ERROR");

        assert!(filter.matches(b"This is an ERROR message"));
        assert!(filter.matches(b"this is an error message"));
        assert!(!filter.matches(b"This is a warning message"));
        assert!(filter.matches(b"ERROR")); // Should match standalone too
        assert!(filter.matches(b"error"));
    }

    #[test]
    fn test_filter_empty() {
        let filter = Filter::new("");

        assert!(filter.matches(b"anything"));
        assert!(filter.matches(b""));
    }

    #[test]
    fn test_filter_disabled() {
        let mut filter = Filter::new("ERROR");
        filter.set_enabled(false);

        assert!(filter.matches(b"anything"));
        assert!(filter.matches(b"nothing"));
    }

    #[test]
    fn test_filter_case_insensitive() {
        let filter = Filter::new("Error");

        assert!(filter.matches(b"ERROR"));
        assert!(filter.matches(b"error"));
        assert!(filter.matches(b"Error"));
        assert!(filter.matches(b"eRRoR"));
    }

    #[test]
    fn test_filter_utf8() {
        let filter = Filter::new("test");

        // ASCII lowercase comparison only works with ASCII characters
        // "ë" won't match "e" with ASCII lowercase
        assert!(!filter.matches("tëst".as_bytes()));
        assert!(filter.matches("TEST".as_bytes()));
    }

    #[test]
    fn test_filter_group_or() {
        let mut group = FilterGroup::new();
        group.add_filter(Filter::new("ERROR"));
        group.add_filter(Filter::new("timeout"));

        // Group uses OR logic - at least one filter must match
        assert!(group.matches(b"ERROR: connection timeout")); // both match
        assert!(group.matches(b"ERROR: failed")); // only "ERROR" matches
        assert!(group.matches(b"timeout occurred")); // only "timeout" matches
        assert!(!group.matches(b"info message")); // neither matches
    }

    #[test]
    fn test_filter_set_and() {
        let mut set = FilterSet::new();

        let mut group1 = FilterGroup::new();
        group1.add_filter(Filter::new("ERROR"));
        set.add_group(group1);

        let mut group2 = FilterGroup::new();
        group2.add_filter(Filter::new("WARN"));
        set.add_group(group2);

        // AND logic: both groups must match
        assert!(set.matches(b"ERROR and WARN together"));
        assert!(!set.matches(b"This is an ERROR")); // Only group1 matches
        assert!(!set.matches(b"This is a WARN")); // Only group2 matches
        assert!(!set.matches(b"This is INFO")); // Neither matches
    }

    #[test]
    fn test_filter_set_empty() {
        let set = FilterSet::new();

        assert!(set.matches(b"anything"));
    }

    #[test]
    fn test_filter_modify_text() {
        let mut filter = Filter::new("old");
        filter.set_text("new");

        assert!(!filter.matches(b"old value"));
        assert!(filter.matches(b"new value"));
    }

    #[test]
    fn test_bmh_find_all_basic() {
        let matcher = BMHMatcher::new(vec![b'a', b'b']);
        let matches = matcher.find_all(b"abc ab ab");
        assert_eq!(matches, vec![(0, 2), (4, 6), (7, 9)]);
    }

    #[test]
    fn test_bmh_find_all_overlapping() {
        let matcher = BMHMatcher::new(vec![b'a', b'a']);
        let matches = matcher.find_all(b"aaaa");
        assert_eq!(matches, vec![(0, 2), (1, 3), (2, 4)]);
    }

    #[test]
    fn test_bmh_find_all_case_insensitive() {
        // Pattern in lowercase
        let matcher = BMHMatcher::new(vec![b't', b'e', b's', b't']);
        // Text also in lowercase (caller responsibility)
        let matches = matcher.find_all(b"this is a test string with test");
        assert_eq!(matches, vec![(10, 14), (27, 31)]);
    }

    #[test]
    fn test_bmh_find_all_empty_pattern() {
        let matcher = BMHMatcher::new(vec![]);
        let matches = matcher.find_all(b"hello");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_bmh_find_all_pattern_longer_than_text() {
        let matcher = BMHMatcher::new(vec![
            b'h', b'e', b'l', b'l', b'o', b' ', b'w', b'o', b'r', b'l', b'd',
        ]);
        let matches = matcher.find_all(b"hello");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_bmh_find_all_single_char() {
        let matcher = BMHMatcher::new(vec![b'a']);
        let matches = matcher.find_all(b"banana");
        assert_eq!(matches, vec![(1, 2), (3, 4), (5, 6)]);
    }

    #[test]
    fn test_bmh_find_all_no_matches() {
        let matcher = BMHMatcher::new(vec![b'x', b'y', b'z']);
        let matches = matcher.find_all(b"hello world");
        assert!(matches.is_empty());
    }

    // FilterList tests
    #[test]
    fn test_filter_list_empty() {
        let list = FilterList::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert!(list.matches(b"anything"));
    }

    #[test]
    fn test_filter_list_single_include() {
        let mut list = FilterList::new();
        list.add_include("error");

        assert!(!list.is_empty());
        assert_eq!(list.len(), 1);
        assert!(list.matches(b"this is an ERROR message"));
        assert!(!list.matches(b"this is a warning"));
    }

    #[test]
    fn test_filter_list_multiple_includes_and_logic() {
        let mut list = FilterList::new();
        list.add_include("error");
        list.add_include("timeout");

        // AND logic: must match ALL includes
        assert!(list.matches(b"error timeout both"));
        assert!(!list.matches(b"only error here"));
        assert!(!list.matches(b"only timeout here"));
        assert!(!list.matches(b"neither here"));
    }

    #[test]
    fn test_filter_list_single_exclude() {
        let mut list = FilterList::new();
        list.add_exclude("debug");

        assert!(list.matches(b"error message"));
        assert!(!list.matches(b"debug message"));
        assert!(list.matches(b"this is INFO"));
    }

    #[test]
    fn test_filter_list_multiple_excludes() {
        let mut list = FilterList::new();
        list.add_exclude("debug");
        list.add_exclude("trace");

        // AND NOT logic: must NOT match ANY exclude
        assert!(list.matches(b"error message"));
        assert!(!list.matches(b"debug output"));
        assert!(!list.matches(b"trace info"));
        assert!(!list.matches(b"debug and trace"));
    }

    #[test]
    fn test_filter_list_include_and_exclude() {
        let mut list = FilterList::new();
        list.add_include("error");
        list.add_exclude("debug");

        // Must match include AND not match exclude
        assert!(list.matches(b"error occurred"));
        assert!(!list.matches(b"error with debug"));
        assert!(!list.matches(b"just debug"));
        assert!(!list.matches(b"neither"));
    }

    #[test]
    fn test_filter_list_clear() {
        let mut list = FilterList::new();
        list.add_include("error");
        list.add_exclude("debug");

        assert_eq!(list.len(), 2);

        list.clear();

        assert!(list.is_empty());
        assert!(list.matches(b"anything"));
    }

    #[test]
    fn test_filter_list_remove_include() {
        let mut list = FilterList::new();
        list.add_include("error");
        list.add_include("warning");

        assert_eq!(list.len(), 2);

        let removed = list.remove_include(0);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().pattern(), "error");
        assert_eq!(list.len(), 1);

        // Remaining filter is "warning"
        assert!(list.matches(b"warning message"));
        assert!(!list.matches(b"error message"));
    }

    #[test]
    fn test_filter_list_remove_exclude() {
        let mut list = FilterList::new();
        list.add_exclude("debug");
        list.add_exclude("trace");

        let removed = list.remove_exclude(0);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().pattern(), "debug");
        assert_eq!(list.len(), 1);

        // Remaining exclude is "trace"
        assert!(!list.matches(b"trace"));
        assert!(list.matches(b"debug")); // debug no longer excluded
    }

    #[test]
    fn test_filter_list_remove_out_of_bounds() {
        let mut list = FilterList::new();
        list.add_include("error");

        assert!(list.remove_include(1).is_none());
        assert!(list.remove_include(100).is_none());
    }

    #[test]
    fn test_filter_list_case_insensitive() {
        let mut list = FilterList::new();
        list.add_include("ERROR");

        assert!(list.matches(b"error"));
        assert!(list.matches(b"ERROR"));
        assert!(list.matches(b"Error"));
        assert!(list.matches(b"eRrOr"));
    }

    #[test]
    fn test_filter_list_pattern_access() {
        let mut list = FilterList::new();
        list.add_include("test");

        assert_eq!(list.includes()[0].pattern(), "test");
        assert!(list.excludes().is_empty());
    }

    #[test]
    fn test_filter_list_default() {
        let list: FilterList = Default::default();
        assert!(list.is_empty());
        assert!(list.matches(b"anything"));
    }

    #[test]
    fn test_filter_list_complex_scenario() {
        let mut list = FilterList::new();

        // Simulate: Show errors and warnings but not from debug builds
        list.add_include("error");
        list.add_include("warning");
        list.add_exclude("debug");

        // Line must contain BOTH error AND warning (AND logic), AND NOT debug
        assert!(list.matches(b"error and warning together"));
        assert!(list.matches(b"WARNING: then ERROR occurred"));
        assert!(!list.matches(b"error in debug mode")); // has debug
        assert!(!list.matches(b"debug warning")); // has debug
        assert!(!list.matches(b"error only")); // missing warning
        assert!(!list.matches(b"warning only")); // missing error
        assert!(!list.matches(b"info message")); // has neither
    }

    #[test]
    fn test_filter_rule_basic() {
        let rule = FilterRule::new("test", FilterKind::Include);

        assert!(rule.matches(b"this is a test"));
        assert!(rule.matches(b"TESTING"));
        assert!(!rule.matches(b"hello world"));
    }

    #[test]
    fn test_filter_rule_empty_pattern() {
        let rule = FilterRule::new("", FilterKind::Include);
        assert!(rule.matches(b"anything"));
        assert!(rule.matches(b""));
    }
}
