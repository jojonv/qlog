## Why

Two issues:
1. **Wrap mode limitation**: Content is always wrapped with no option to view full-width lines. This hides information when lines are long and the existing horizontal scroll (`h/l`) has no meaningful effect.
2. **Scroll bug**: Viewport scrolling uses a hardcoded height of 20 lines, causing the view to not scroll correctly when the terminal is larger or when moving through content.

## What Changes

- Add wrap/no-wrap toggle (`w` key in Normal mode) to switch between wrapped and non-wrapped content display
- In no-wrap mode, display horizontal scrollbar when content exceeds viewport width
- Always show vertical scrollbar (when content exceeds viewport height)
- Fix hardcoded viewport height bug by tracking actual viewport height dynamically
- Update status bar to show current wrap state (`[WRAP]`/`[nowrap]`)

## Capabilities

### New Capabilities

- `content-wrap-mode`: Toggle between wrapped and non-wrapped content display with appropriate scrollbars

### Modified Capabilities

None. This is a new capability that extends the existing content viewing without changing requirements of `focus-modes` or `filter-groups`.

## Impact

- `src/app.rs`: Add `wrap_mode: bool` and `viewport_height: usize` fields; add `w` key handler; fix `clamp_scroll()`
- `src/ui/mod.rs`: Add scrollbar rendering; conditional wrap; track viewport height
