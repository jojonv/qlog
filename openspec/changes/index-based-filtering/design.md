## Context

The log viewer loads all log entries into `logs: Vec<LogEntry>` and maintains a duplicate `filtered_logs: Vec<LogEntry>` for filtered results. With large log files (10GB+), this causes memory consumption of 15GB+ due to:

1. **Full duplication**: `.cloned().collect()` creates complete copies of all filtered entries
2. **String overhead**: Each `LogEntry` contains `raw: String` (24 bytes overhead + content)
3. **Dual storage**: Both original and filtered copies exist simultaneously

Competing tools (klogg, lnav) achieve low memory by using index-based references instead of cloning data.

## Goals / Non-Goals

**Goals:**
- Replace `filtered_logs: Vec<LogEntry>` with `filtered_indices: Vec<usize>`
- Eliminate data duplication during filtering
- Reduce memory consumption by 30-50%

**Non-Goals:**
- Memory-mapped file storage (future phase)
- Virtualized viewport/windowed loading (future phase)
- Changes to file loading mechanism

## Decisions

### D1: Index-based filtering storage

**Decision**: Store filtered results as indices into `logs` vector instead of cloned entries.

**Rationale**: 
- Indices are 8 bytes vs. LogEntry which is 32+ bytes + string content
- No data duplication during filter updates
- Faster filter application (no cloning)

**Alternatives considered**:
- `Vec<&LogEntry>`: Rejected - lifetime management complexity
- `Vec<Rc<LogEntry>>`: Rejected - still stores pointers, adds reference counting overhead

### D2: Accessor method pattern

**Decision**: Add helper method `get_filtered_entry(idx: usize) -> &LogEntry` to abstract index lookup.

**Rationale**:
- Single point of change for access logic
- Cleaner than `self.logs[self.filtered_indices[idx]]` everywhere
- Bounds checking in one location

### D3: Keep `filtered_len()` method

**Decision**: Add `filtered_len() -> usize` returning `filtered_indices.len()`.

**Rationale**:
- Semantic clarity over `filtered_indices.len()`
- Matches existing `filtered_logs.len()` usage pattern

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Bounds errors on index access | Use `get()` with proper error handling; add debug assertions |
| Stale indices after log modification | Logs are append-only after initial load; no modification after filtering |
| Slightly slower per-entry access | One indirection (`logs[indices[i]]`) vs direct access; negligible for UI rendering |
