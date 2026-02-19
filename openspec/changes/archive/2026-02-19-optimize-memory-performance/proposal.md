## Why

The application consumes ~15GB of memory when loading 2GB log files—7.5× the file size. Filtering is slow due to repeated string allocations on every filter check. The target is <4GB memory usage in all scenarios, with significantly faster filtering.

Root causes:
1. Every log line stored as heap-allocated `String` with 3-4× overhead
2. `to_lowercase()` creates new strings on every filter check
3. Visual line calculations traverse all entries, not just visible ones
4. Vec growth strategy doubles capacity, wasting memory

## What Changes

- **BREAKING**: Replace `LogEntry::raw: String` with `MmapStr` (reference into memory-mapped file)
- Store only line offsets (12 bytes/line) instead of full content copies
- Case-insensitive byte comparison (zero-allocation filtering)
- Filter text caching (convert to lowercase bytes once)
- Lazy visual line calculation (only visible + buffer lines)
- Remove `recalculate_max_line_width()` and full visual line recalculation
- Unified mmap approach for all file sizes (no special case for small files)
- Lossy UTF-8 handling for robustness (accept any file encoding)

## Capabilities

### New Capabilities

- `mmap-log-storage`: Memory-mapped log storage with line index. Provides O(1) random access, zero-copy string views, and predictable memory footprint (~file size + 10% for indices).

### Modified Capabilities

- (none - this is an internal optimization, no spec-level behavior changes)

## Impact

**Breaking API Changes**:
- `LogEntry::raw` type changes from `String` to `MmapStr`
- Lifetime management required (mmap must outlive entries)
- Any code directly accessing `entry.raw` will need updates

**Affected Files**:
- `src/storage/loader.rs` - replace String loading with mmap + line index
- `src/model/log_entry.rs` - replace String with MmapStr
- `src/model/filter.rs` - zero-allocation case-insensitive matching
- `src/app.rs` - lazy visual line calculation, remove full recalculations

**Memory Impact**:
- 2GB file: 15GB → ~2.2GB (7× reduction)
- 4GB file: ~4.4GB (within target)

**Performance Impact**:
- Filtering: 10-100× faster (no string allocations)
- Startup: similar or faster (mmap is lazy)
- Random access: O(1) same as before
