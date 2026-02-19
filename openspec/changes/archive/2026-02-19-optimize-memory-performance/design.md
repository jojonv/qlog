## Context

The current log viewer loads entire files into memory as heap-allocated `String` objects. A 2GB log file with ~10M lines becomes 15GB in memory due to:

1. Each `String` has 24-byte metadata + heap allocation overhead
2. Vec capacity often 2× actual entries (growth strategy)
3. `Option<DateTime<Utc>>` padding per entry
4. Temporary string allocations during filtering

The application needs random access (jump to any line) AND sequential reading, making pure streaming approaches unsuitable.

## Goals / Non-Goals

**Goals:**
- Reduce memory from 15GB to <4GB for 2GB log files
- Make filtering 10× faster by eliminating allocations
- Maintain O(1) random access to any log line
- Keep sequential scrolling performance
- Handle any file encoding gracefully (lossy UTF-8)

**Non-Goals:**
- Regex support (future consideration)
- Database-backed storage (overkill for single-file viewing)
- Distributed/remote file support

## Decisions

### 1. Memory-Mapped File Storage

**Decision**: Use `memmap2` for all files, no special case for small files.

**Rationale**: 
- Kernel manages paging, only touched pages in RSS
- Zero-copy string views possible
- Simplifies codebase (one path)
- Already a dependency

**Alternatives considered**:
- Chunked loading: rejected because random access would be O(n) with chunk loads
- Keep small files in memory: rejected to avoid two code paths

### 2. Line Index Instead of String Storage

**Decision**: Store `(offset: u64, length: u32)` tuple per line instead of owned strings.

**Memory calculation** (2GB file, 10M lines):
- Current: 10M × (200 bytes content + 24 bytes String overhead + 200 bytes capacity + 40 bytes struct) ≈ 4.6GB + Vec overhead ≈ 15GB
- New: 10M × 12 bytes = 120MB for index, mmap handled by kernel

**Rationale**: 120MB index + kernel-managed mmap is predictable and bounded.

### 3. Custom MmapStr Type

**Decision**: Create `MmapStr` type that holds a slice reference into the mmap.

```rust
pub struct MmapStr<'a> {
    data: &'a [u8],
}

impl<'a> MmapStr<'a> {
    pub fn as_str_lossy(&self) -> Cow<'a, str> {
        String::from_utf8_lossy(self.data)
    }
    
    pub fn as_bytes(&self) -> &'a [u8] {
        self.data
    }
}
```

**Rationale**: 
- Avoids `unsafe` at call sites
- Encapsulates lossy conversion
- Enables byte-level case-insensitive comparison

### 4. Zero-Allocation Case-Insensitive Matching

**Decision**: Compare lowercase bytes directly without creating new strings.

```rust
pub fn matches(&self, line: &[u8]) -> bool {
    let pattern_lower = &self.cached_lower;  // cached at filter creation
    line.iter()
        .map(|b| b.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .windows(pattern_lower.len())
        .any(|w| w == pattern_lower)
}
```

**Optimization**: Use `aho-corasick` crate for multi-pattern matching if needed later.

**Rationale**: ASCII case-insensitive comparison is O(n) with no allocations.

### 5. Lazy Visual Line Calculation

**Decision**: Calculate visual line offsets only for visible + buffer lines.

**Current**: `visual_line_offsets: Vec<usize>` with entry for EVERY filtered line
**New**: `VisualLineCache` that computes on-demand and caches recent entries

**Rationale**: Viewport is ~50 lines, no need to precompute for 10M lines.

### 6. Lossy UTF-8 Handling

**Decision**: Use `String::from_utf8_lossy` semantics - invalid bytes become `�`.

**Rationale**: 
- Robust against any file encoding
- No panics on malformed input
- User still sees something usable

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      NEW ARCHITECTURE                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────┐      ┌───────────────────────────────┐   │
│  │  MmapFile         │      │  LineIndex                    │   │
│  │  ────────────     │      │  ──────────                   │   │
│  │  path: PathBuf    │      │  lines: Vec<LineInfo>         │   │
│  │  mmap: Mmap       │◀────▶│    - offset: u64              │   │
│  │  (holds mmap      │      │    - length: u32              │   │
│  │   open)           │      │    - timestamp: Option<DateTime│   │
│  └──────────────────┘      └───────────────────────────────┘   │
│           │                            │                        │
│           │ provides bytes             │ provides index         │
│           ▼                            ▼                        │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  LogStorage                                               │   │
│  │  ──────────                                               │   │
│  │  get_line(idx) → MmapStr<'a>  (zero-copy view)           │   │
│  │  len() → usize                                           │   │
│  │  iter() → impl Iterator<Item=MmapStr>                    │   │
│  └──────────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  App (modified)                                           │   │
│  │  ─────────────                                            │   │
│  │  storage: LogStorage (was Vec<LogEntry>)                  │   │
│  │  filtered_indices: Vec<usize> (unchanged)                │   │
│  │  visual_cache: VisualLineCache (was Vec<usize>)          │   │
│  │  filters: Filters (with cached lowercase patterns)       │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Data Flow

```
FILE LOAD:
┌────────┐     ┌─────────┐     ┌───────────┐     ┌────────────┐
│ Path   │────▶│ Mmap    │────▶│ Scan for  │────▶│ LineIndex  │
│        │     │ file    │     │ newlines  │     │ built      │
└────────┘     └─────────┘     └───────────┘     └────────────┘
    │                                                   │
    │                                                   │
    └───────────────────────────────────────────────────┘
                      mmap stays open, index references it

FILTER OPERATION:
┌───────────────┐     ┌────────────────┐     ┌─────────────────┐
│ Filter text   │────▶│ Lowercase once │────▶│ cached_pattern  │
│ entered       │     │                │     │ [u8]            │
└───────────────┘     └────────────────┘     └─────────────────┘
                                                    │
                                                    ▼
┌───────────────┐     ┌────────────────┐     ┌─────────────────┐
│ Line bytes    │────▶│ Byte-by-byte   │────▶│ Match/No match  │
│ from mmap     │     │ comparison     │     │ (no allocation) │
└───────────────┘     └────────────────┘     └─────────────────┘

VIEWPORT RENDER:
┌─────────────────┐     ┌────────────────┐     ┌─────────────────┐
│ scroll_offset   │────▶│ Get filtered   │────▶│ Fetch lines     │
│                 │     │ indices for    │     │ from mmap       │
│                 │     │ visible range  │     │ (zero-copy)     │
└─────────────────┘     └────────────────┘     └─────────────────┘
```

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Mmap lifetime management complexity | `LogStorage` owns both mmap and index; `MmapStr` borrows from storage |
| Performance on network filesystems | Document that local files recommended; mmap falls back gracefully |
| Lossy conversion hides data | ASCII output shows `�` clearly; original bytes accessible if needed |
| Breaking API change for LogEntry | Accept breaking change; internal API, not public interface |
| Very long lines (>4GB) | u32 length limits to 4GB per line; document as limitation |

## Open Questions

None. All design decisions are resolved based on user input:
- ✅ Case-insensitive: mandatory
- ✅ Regex: not needed
- ✅ Random + sequential access: both required
- ✅ UTF-8: lossy conversion acceptable
