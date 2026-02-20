## 1. Dependency Setup

- [x] 1.1 Add `arboard = "3"` to Cargo.toml dependencies

## 2. Selection Domain Model

- [x] 2.1 Create `src/model/selection.rs` with `Direction` enum (Up, Down)
- [x] 2.2 Create `Selection` struct with anchor and direction fields
- [x] 2.3 Implement `Selection::new()` constructor
- [x] 2.4 Implement `Selection::is_active()` method
- [x] 2.5 Implement `Selection::start(cursor: usize)` method
- [x] 2.6 Implement `Selection::extend(cursor, direction)` method
- [x] 2.7 Implement `Selection::clear()` method
- [x] 2.8 Implement `Selection::contains(idx, cursor)` method
- [x] 2.9 Implement `Selection::range(cursor)` method
- [x] 2.10 Export selection module from `src/model/mod.rs`

## 3. Clipboard Abstraction

- [x] 3.1 Create `src/clipboard.rs` with `ClipboardError` enum
- [x] 3.2 Create `Clipboard` struct wrapping arboard::Clipboard
- [x] 3.3 Implement `Clipboard::new()` constructor with error handling
- [x] 3.4 Implement `Clipboard::copy(text: &str)` method
- [x] 3.5 Export clipboard module from `src/lib.rs`

## 4. App Integration - Selection

- [x] 4.1 Add `selection: Selection` field to `App` struct
- [x] 4.2 Initialize selection in `App::new()`
- [x] 4.3 Implement `x` key handler for starting/extending selection
- [x] 4.4 Modify `j` key handler to extend selection when active
- [x] 4.5 Modify `k` key handler to extend selection when active
- [x] 4.6 Implement `Escape` key handler to clear selection
- [x] 4.7 Add method to clear selection when filters change

## 5. App Integration - Clipboard

- [x] 5.1 Add `clipboard: Option<Clipboard>` field to `App` struct
- [x] 5.2 Initialize clipboard in `App::new()` with error handling
- [x] 5.3 Implement `y` key handler with selection check
- [x] 5.4 Implement clipboard copy logic with raw line retrieval
- [x] 5.5 Add status message on successful copy ("Copied N lines")
- [x] 5.6 Add error message on clipboard unavailable

## 6. UI Rendering

- [x] 6.1 Modify `is_in_selection()` check to use `app.selection.contains()`
- [x] 6.2 Apply DarkGray background to all lines in selection range
- [x] 6.3 Ensure selection highlighting updates on cursor move

## 7. Testing & Validation

- [x] 7.1 Verify selection model unit tests pass
- [x] 7.2 Test single line selection with `x`
- [x] 7.3 Test multi-line selection with `j`/`k`
- [x] 7.4 Test selection clearing with Escape
- [x] 7.5 Test copy functionality with `y`
- [x] 7.6 Test filter change clears selection
- [x] 7.7 Test error handling on clipboard unavailable
- [x] 7.8 Run clippy for code quality
- [x] 7.9 Verify build compiles without warnings
