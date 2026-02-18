## Context

The como-log-viewer is a TUI application for analyzing log files. Currently, it:
1. Collects all matching file paths using `glob::glob()` into a Vec
2. Passes paths to a background thread
3. Loads each file sequentially using BufReader

**Problem**: `glob()` holds directory handles open during iteration. With patterns like `/var/log/**/*.log` spanning thousands of directories and files, combined with the 1024 FD limit, the application crashes with FD exhaustion.

**Current FD Usage Pattern:**
- N directory handles from glob iterator (unbounded)
- 1 file handle per load operation
- Total: N + 1, where N can exceed 1024

## Goals / Non-Goals

**Goals:**
- Reduce max FD usage to ~50-60 (safe margin below 1024 limit)
- Support processing millions of log files without FD exhaustion
- Maintain responsive UI during loading
- Handle FD exhaustion gracefully with clear error messages

**Non-Goals:**
- Parallel file loading (adds complexity, marginal benefit for this use case)
- Persistent caching/indexing (out of scope)
- Network file support (local files only)

## Decisions

### Decision 1: Replace glob with WalkDir

**Choice**: Use `walkdir::WalkDir` with `max_open(10)`

**Rationale**: 
- `walkdir` allows explicit control over concurrent directory handles
- `max_open(10)` limits directory traversal to 10 concurrent FDs
- Streaming iterator pattern - no upfront collection

**Alternatives considered:**
- Keep `glob`, add batch processing: Doesn't solve root cause (glob still holds handles)
- Custom recursive traversal: Reinventing walkdir, error-prone

### Decision 2: Sequential File Processing

**Choice**: Process files one at a time in discovery order (no parallel loading)

**Rationale**:
- Simpler implementation, easier to reason about FD usage
- BufReader is fast for line-by-line parsing
- Avoids thread synchronization overhead
- UI updates are already incremental via channels

**Alternatives considered:**
- Parallel loading with rayon + semaphore: More FDs, more complexity, marginal speedup
- Async file I/O: Requires tokio, adds significant complexity to existing sync codebase

### Decision 3: Memory-Mapped I/O for Large Files

**Choice**: Use `memmap2` for files > 10MB, BufReader for smaller files

**Rationale**:
- Already in Cargo.toml (no new dependency)
- Reduces memory copies for large files
- OS handles paging efficiently
- Threshold at 10MB balances mmap overhead vs benefit

**Alternatives considered:**
- Always use mmap: Overkill for small files (<1KB logs are common)
- Never use mmap: Loses benefit for large files, but simpler
- Read entire file: Memory explosion for large logs

### Decision 4: Streaming Path Discovery

**Choice**: Discover and process files in a single streaming pass

**Rationale**:
- No path collection phase = lower memory
- Process-as-discovered = faster time-to-first-result
- Natural FD cleanup after each file

**Architecture:**
```
[WalkDir iterator] 
    → [filter by log extension]
    → [load and parse file]
    → [send to UI channel]
    → [drop file handle]
```

## Risks / Trade-offs

**Risk: WalkDir max_open too aggressive**
→ Mitigation: Make max_open configurable via env var, default to 10

**Risk: Out-of-order loading breaks user expectations**
→ Mitigation: Sort discovered paths before processing if <10k files, otherwise document streaming behavior

**Risk: mmap fails on special files (pipes, sockets)**
→ Mitigation: Fallback to BufReader on mmap error

**Risk: FD exhaustion still possible (system-wide limit)**
→ Mitigation: Catch EMFILE/ENFILE errors, pause 100ms, retry with exponential backoff, fail gracefully after 3 retries

**Trade-off: Slower startup vs reliable execution**
→ Users prefer slower but working over fast but crash-prone

## Migration Plan

**Phase 1: Add walkdir, keep glob temporarily**
- Add walkdir dependency
- Implement streaming loader as separate module
- Keep existing glob-based loader

**Phase 2: Switch to streaming loader**
- Replace glob call with walkdir
- Remove glob dependency
- Test with large directories

**Rollback**: Revert to previous commit if streaming loader shows issues

## Open Questions

1. Should we preserve file ordering? Current glob gives sorted results, walkdir is directory-order.
   - Recommendation: Sort if <10k files, otherwise document streaming behavior

2. Should max_open be configurable?
   - Recommendation: Yes, via environment variable `COMO_MAX_OPEN_DIRS` defaulting to 10
