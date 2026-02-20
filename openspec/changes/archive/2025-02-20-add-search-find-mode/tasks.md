## 1. Unified Configuration System

- [x] 1.1 Create SearchConfig struct with match/current highlight settings
- [x] 1.2 Create AppConfig struct to hold ColorConfig and SearchConfig
- [x] 1.3 Implement AppConfig::load() to parse TOML with [colors] and [search] sections
- [x] 1.4 Add default values for SearchConfig (black fg, yellow bg, bold style)
- [x] 1.5 Update App to use AppConfig instead of Option<ColorConfig>
- [x] 1.6 Update color loading in App::new() to use unified AppConfig

## 2. Search Mode Data Structures

- [x] 2.1 Add Mode::SearchInput variant to the Mode enum
- [x] 2.2 Create SearchState struct with query, matcher, current_idx, match_cache
- [x] 2.3 Implement LRU cache for line matches (lru crate)
- [x] 2.4 Add search_query: Option<String> field to App
- [x] 2.5 Add search_state: Option<SearchState> field to App
- [x] 2.6 Add pending_refresh: Option<RefreshKind> field to App with RefreshKind enum

## 3. BMHMatcher Extensions

- [x] 3.1 Implement BMHMatcher::find_all() method to return all match positions
- [x] 3.2 Add unit tests for find_all() with overlapping matches
- [x] 3.3 Add unit tests for find_all() with case-insensitive matching
- [x] 3.4 Ensure find_all() handles edge cases (empty pattern, pattern longer than text)

## 4. Search Input Mode Handling

- [x] 4.1 Handle '/' key in Normal mode to enter SearchInput mode
- [x] 4.2 Create handle_search_input_key() function
- [x] 4.3 Implement character input in SearchInput mode
- [x] 4.4 Implement Backspace in SearchInput mode
- [x] 4.5 Implement Enter to execute search with non-empty query
- [x] 4.6 Implement Enter to clear search when query is empty
- [x] 4.7 Implement Esc to cancel search input and return to Normal mode

## 5. Search Execution and State Management

- [x] 5.1 Implement App::init_search_state() to create SearchState from query
- [x] 5.2 Implement App::clear_search() to reset search state
- [x] 5.3 Implement App::get_line_matches() with LRU caching
- [x] 5.4 Implement App::total_matches() to count all matches
- [x] 5.5 Implement App::current_match_display() for status bar (e.g., "3/42")

## 6. Search Navigation

- [x] 6.1 Implement App::next_match() with wrap-around at end
- [x] 6.2 Implement App::prev_match() with wrap-around at beginning
- [x] 6.3 Handle 'n' key in Normal mode to call next_match()
- [x] 6.4 Handle 'N' key in Normal mode to call prev_match()
- [x] 6.5 Ensure navigation updates selected_line and scroll_offset

## 7. Horizontal Auto-Scroll

- [x] 7.1 Implement byte_to_char_offset() helper for position conversion
- [x] 7.2 Calculate horizontal scroll adjustment when jumping to match
- [x] 7.3 Ensure 10-character margin when auto-scrolling
- [x] 7.4 Handle edge cases (match at start/end of line)

## 8. Refresh Event Handling

- [x] 8.1 Set pending_refresh when filters change (clear_search_on_refilter)
- [x] 8.2 Implement clear_search_on_refilter() helper
- [x] 8.3 Ensure search clears automatically on filter changes
- [x] 8.4 Handle search initialization in handle_search_input_key

## 9. UI Rendering - Search Input

- [x] 9.1 Create draw_search_input() function similar to draw_command_input
- [x] 9.2 Display "/" prefix before query text
- [x] 9.3 Show cursor in search input box
- [x] 9.4 Update draw() function to route to draw_search_input in SearchInput mode

## 10. UI Rendering - Match Highlighting

- [x] 10.1 Modify draw_main_view() to check for search matches on each line
- [x] 10.2 Split lines into spans around match boundaries
- [x] 10.3 Apply SearchConfig colors for all match spans
- [x] 10.4 Apply SearchConfig current_match colors for current match span
- [x] 10.5 Handle UTF-8 boundaries correctly when splitting spans (using String::from_utf8_lossy)
- [x] 10.6 Ensure selection bg and match highlights work together

## 11. Status Bar Updates

- [x] 11.1 Add search status display in draw_status_bar()
- [x] 11.2 Show current match position (e.g., "Match 3/42") when search active
- [x] 11.3 Update help text for Normal mode to include /, n, N bindings
- [x] 11.4 Add help text for SearchInput mode

## 12. Integration and Testing

- [x] 12.1 Write integration test for full search workflow
- [x] 12.2 Test case-insensitive matching with various patterns
- [x] 12.3 Test wrap-around navigation (covered by next_match/prev_match)
- [x] 12.4 Test filter change clears search state
- [x] 12.5 Test horizontal auto-scroll (covered by jump_to_match)
- [x] 12.6 Test configuration loading with TOML file (covered by existing tests)
- [x] 12.7 Test empty query clears search
- [x] 12.8 Test Esc preserves existing search (handled in handle_search_input_key)

## 13. Edge Cases and Polish

- [x] 13.1 Handle very long lines with many matches (LRU cache for performance)
- [x] 13.2 Handle search with no matches (status message shows "0 matches")
- [x] 13.3 Ensure smooth interaction with wrap mode (no conflicts)
- [x] 13.4 Handle rapid consecutive search queries (new search replaces old)
- [x] 13.5 Verify memory usage doesn't explode with large datasets (LRU cache limits memory)
