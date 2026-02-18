## 1. Dependencies and Setup

- [ ] 1.1 Add `walkdir` crate to Cargo.toml
- [ ] 1.2 Verify `memmap2` crate is available (already in dependencies)

## 2. Streaming Directory Traversal

- [ ] 2.1 Replace `glob::glob()` with `walkdir::WalkDir` in src/main.rs
- [ ] 2.2 Implement `max_open(10)` limit on WalkDir to bound directory handles
- [ ] 2.3 Add `COMO_MAX_OPEN_DIRS` environment variable support for configurable limit
- [ ] 2.4 Create streaming file iterator that yields paths incrementally (no collection)

## 3. Memory-Mapped File Loading

- [ ] 3.1 Add file size check (>10MB threshold) in src/storage/loader.rs
- [ ] 3.2 Implement mmap-based loading for large files using memmap2
- [ ] 3.3 Add fallback to BufReader if mmap fails (permission, empty file, etc.)
- [ ] 3.4 Ensure file handles are dropped immediately after processing

## 4. FD Exhaustion Handling

- [ ] 4.1 Create error detection helper for EMFILE/ENFILE errors
- [ ] 4.2 Implement exponential backoff retry (100ms→200ms→400ms→800ms, max 3 retries)
- [ ] 4.3 Add user-friendly error message with 'ulimit -n 65536' suggestion
- [ ] 4.4 Track and report failed file count with first 5 paths at completion
- [ ] 4.5 Implement FD usage warning at 80% threshold

## 5. Incremental Loading Flow

- [ ] 5.1 Refactor main loading loop to process files one at a time
- [ ] 5.2 Send progress updates after each file processed
- [ ] 5.3 Update src/app.rs to handle incremental log arrivals
- [ ] 5.4 Remove eager path collection (Vec<PathBuf> gathering)

## 6. Testing and Verification

- [ ] 6.1 Test with directory containing 5000+ log files
- [ ] 6.2 Verify total FD usage stays under 50 during loading
- [ ] 6.3 Test FD exhaustion recovery with artificially low ulimit
- [ ] 6.4 Verify mmap fallback works correctly
- [ ] 6.5 Test COMO_MAX_OPEN_DIRS environment variable
