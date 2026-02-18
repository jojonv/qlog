## Context

The log viewer currently wraps all content lines, hiding truncated information and making the existing horizontal scroll (`h/l` keys) ineffective. Additionally, the scroll logic uses a hardcoded viewport height of 20 lines, causing incorrect scrolling behavior on terminals with different sizes.

## Goals / Non-Goals

**Goals:**
- Toggle between wrapped and non-wrapped content display
- Make horizontal scroll functional in non-wrapped mode
- Show scrollbars when content exceeds viewport
- Fix the hardcoded viewport height bug

**Non-Goals:**
- Column alignment or formatting changes
- Persisting wrap mode preference across sessions
- Changing existing keybindings for navigation

## Decisions

### 1. Wrap toggle key: `w`
**Rationale:** `w` is unused in both Normal and Filter modes. Mnemonic: "wrap".
**Alternatives considered:** `W` (shift), `z` (vim tradition for fold-related)

### 2. Scrollbar visibility
- **Vertical:** Show when `filtered_logs.len() > viewport_height`
- **Horizontal:** Show when `max_line_width > viewport_width` AND `wrap_mode == false`

**Rationale:** Vertical scrollbar provides context for position. Horizontal only meaningful when wrap is off.

### 3. Viewport height tracking
Store `viewport_height: usize` in App, updated each frame in `draw_main_view()`.

**Rationale:** UI knows the actual rendered area height; App needs it for scroll clamping.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Viewport height race condition (frame lag) | Update from UI each frame before clamping |
| Scrollbar takes screen space | Reduce content area by scrollbar width; acceptable tradeoff |
