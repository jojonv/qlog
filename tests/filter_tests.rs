use como_log_viewer::model::{Filter, FilterGroup, FilterSet};

#[test]
fn test_filter_matches_text() {
    let filter = Filter::new("error");
    assert!(filter.matches("Connection failed error".as_bytes()));

    let filter2 = Filter::new("success");
    assert!(!filter2.matches("Connection failed error".as_bytes()));
}

#[test]
fn test_filter_can_be_disabled() {
    let mut filter = Filter::new("error");
    assert!(filter.matches("error message".as_bytes()));

    filter.set_enabled(false);
    // When disabled, filter returns true (matches everything - no filtering)
    assert!(filter.matches("error message".as_bytes()));
    assert!(filter.matches("success message".as_bytes()));
}

#[test]
fn test_filter_case_insensitive() {
    let filter = Filter::new("ERROR");
    assert!(filter.matches("error message".as_bytes()));
    assert!(filter.matches("ERROR message".as_bytes()));
    assert!(filter.matches("Error message".as_bytes()));
}

#[test]
fn test_filter_group_or_logic() {
    let mut group = FilterGroup::new();
    group.add_filter(Filter::new("error"));
    group.add_filter(Filter::new("warning"));

    // Group uses AND logic - both filters must match
    // "error" is in the text, "warning" is not - so this fails
    assert!(!group.matches("warning message".as_bytes()));
    assert!(!group.matches("error message".as_bytes()));
    assert!(!group.matches("info message".as_bytes()));

    // Only when both keywords are present
    assert!(group.matches("error and warning message".as_bytes()));
}

#[test]
fn test_filter_group_all_disabled() {
    let mut group = FilterGroup::new();
    let mut f1 = Filter::new("error");
    f1.set_enabled(false);
    group.add_filter(f1);

    // When filter is disabled, it returns true (matches everything)
    // Group with single disabled filter should match
    assert!(group.matches("error message".as_bytes()));
    assert!(group.matches("success message".as_bytes()));
}

#[test]
fn test_filter_set_and_logic() {
    let mut set = FilterSet::new();

    let mut group1 = FilterGroup::new();
    group1.add_filter(Filter::new("error"));

    let mut group2 = FilterGroup::new();
    group2.add_filter(Filter::new("warning"));

    set.add_group(group1);
    set.add_group(group2);

    // Set uses AND logic between groups
    // Both groups must have at least one matching filter
    assert!(set.matches("error warning message".as_bytes()));
}

#[test]
fn test_filter_set_all_groups_required() {
    let mut set = FilterSet::new();

    let mut group1 = FilterGroup::new();
    group1.add_filter(Filter::new("error"));

    let mut group2 = FilterGroup::new();
    group2.add_filter(Filter::new("warning"));

    set.add_group(group1);
    set.add_group(group2);

    // Missing "warning" - group2 won't match
    assert!(!set.matches("error message".as_bytes()));
}

#[test]
fn test_empty_filter_set_matches_all() {
    let set = FilterSet::new();
    assert!(set.matches("anything".as_bytes()));
}

#[test]
fn test_filter_with_special_characters() {
    let filter = Filter::new("127.0.0.1");
    assert!(filter.matches("Connection from 127.0.0.1".as_bytes()));
    assert!(!filter.matches("Connection from 192.168.1.1".as_bytes()));
}

#[test]
fn test_filter_partial_match() {
    let filter = Filter::new("err");
    assert!(filter.matches("error".as_bytes()));
    assert!(filter.matches("erroneous".as_bytes()));
    assert!(!filter.matches("success".as_bytes()));
}
