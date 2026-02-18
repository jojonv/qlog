## 1. Core Data Structure Changes

- [ ] 1.1 Replace `filtered_logs: Vec<LogEntry>` with `filtered_indices: Vec<usize>` in App struct
- [ ] 1.2 Update `App::new()` to initialize empty `filtered_indices` instead of `filtered_logs`
- [ ] 1.3 Add `filtered_len() -> usize` method returning `filtered_indices.len()`
- [ ] 1.4 Add `get_filtered_entry(idx: usize) -> Option<&LogEntry>` accessor method

## 2. Filter Logic Updates

- [ ] 2.1 Refactor `update_filtered_logs()` to store indices instead of cloned entries
- [ ] 2.2 Ensure filter matching logic works with index-based approach

## 3. UI Rendering Updates

- [ ] 3.1 Update `draw_main_view()` in ui/mod.rs to use `get_filtered_entry()` for log display
- [ ] 3.2 Update scrollbar calculations to use `filtered_len()` instead of `filtered_logs.len()`
- [ ] 3.3 Update filter bar display to use new accessor methods
- [ ] 3.4 Update status bar line count to use `filtered_len()`
- [ ] 3.5 Update loading screen entry count display if needed

## 4. Navigation and Selection Updates

- [ ] 4.1 Update `handle_normal_key()` selection bounds to use `filtered_len()`
- [ ] 4.2 Update `clamp_scroll()` to use `filtered_len()` for bounds checking
- [ ] 4.3 Update visual line calculations in `recalculate_visual_lines()` to use indices

## 5. Command Handling Updates

- [ ] 5.1 Update `write_filtered_logs()` to iterate using indices instead of `filtered_logs`
- [ ] 5.2 Update `execute_command()` if it references `filtered_logs.len()`

## 6. Verification

- [ ] 6.1 Build and verify no compilation errors
- [ ] 6.2 Test with sample log file - filtering works correctly
- [ ] 6.3 Test navigation (j/k, g/G, scrolling) works correctly
- [ ] 6.4 Test filter toggle on/off works correctly
- [ ] 6.5 Test `:w` command exports filtered logs correctly
- [ ] 6.6 Verify memory reduction with large log file
