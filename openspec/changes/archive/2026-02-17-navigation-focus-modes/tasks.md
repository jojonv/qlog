## 1. Mode Switching

- [x] 1.1 Add `t` key handler in Content mode to switch to Filter mode
- [x] 1.2 Add `t` key handler in Filter mode to switch to Content mode
- [x] 1.3 Add `Esc` key handler in Filter mode to switch to Content mode
- [x] 1.4 Ensure app starts in Content mode (Mode::Normal) on launch

## 2. Content Mode Navigation

- [x] 2.1 Move filter navigation keys (j/k) from Normal mode to Filter mode only
- [x] 2.2 Move group switching keys (h/l) from Normal mode to Filter mode only
- [x] 2.3 Implement j key in Content mode to scroll down (selected_line += 1)
- [x] 2.4 Implement k key in Content mode to scroll up (selected_line -= 1)
- [x] 2.5 Implement l key in Content mode to scroll right (horizontal_scroll += amount)
- [x] 2.6 Implement h key in Content mode to scroll left (horizontal_scroll -= amount)
- [x] 2.7 Ensure scroll_offset updates when selected_line moves out of view

## 3. Filter Mode Refactoring

- [x] 3.1 Move filter selection (j/k) to Filter mode handler
- [x] 3.2 Move group switching (h/l) to Filter mode handler
- [x] 3.3 Move filter add (f) to Filter mode handler
- [x] 3.4 Move new group + filter (F) to Filter mode handler
- [x] 3.5 Move filter delete (d) to Filter mode handler
- [x] 3.6 Move filter toggle (Space) to Filter mode handler

## 4. Visual Feedback

- [x] 4.1 Add visual highlight for selected_line in content view
- [x] 4.2 Update status bar to show current mode (Content/Filter)
- [x] 4.3 Update status bar help text to show mode-specific keybindings

## 5. Bounds Checking

- [x] 5.1 Clamp selected_line to valid range [0, total_lines-1]
- [x] 5.2 Clamp horizontal_scroll to valid range [0, max]
- [x] 5.3 Clamp selected_filter to valid range within group
- [x] 5.4 Clamp selected_group to valid range
