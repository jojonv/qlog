## Context

The como-log-viewer is a Rust TUI application built with ratatui that displays log files. Currently, it only applies minimal coloring: timestamps are cyan and the selected line has a dark gray background. Users need a way to visually highlight important log patterns.

The application uses memory-mapped file access for efficient large file handling and renders lines through ratatui's `Paragraph` and `Line` widgets. The UI rendering logic is centralized in `src/ui/mod.rs`.

**Current Architecture:**
- `src/ui/mod.rs` - All UI rendering with ratatui
- `src/model/` - Data structures (LogStorage, LineInfo, etc.)
- Logs are rendered as `Line` objects with `Span` elements
- Colors are applied via ratatui's `Style` with `Color` enum

## Goals / Non-Goals

**Goals:**
- Enable configurable pattern-based log line coloring via TOML files
- Support case-insensitive partial matching for flexibility
- Support wildcard patterns for flexible matching
- Use ratatui color names for consistency
- Cross-platform config file discovery (current dir + home dir)

**Non-Goals:**
- Runtime configuration reloading (config loaded once at startup)
- Per-character or regex-based coloring (entire lines only)
- Multiple colors per line (first match wins)
- Background color support (foreground only for now)
- CLI flag for config path
- Parent directory config discovery

## Decisions

### Decision: Use TOML over JSON
**Rationale:** TOML supports quoted keys with special characters (like `"*error*"`) more naturally than JSON. It also allows trailing commas and is more human-readable for configuration.

### Decision: Wildcard syntax with `*`
**Rationale:** Simple and intuitive. `*error` = ends with, `error*` = starts with, `*error*` = contains. This covers 99% of use cases without complex regex overhead.

**Alternative considered:** Full regex support - rejected for performance and simplicity.

### Decision: First-match-wins precedence
**Rationale:** Config file order determines priority, giving users explicit control over which pattern takes precedence. This is simple and predictable.

**Alternative considered:** Priority levels or most-specific-match - rejected for complexity.

### Decision: Add `toml` and `dirs` dependencies
**Rationale:**
- `toml`: Standard Rust TOML parser, well-maintained
- `dirs`: Industry standard for cross-platform home directory detection

Both are lightweight and widely used in the Rust ecosystem.

### Decision: No coloring when config not found
**Rationale:** Explicit opt-in prevents unexpected behavior. Users must create config to get coloring, making it a conscious choice.

### Decision: Pattern matching at render time
**Rationale:** Checking patterns during rendering keeps the implementation simple and avoids storing pattern state in LogStorage. Since we're coloring entire lines (not tokens), we only need to check once per line at render time.

**Alternative considered:** Pre-compute matches during file loading - rejected because it would require storing pattern match results in LineInfo, complicating the data model.

## Risks / Trade-offs

**Performance impact with many patterns** → Each visible line checks all patterns. With 100+ patterns, this could cause lag.
*Mitigation:* Optimize by compiling patterns once, cache compiled matchers. Document recommendation to keep pattern count reasonable (<50).

**Color conflicts with existing styling** → Colored lines may conflict with selection highlight.
*Mitigation:* Selection background overrides line color (keep current behavior - selected line gets dark gray background regardless of line color).

**Case-insensitivity for non-ASCII** → Unicode case folding is complex.
*Mitigation:* Document that matching is ASCII-case-insensitive only. For most log files this is sufficient.

**TOML parsing errors** → Invalid TOML could crash the app.
*Mitigation:* Graceful error handling - log error to stderr and proceed with no coloring.

## Migration Plan

This is a pure addition with no breaking changes:
1. Add `toml` and `dirs` to Cargo.toml dependencies
2. Create `src/config.rs` module
3. Modify `src/ui/mod.rs` to apply coloring
4. Test with sample TOML config
5. Update documentation with config examples

Rollback: Remove config module and revert UI changes. No data migration needed.

## Open Questions

None at this time - design is complete.
