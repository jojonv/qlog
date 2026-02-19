## Context

Current filtering implementation in `src/model/filter.rs` uses naive substring search with nested loops, resulting in O(pattern_len × text_len) complexity per line. Each log line triggers expensive UTF-8 conversion via `as_str_lossy()` which allocates a String, followed by character-by-character comparison. For large log files (2GB+), this creates millions of allocations and becomes a severe performance bottleneck.

## Goals / Non-Goals

**Goals:**
- Implement Boyer-Moore-Horspool algorithm for O(n/m) average-case pattern matching
- Eliminate per-line String allocations during filtering
- Add early termination for AND-combined filters (skip remaining patterns on first mismatch)
- Maintain identical user-facing filter behavior (case-insensitive substring matching)
- Achieve 10-100x speedup on typical filter patterns

**Non-Goals:**
- Changing filter semantics (OR/AND composition, case-insensitivity)
- Parallelization or SIMD optimization (future work)
- Adding new filter types or operators
- API changes to filter configuration

## Decisions

### Algorithm Choice: Boyer-Moore-Horspool over alternatives

**Choice**: Boyer-Moore-Horspool (BMH) algorithm for substring matching.

**Rationale**:
- **Boyer-Moore-Horspool**: Best for short patterns with large alphabet (like log text). Simpler to implement than full Boyer-Moore, comparable performance for typical patterns (< 100 chars).
- **KMP (Knuth-Morris-Pratt)**: Better for repetitive patterns, but BMH's bad-character heuristic is more effective for varied log content.
- **Naive**: Current approach, clearly insufficient.

**Trade-off**: BMH requires O(m) preprocessing to build skip table (m = pattern length), but this cost is amortized across millions of line comparisons.

### Byte-based Matching over UTF-8 Strings

**Choice**: Match against raw bytes instead of converting to UTF-8 strings.

**Rationale**:
- Eliminates `as_str_lossy()` allocation per line
- UTF-8 is backward-compatible with ASCII; case conversion can work on bytes
- Log files are predominantly ASCII; byte-level matching is sufficient

**Implementation**: Preprocess filter patterns to lowercase bytes, store as `Vec<u8>`. Compare directly against line bytes without conversion.

### Early Termination for AND Filters

**Choice**: Evaluate filter groups in order, terminate early on first mismatch.

**Rationale**:
- AND logic: all groups must match, so first failure means overall failure
- Typical filters have 2-5 groups; early exit saves 20-80% of comparisons
- Minimal code complexity: add `break` after mismatch in AND mode

## Risks / Trade-offs

**Risk**: Case-insensitive matching of non-ASCII UTF-8 characters may behave differently with byte-based matching.

**Mitigation**: Document limitation - byte-based case folding only affects ASCII range (A-Z/a-z). Non-ASCII characters are matched case-sensitively, which is acceptable for log files dominated by ASCII content.

**Risk**: Preprocessing skip table for each pattern adds upfront cost.

**Mitigation**: Cache skip tables per filter; patterns rarely change during a viewing session.

**Trade-off**: Memory vs Speed
- BMH requires O(m) extra memory per pattern for skip table (m = pattern length, typically 5-50 bytes)
- Net memory savings from eliminating per-line String allocations far exceeds this overhead

## Migration Plan

1. **Phase 1**: Implement `BMHMatcher` struct with `find()` method
2. **Phase 2**: Replace `as_str_lossy()` calls with direct byte matching in filter evaluation
3. **Phase 3**: Add early termination for AND-combined filters
4. **Phase 4**: Benchmark against naive implementation to verify speedup
5. **Rollback**: Revert to naive implementation if critical bugs found (code isolated in separate module)

## Open Questions

- Should we cache BMH skip tables in Filter struct, or compute on each match call?
- How to handle UTF-8 multi-byte characters in case-insensitive matching (likely out of scope)?
