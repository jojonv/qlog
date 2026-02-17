## 1. App State Updates

- [ ] 1.1 Add `wrap_mode: bool` field to App struct (default: `true`)
- [ ] 1.2 Add `viewport_height: usize` field to App struct (default: `20`)
- [ ] 1.3 Update `clamp_scroll()` to use `self.viewport_height` instead of hardcoded `20`
- [ ] 1.4 Add `w` key handler in `handle_normal_key()` to toggle `wrap_mode`

## 2. UI Scrollbar Implementation

- [ ] 2.1 Add scrollbar imports (`Scrollbar`, `ScrollbarOrientation`, `ScrollbarState`)
- [ ] 2.2 Add helper to calculate max line width from filtered logs
- [ ] 2.3 Implement vertical scrollbar rendering in `draw_main_view()`
- [ ] 2.4 Implement horizontal scrollbar rendering (only when `wrap_mode == false`)
- [ ] 2.5 Update `viewport_height` in App from actual rendered area height

## 3. Conditional Wrap Mode

- [ ] 3.1 Conditionally apply `.wrap()` based on `app.wrap_mode`
- [ ] 3.2 Add `[WRAP]`/`[nowrap]` indicator to status bar
- [ ] 3.3 Update status bar help text to include `w: Wrap` keybinding
