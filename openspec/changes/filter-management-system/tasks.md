## 1. Filter Modal UI Component

- [ ] 1.1 Create FilterModal widget structure in src/ui/filter_modal.rs
- [ ] 1.2 Implement modal layout with centered positioning and background dimming
- [ ] 1.3 Add filter type dropdown (Level, Text, Date, Source Context)
- [ ] 1.4 Implement Level filter form with radio buttons
- [ ] 1.5 Implement Text filter form with input, case-sensitive checkbox, regex checkbox
- [ ] 1.6 Implement Date filter form with From/To date inputs
- [ ] 1.7 Implement Source Context filter form with autocomplete
- [ ] 1.8 Add input validation (empty check, regex validation)
- [ ] 1.9 Show active filters sidebar in modal
- [ ] 1.10 Handle Escape and 'q' keys to close modal
- [ ] 1.11 Wire 'f' key to open filter modal in App event handler
- [ ] 1.12 Implement relative date parsing ("2 hours ago", "yesterday", "today", etc.)
- [ ] 1.13 Add shorthand date syntax ("-1h", "-30m", "now")
- [ ] 1.14 Convert relative expressions to absolute timestamps on filter creation
- [ ] 1.15 Validate relative date syntax and show error for invalid input

## 2. Filter Bar UI Component

- [ ] 2.1 Create FilterBar widget in src/ui/filter_bar.rs
- [ ] 2.2 Implement filter chips with visual styling
- [ ] 2.3 Add enabled/disabled visual states (opacity, strikethrough)
- [ ] 2.4 Implement keyboard navigation between chips (Tab, arrows)
- [ ] 2.5 Add Space key handler to toggle filter state
- [ ] 2.6 Add 't' key handler to toggle selected filter (Helix style)
- [ ] 2.7 Add 'd' key handler to delete selected filter
- [ ] 2.8 Add Delete key handler to remove selected filter
- [ ] 2.9 Implement 'F' key to clear all filters with confirmation
- [ ] 2.10 Position filter bar at top of main view
- [ ] 2.11 Update main UI layout to include filter bar
- [ ] 2.12 Add NOT (negation) toggle with '!' and Ctrl+n keys
- [ ] 2.13 Add NOT visual indicator (red border, "NOT" prefix, red color accent)
- [ ] 2.14 Extend Filter struct to include negated field
- [ ] 2.15 Implement negated filter evaluation (show logs that do NOT match)
- [ ] 2.16 Calculate and display match count per filter chip (format: [Error (152)])
- [ ] 2.17 Update match counts live during log loading
- [ ] 2.18 Display "99k+" for counts exceeding 100,000
- [ ] 2.19 Disable match counts when loading > 1M logs (performance)
- [ ] 2.20 Show exact count in tooltip on chip focus

## 3. Filter Logic System

- [ ] 3.1 Extend Filter struct to include enabled field
- [ ] 3.2 Create FilterOperator enum (And, Or)
- [ ] 3.3 Update App state to store Vec<Filter> and Vec<FilterOperator>
- [ ] 3.4 Implement filter evaluation with AND/OR logic
- [ ] 3.5 Add visual operators (●, ○) between filter chips
- [ ] 3.6 Implement click handler to toggle operator state
- [ ] 3.7 Implement keyboard handler (Enter) to toggle operator
- [ ] 3.8 Ensure filter results update immediately on logic change
- [ ] 3.9 Add default AND operator when adding second filter
- [ ] 3.10 Test complex filter combinations: (A AND B) OR C

## 4. Filter Persistence

- [ ] 4.1 Create Config module in src/config.rs
- [ ] 4.2 Define FilterConfig struct for serialization
- [ ] 4.3 Implement get_config_path() with XDG directory support
- [ ] 4.4 Add save_filters() function to write JSON to disk
- [ ] 4.5 Add load_filters() function to read JSON from disk
- [ ] 4.6 Implement error handling for corrupted/missing config files
- [ ] 4.7 Auto-save filters on application exit
- [ ] 4.8 Auto-restore filters on application startup
- [ ] 4.9 Handle migration when loading older config versions
- [ ] 4.10 Add manual save/load command infrastructure (for future :save/:load)
- [ ] 4.11 Create FilterHistory struct in src/history.rs
- [ ] 4.12 Implement history tracking (last 20 unique filter combinations)
- [ ] 4.13 Persist history to ~/.config/como-log-viewer/history.json
- [ ] 4.14 Add timestamp, filters, operators, summary, match_count to each entry
- [ ] 4.15 Create HistoryPopup widget for displaying history
- [ ] 4.16 Wire Ctrl+h to open history popup
- [ ] 4.17 Wire :history command to open history popup
- [ ] 4.18 Implement j/k navigation in history popup
- [ ] 4.19 Implement Enter to apply selected history entry
- [ ] 4.20 Remove oldest entry when history exceeds 20 entries

## 5. Integration and Testing

- [ ] 5.1 Integrate FilterModal into App event loop
- [ ] 5.2 Integrate FilterBar into main UI layout
- [ ] 5.3 Update filtered entries when filters change
- [ ] 5.4 Add unit tests for filter evaluation logic
- [ ] 5.5 Add unit tests for filter persistence (save/load)
- [ ] 5.6 Add integration tests for filter modal workflow
- [ ] 5.7 Test keyboard shortcuts in all modes
- [ ] 5.8 Verify Helix-style keybindings work consistently
- [ ] 5.9 Test with actual log files (7.6M entries)
- [ ] 5.10 Profile performance with multiple active filters

## 6. Documentation and Polish

- [ ] 6.1 Update README with filter management features
- [ ] 6.2 Add keyboard shortcut reference to help text
- [ ] 6.3 Create example filter configurations
- [ ] 6.4 Add inline help text in filter modal
- [ ] 6.5 Show filter match counts in status bar
- [ ] 6.6 Add visual feedback when filters are applied/removed
- [ ] 6.7 Test edge cases (empty filter list, all filters disabled)
- [ ] 6.8 Verify no regressions in existing navigation
- [ ] 6.9 Final code review and cleanup
- [ ] 6.10 Update CHANGELOG.md
