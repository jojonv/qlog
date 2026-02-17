## 1. Color Infrastructure

- [ ] 1.1 Create `src/ui/colors.rs` module with GROUP_PALETTE constant (6 colors: Cyan, Magenta, Yellow, Green, LightBlue, LightRed)
- [ ] 1.2 Add `group_color(index: usize) -> Color` function that cycles through palette
- [ ] 1.3 Export colors module from `src/ui/mod.rs`

## 2. Filter Bar Visual Indicators

- [ ] 2.1 Modify `draw_filter_bar()` to apply group color to each filter chip text
- [ ] 2.2 Add dark background (DarkGray) + Bold to selected filter styling
- [ ] 2.3 Add subtle background tint to all filters in active group during navigation
- [ ] 2.4 Update group separator (`│`) to use neutral color (white/gray)

## 3. Log Line Match Highlighting

- [ ] 3.1 Create `highlight_matches()` function that finds all filter matches in a log line
- [ ] 3.2 Implement first-match-wins logic for overlapping matches
- [ ] 3.3 Build `Vec<Span>` with appropriate colors for each matched region
- [ ] 3.4 Integrate highlighting into `draw_main_view()` for formatted log display
- [ ] 3.5 Skip highlighting for disabled filters

## 4. Testing & Verification

- [ ] 4.1 Build and verify no compilation errors
- [ ] 4.2 Test group colors display correctly in filter bar
- [ ] 4.3 Test selection indicator shows on selected filter
- [ ] 4.4 Test group context highlight during h/l navigation
- [ ] 4.5 Test log highlighting with single filter match
- [ ] 4.6 Test log highlighting with multiple filters from same group
- [ ] 4.7 Test log highlighting with overlapping matches (first wins)
- [ ] 4.8 Test that disabled filters don't highlight matches
