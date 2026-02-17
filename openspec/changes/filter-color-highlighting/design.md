## Context

The filter-groups feature allows users to create filter groups (AND logic between groups, OR within groups). Currently, filters render with bold+underline for selection but lack visual distinction between groups. Log lines display as plain text without any indication of which filters matched.

This design adds:
1. Per-group color coding for filter chips
2. Visual selection indicators when navigating filters/groups
3. In-log highlighting of matched substrings

## Goals / Non-Goals

**Goals:**
- Each filter group assigned a distinct color from a fixed palette
- Selected filter shows clear visual indicator (background highlight)
- Active group context shown when navigating between groups
- Log lines highlight matched substrings in the corresponding group's color
- First match wins for overlapping highlights

**Non-Goals:**
- User-configurable colors (future consideration)
- Regex-based highlighting (uses existing substring matching)
- Highlighting in the raw log view (only formatted view)

## Decisions

### D1: Color Palette per Group Index

**Decision:** Fixed color palette assigned by group index, cycling through 6 distinct colors.

```
Group 0 → Cyan
Group 1 → Magenta  
Group 2 → Yellow
Group 3 → Green
Group 4 → LightBlue
Group 5 → LightRed
...cycle back to Cyan
```

**Rationale:** 
- Predictable and consistent across sessions
- No configuration needed
- Sufficient distinctiveness for typical usage (3-4 groups)

**Alternatives considered:**
- Hash-based: Would be inconsistent with group ordering
- User-configurable: Adds complexity, defer to future

### D2: Selection Indicator Style

**Decision:** Dark background highlight (Color::DarkGray) + Bold for selected filter.

**Rationale:**
- High contrast against any group color
- Doesn't change the text color identity
- Works well in both light and dark terminals

### D3: Group Context Highlighting

**Decision:** When navigating with h/l, show subtle background tint on all filters in the active group.

**Rationale:**
- Helps user understand which group they're operating in
- Subtle enough not to distract from actual selection

### D4: Match Highlighting in Logs

**Decision:** For each visible log line, find all filter matches and apply group color to matched substrings.

**Rules:**
- First match wins (no overlapping highlights)
- Case-insensitive matching (consistent with filter behavior)
- Only highlight in formatted log view, not raw view

**Implementation approach:**
```rust
fn highlight_matches(line: &str, filters: &[(String, Color)]) -> Vec<Span> {
    // For each position in line, find first matching filter
    // Build spans with appropriate colors
}
```

### D5: Color Utility Module

**Decision:** Add a `colors` module in `src/ui/` with:
- `GROUP_PALETTE: [Color; 6]` constant
- `fn group_color(index: usize) -> Color` function

## Risks / Trade-offs

**Performance with many filters:**
- Risk: O(filters × line_length) per visible line
- Mitigation: Only process visible lines (~50), modern CPUs handle this easily

**Terminal color support:**
- Risk: Some terminals may not render all palette colors correctly
- Mitigation: Use standard ANSI colors that have broad support

**Visual clutter:**
- Risk: Too many highlights could reduce readability
- Mitigation: Colors are distinct but not overly bright; first-match-wins prevents double-highlighting

## Open Questions

None - design is complete for implementation.
