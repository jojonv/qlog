## 1. Command Registry

- [ ] 1.1 Add `const COMMANDS: &[&str]` to app.rs with all commands sorted alphabetically (filter, filter-clear, filter-out, list-filters, q, quit, w, write)

## 2. State Tracking

- [ ] 2.1 Add `completion_index: Option<usize>` field to App struct
- [ ] 2.2 Initialize `completion_index: None` in `App::new()`

## 3. Completion Logic

- [ ] 3.1 Implement `get_completion_matches(&self) -> Vec<&'static str>` method that filters COMMANDS by current input prefix (case-insensitive)
- [ ] 3.2 Implement `apply_completion(&mut self)` method that cycles through matches and updates input_buffer

## 4. Key Handling

- [ ] 4.1 Add `KeyCode::Tab` case in `handle_command_key()` that calls `apply_completion()`
- [ ] 4.2 Reset `completion_index` to `None` on `KeyCode::Char(c)` input
- [ ] 4.3 Reset `completion_index` to `None` on `KeyCode::Backspace` input

## 5. Testing

- [ ] 5.1 Add unit test for `get_completion_matches()` with various prefixes
- [ ] 5.2 Add unit test for cycling behavior (wrap-around)
- [ ] 5.3 Add unit test for reset on non-Tab input
- [ ] 5.4 Run `cargo test` to verify all tests pass

## 6. Finalization

- [ ] 6.1 Run `cargo fmt` to format code
- [ ] 6.2 Run `cargo clippy --all-features -- -D warnings` to check for lint issues
