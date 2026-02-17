use como_log_viewer::model::{Filter, FilterGroup, FilterSet, LogEntry};

fn create_test_entry(text: &str) -> LogEntry {
    LogEntry::new(text.to_string(), None)
}

#[test]
fn test_filter_matches_text() {
    let entry = create_test_entry("Connection failed error");

    let filter = Filter::new("error".to_string());
    assert!(filter.matches(&entry.raw));

    let filter2 = Filter::new("success".to_string());
    assert!(!filter2.matches(&entry.raw));
}

#[test]
fn test_filter_can_be_disabled() {
    let entry = create_test_entry("error message");

    let mut filter = Filter::new("error".to_string());
    assert!(filter.matches(&entry.raw));

    filter.enabled = false;
    assert!(!filter.matches(&entry.raw));
}

#[test]
fn test_filter_group_or_logic() {
    let entry = create_test_entry("warning message");

    let mut group = FilterGroup::new();
    group.filters.push(Filter::new("error".to_string()));
    group.filters.push(Filter::new("warning".to_string()));

    assert!(group.matches(&entry.raw));
}

#[test]
fn test_filter_group_all_disabled() {
    let entry = create_test_entry("error message");

    let mut group = FilterGroup::new();
    let mut f1 = Filter::new("error".to_string());
    f1.enabled = false;
    group.filters.push(f1);

    assert!(!group.matches(&entry.raw));
}

#[test]
fn test_filter_set_and_logic() {
    let entry = create_test_entry("error warning message");

    let mut set = FilterSet::new();

    let mut group1 = FilterGroup::new();
    group1.filters.push(Filter::new("error".to_string()));

    let mut group2 = FilterGroup::new();
    group2.filters.push(Filter::new("warning".to_string()));

    set.groups.push(group1);
    set.groups.push(group2);

    assert!(set.matches(&entry));
}

#[test]
fn test_filter_set_all_groups_required() {
    let entry = create_test_entry("error message");

    let mut set = FilterSet::new();

    let mut group1 = FilterGroup::new();
    group1.filters.push(Filter::new("error".to_string()));

    let mut group2 = FilterGroup::new();
    group2.filters.push(Filter::new("warning".to_string()));

    set.groups.push(group1);
    set.groups.push(group2);

    assert!(!set.matches(&entry));
}

#[test]
fn test_empty_filter_set_matches_all() {
    let entry = create_test_entry("anything");
    let set = FilterSet::new();

    assert!(set.matches(&entry));
}
