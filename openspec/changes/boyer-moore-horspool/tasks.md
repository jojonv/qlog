## 1. Core Algorithm Implementation

- [ ] 1.1 Create `BMHMatcher` struct in `src/model/` (or `src/utils/` if exists)
- [ ] 1.2 Implement BMH skip table generation: `build_skip_table(pattern: &[u8]) -> [usize; 256]`
- [ ] 1.3 Implement BMH search method: `find(&self, text: &[u8]) -> Option<usize>`
- [ ] 1.4 Add unit tests for BMH matcher with various patterns (empty, single char, not found, multiple matches)

## 2. Byte-based Pattern Preprocessing

- [ ] 2.1 Add method to convert filter patterns to lowercase bytes: `pattern_to_lowercase_bytes(text: &str) -> Vec<u8>`
- [ ] 2.2 Store preprocessed patterns in Filter struct (consider caching skip tables)
- [ ] 2.3 Ensure ASCII-only case folding (A-Z ↔ a-z at byte level)

## 3. Filter Evaluation Refactoring

- [ ] 3.1 Identify current naive matching code in `src/model/filter.rs` (around line 106-117)
- [ ] 3.2 Replace naive nested loop matching with BMH matcher calls
- [ ] 3.3 Remove `as_str_lossy()` conversions - match directly on `&[u8]` from mmap
- [ ] 3.4 Ensure case-insensitive matching uses preprocessed lowercase bytes
- [ ] 3.5 Verify all existing filter tests still pass

## 4. Early Termination Optimization

- [ ] 4.1 Locate filter group evaluation logic (AND-combined groups)
- [ ] 4.2 Add early termination: break from group loop on first mismatch in AND mode
- [ ] 4.3 Ensure OR-combined groups still evaluate all patterns (no early termination)
- [ ] 4.4 Add tests verifying early termination behavior

## 5. Integration and Testing

- [ ] 5.1 Run existing filter unit tests to ensure no regressions
- [ ] 5.2 Run integration tests with sample log files
- [ ] 5.3 Verify filter UI behavior unchanged (j/k navigation, add/remove filters)
- [ ] 5.4 Test edge cases: empty patterns, unicode patterns, very long patterns (>100 chars)

## 6. Performance Validation

- [ ] 6.1 Create benchmark comparing naive vs BMH matching on realistic log data
- [ ] 6.2 Measure allocation reduction using heap profiling tools
- [ ] 6.3 Verify 10x+ speedup on typical filter patterns
- [ ] 6.4 Document benchmark results in PR description

## 7. Documentation

- [ ] 7.1 Add doc comments to `BMHMatcher` struct and methods
- [ ] 7.2 Update README or code comments explaining the optimization
- [ ] 7.3 Note byte-level case folding limitation (ASCII-only) in comments
