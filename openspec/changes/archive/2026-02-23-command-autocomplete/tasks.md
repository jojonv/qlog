## 1. Command Registry

- [x] 1.1 Add `const COMMANDS: &[&str]` to app.rs with all commands sorted alphabetically (filter, filter-clear, filter-out, list-filters, q, quit, w, write)

## 2. State Tracking

- [x] 2.1 Add `completion_index: Option<usize>` field to App struct
- [x] 2.2 Add `completion_prefix: String` field to App struct for tracking original prefix
- [x] 2.3 Initialize both fields in `App::new()`

## 3. Completion Logic

- [x] 3.1 Implement `get_completion_matches(&self, prefix: &str)` method that filters COMMANDS by prefix (case-insensitive)
- [x] 3.2 Implement `apply_completion(&mut self)` method that cycles through matches and updates input_buffer

## 4. Key Handling

- [x] 4.1 Add `KeyCode::Tab` case in `handle_command_key()` that calls `apply_completion()`
- [x] 4.2 Reset `completion_index` and `completion_prefix` on `KeyCode::Char(c)` input
- [x] 4.3 Reset `completion_index` and `completion_prefix` on `KeyCode::Backspace` input

## 5. Testing

- [x] 5.1 Add unit test for `get_completion_matches()` with various prefixes
- [x] 5.2 Add unit test for cycling behavior (wrap-around)
- [x] 5.3 Add unit test for reset on non-Tab input
- [x] 5.4 Add unit test for argument preservation
- [x] 5.5 Run `cargo test` - all 89 tests pass

## 6. Finalization

- [x] 6.1 Run `cargo fmt` to format code
- [x] 6.2 Run `cargo clippy` - no new warnings in app.rs (pre-existing warnings in other modules)
