## 1. Dependency Setup

- [ ] 1.1 Add `arboard = "3"` to Cargo.toml dependencies

## 2. Selection Domain Model

- [ ] 2.1 Create `src/model/selection.rs` with `Direction` enum (Up, Down)
- [ ] 2.2 Create `Selection` struct with anchor and direction fields
- [ ] 2.3 Implement `Selection::new()` constructor
- [ ] 2.4 Implement `Selection::is_active()` method
- [ ] 2.5 Implement `Selection::start(cursor: usize)` method
- [ ] 2.6 Implement `Selection::extend(cursor, direction)` method
- [ ] 2.7 Implement `Selection::clear()` method
- [ ] 2.8 Implement `Selection::contains(idx, cursor)` method
- [ ] 2.9 Implement `Selection::range(cursor)` method
- [ ] 2.10 Export selection module from `src/model/mod.rs`

## 3. Clipboard Abstraction

- [ ] 3.1 Create `src/clipboard.rs` with `ClipboardError` enum
- [ ] 3.2 Create `Clipboard` struct wrapping arboard::Clipboard
- [ ] 3.3 Implement `Clipboard::new()` constructor with error handling
- [ ] 3.4 Implement `Clipboard::copy(text: &str)` method
- [ ] 3.5 Export clipboard module from `src/lib.rs`

## 4. App Integration - Selection

- [ ] 4.1 Add `selection: Selection` field to `App` struct
- [ ] 4.2 Initialize selection in `App::new()`
- [ ] 4.3 Implement `x` key handler for starting/extending selection
- [ ] 4.4 Modify `j` key handler to extend selection when active
- [ ] 4.5 Modify `k` key handler to extend selection when active
- [ ] 4.6 Implement `Escape` key handler to clear selection
- [ ] 4.7 Add method to clear selection when filters change

## 5. App Integration - Clipboard

- [ ] 5.1 Add `clipboard: Option<Clipboard>` field to `App` struct
- [ ] 5.2 Initialize clipboard in `App::new()` with error handling
- [ ] 5.3 Implement `y` key handler with selection check
- [ ] 5.4 Implement clipboard copy logic with raw line retrieval
- [ ] 5.5 Add status message on successful copy ("Copied N lines")
- [ ] 5.6 Add error message on clipboard unavailable

## 6. UI Rendering

- [ ] 6.1 Modify `is_in_selection()` check to use `app.selection.contains()`
- [ ] 6.2 Apply DarkGray background to all lines in selection range
- [ ] 6.3 Ensure selection highlighting updates on cursor move

## 7. Testing & Validation

- [ ] 7.1 Verify selection model unit tests pass
- [ ] 7.2 Test single line selection with `x`
- [ ] 7.3 Test multi-line selection with `j`/`k`
- [ ] 7.4 Test selection clearing with Escape
- [ ] 7.5 Test copy functionality with `y`
- [ ] 7.6 Test filter change clears selection
- [ ] 7.7 Test error handling on clipboard unavailable
- [ ] 7.8 Run clippy for code quality
- [ ] 7.9 Verify build compiles without warnings
