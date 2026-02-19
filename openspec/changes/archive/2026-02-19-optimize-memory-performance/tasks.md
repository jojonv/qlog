## 1. Core Types

- [x] 1.1 Create `MmapStr` type in `src/model/mmap_str.rs` with `as_bytes()` and `as_str_lossy()` methods
- [x] 1.2 Create `LineInfo` struct with `offset: u64`, `length: u32`, `timestamp: Option<DateTime<Utc>>`
- [x] 1.3 Create `LogStorage` struct that owns mmap and `Vec<LineInfo>`
- [x] 1.4 Implement `LogStorage::get_line(idx) -> MmapStr` returning zero-copy view
- [x] 1.5 Implement `LogStorage::len()` and `iter()` for storage access
- [x] 1.6 Add lifetime parameter to `MmapStr` tied to `LogStorage`

## 2. Storage Loader Refactor

- [x] 2.1 Modify `src/storage/loader.rs` to build line index instead of Vec<String>
- [x] 2.2 Implement newline scanning to populate `Vec<LineInfo>` with offsets and lengths
- [x] 2.3 Extract timestamp during scan (keep existing timestamp parsing logic)
- [x] 2.4 Remove `LogEntry::new()` with String allocation
- [x] 2.5 Return `LogStorage` instead of `Vec<LogEntry>`

## 3. Filter Optimization

- [x] 3.1 Add `cached_lower: Vec<u8>` field to `Filter` struct in `src/model/filter.rs`
- [x] 3.2 Implement filter text caching on creation/modification
- [x] 3.3 Create zero-allocation `matches(bytes: &[u8]) -> bool` method
- [x] 3.4 Use ASCII lowercase byte comparison without String allocation
- [x] 3.5 Update `FilterGroup::matches()` to use new byte-based matching

## 4. App Integration

- [x] 4.1 Replace `Vec<LogEntry>` with `LogStorage` in `App` struct
- [x] 4.2 Update `filtered_indices` population to use new storage API
- [x] 4.3 Update all `self.logs[idx]` accesses to use `self.storage.get_line(idx)`
- [x] 4.4 Fix lifetime annotations throughout App (storage must outlive views)

## 5. Visual Line Cache

- [x] 5.1 Create `VisualLineCache` struct with LRU-style caching
- [x] 5.2 Replace `visual_line_offsets: Vec<usize>` with `VisualLineCache`
- [x] 5.3 Implement on-demand calculation for visible + buffer range
- [x] 5.4 Remove `recalculate_visual_lines()` full traversal
- [x] 5.5 Remove `recalculate_max_line_width()` or make it viewport-scoped

## 6. UI Layer Updates

- [x] 6.1 Update `src/ui/mod.rs` to work with `MmapStr` instead of `&str`
- [x] 6.2 Ensure lossy UTF-8 conversion happens at display time only
- [x] 6.3 Verify viewport rendering still works with lazy visual line cache

## 7. Cleanup and Testing

- [x] 7.1 Remove dead code from old `LogEntry` if fully replaced
- [x] 7.2 Update any remaining String-based code paths
- [x] 7.3 Test with 2GB+ log file to verify <4GB memory usage
- [x] 7.4 Test filtering performance improvement
- [x] 7.5 Test with non-UTF-8 file to verify lossy handling
- [x] 7.6 Test random access (jump to end, jump to middle)
