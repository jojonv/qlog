## 1. Core Algorithm Implementation

- [x] 1.1 Create `BMHMatcher` struct in `src/model/` (or `src/utils/` if exists)
- [x] 1.2 Implement BMH skip table generation: `build_skip_table(pattern: &[u8]) -> [usize; 256]`
- [x] 1.3 Implement BMH search method: `find(&self, text: &[u8]) -> Option<usize>`
- [x] 1.4 Add unit tests for BMH matcher with various patterns (empty, single char, not found, multiple matches)

## 2. Byte-based Pattern Preprocessing

- [x] 2.1 Add method to convert filter patterns to lowercase bytes: `pattern_to_lowercase_bytes(text: &str) -> Vec<u8>`
- [x] 2.2 Store preprocessed patterns in Filter struct (consider caching skip tables)
- [x] 2.3 Ensure ASCII-only case folding (A-Z ↔ a-z at byte level)

## 3. Filter Evaluation Refactoring

- [x] 3.1 Identify current naive matching code in `src/model/filter.rs` (around line 106-117)
- [x] 3.2 Replace naive nested loop matching with BMH matcher calls
- [x] 3.3 Remove `as_str_lossy()` conversions - match directly on `&[u8]` from mmap
- [x] 3.4 Ensure case-insensitive matching uses preprocessed lowercase bytes
- [x] 3.5 Verify all existing filter tests still pass

## 4. Early Termination Optimization

- [x] 4.1 Locate filter group evaluation logic (AND-combined groups)
- [x] 4.2 Add early termination: break from group loop on first mismatch in AND mode
- [x] 4.3 Ensure OR-combined groups still evaluate all patterns (no early termination)
- [x] 4.4 Add tests verifying early termination behavior

## 5. Integration and Testing

- [x] 5.1 Run existing filter unit tests to ensure no regressions
- [x] 5.2 Run integration tests with sample log files
- [x] 5.3 Verify filter UI behavior unchanged (j/k navigation, add/remove filters)
- [x] 5.4 Test edge cases: empty patterns, unicode patterns, very long patterns (>100 chars)

## 6. Performance Validation

- [x] 6.1 Create benchmark comparing naive vs BMH matching on realistic log data
- [x] 6.2 Measure allocation reduction using heap profiling tools
- [x] 6.3 Verify 10x+ speedup on typical filter patterns
- [x] 6.4 Document benchmark results in PR description

## 7. Documentation

- [x] 7.1 Add doc comments to `BMHMatcher` struct and methods
- [x] 7.2 Update README or code comments explaining the optimization
- [x] 7.3 Note byte-level case folding limitation (ASCII-only) in comments
