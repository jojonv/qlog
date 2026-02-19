## Why

Current filtering implementation uses naive substring search with O(pattern_len × text_len) complexity, causing severe performance degradation on large log files (2GB+). For each log line, the system performs expensive UTF-8 conversions and nested character-by-character comparisons, resulting in 10-100x slower filtering than necessary.

## What Changes

- Implement Boyer-Moore-Horspool algorithm for pattern matching in filters
- Replace `as_str_lossy()` UTF-8 conversion with byte-based matching to eliminate per-line allocations
- Add early termination optimization for AND-combined filters (skip remaining patterns after first mismatch)
- Maintain backward compatibility with existing filter behavior (case-insensitive substring matching)
- **No breaking changes** to API or user-facing behavior

## Capabilities

### New Capabilities
- `optimized-string-matching`: High-performance substring matching for filter operations using Boyer-Moore-Horspool algorithm with O(n/m) average-case complexity

### Modified Capabilities
- `filter-groups`: Enhanced implementation to use optimized matching algorithm while maintaining existing behavior semantics

## Impact

- **Core**: `src/model/filter.rs` - Pattern matching algorithm replacement
- **Performance**: 10-100x speedup on typical filter patterns
- **Memory**: Eliminates per-line String allocations during filtering
- **API**: No changes to public interfaces
- **Compatibility**: Existing filters continue to work identically
