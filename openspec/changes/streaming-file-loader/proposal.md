## Why

The application crashes with "An unknown error occurred, possibly due to low max file descriptors" when processing directories with many log files. The current implementation collects all file paths upfront using `glob()` (which holds directory handles open) and then loads files sequentially, exhausting the 1024 FD limit. This prevents users from analyzing large log directories and makes the tool unreliable for production use.

## What Changes

- Replace eager path collection with streaming file discovery
- Replace `glob::glob()` with `walkdir::WalkDir` to limit concurrent directory handles
- Implement bounded concurrency for file loading (max 50 concurrent opens)
- Use memory-mapped I/O for large files instead of reading into buffers
- Add proper error handling for FD exhaustion (EMFILE/ENFILE)
- Process files incrementally instead of holding all paths in memory
- **BREAKING**: Loading behavior changes from "collect all paths then load" to "discover and process incrementally"

## Capabilities

### New Capabilities

- `streaming-loader`: Incremental file discovery and processing with bounded resource usage
- `fd-exhaustion-handling`: Graceful degradation when file descriptor limits are reached

### Modified Capabilities

- None (no existing specs found)

## Impact

**Affected Code:**
- `src/main.rs` - Replace glob with walkdir, restructure main loading flow
- `src/storage/loader.rs` - Add streaming loader, implement mmap support
- `src/app.rs` - Handle incremental log loading updates

**Dependencies:**
- Add `walkdir` crate (for streaming directory traversal with max_open control)
- Use existing `memmap2` crate (already in Cargo.toml but unused)
- Add `rayon` crate (optional, for parallel processing with bounded concurrency)

**Systems:**
- File descriptor usage drops from unbounded to max ~50-60
- Memory usage reduced (no path collection bottleneck, streaming processing)
- Scalability improved (handles millions of files without FD exhaustion)
