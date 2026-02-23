## Context

The qlog TUI application uses a command mode (triggered by `:`) for operations like filtering, writing files, and quitting. Users can now use Tab completion to cycle through command names. Command parsing and completion logic is extracted to a dedicated `src/command.rs` module for better separation of concerns.

## Goals / Non-Goals

**Goals:**
- Tab key cycles through matching command names
- Minimal state addition to `App` struct
- Preserve existing command behavior exactly
- Commands isolated in separate module for testability

**Non-Goals:**
- Argument completion (file paths, filter patterns)
- Fuzzy matching (only prefix matching)
- Completion menu/popup UI

## Decisions

### D1: Cycling completion over first-match

**Decision:** Tab cycles through all matches alphabetically.

**Alternatives considered:**
- First-match: Completes to alphabetically first match. Surprising when user wants `filter-out` but gets `filter-clear`.
- Longest prefix: Stops at ambiguity. Less helpful - user still types.
- Menu popup: Requires new UI component, more complex.

**Rationale:** Cycling is familiar from bash/zsh, simple to implement, and helps users discover available commands.

### D2: Command registry excludes shortcuts

**Decision:** Store full commands only in `const COMMANDS: &[&str]`. Shortcuts (`q`, `w`) work but don't appear in autocomplete.

```rust
const COMMANDS: &[&str] = &[
    "filter",
    "filter-clear",
    "filter-out",
    "list-filters",
    "quit",
    "write",
];
```

**Rationale:** Shortcuts are for fast typing; autocomplete is for discoverability. Users learning commands via Tab should see full names.

### D3: Command module extraction

**Decision:** Extract command logic to `src/command.rs` with `CommandEffect` enum pattern.

```rust
// src/command.rs
pub enum CommandEffect {
    Quit,
    AddFilter { kind: FilterKind, pattern: String },
    ClearFilters,
    WriteFilteredLogs { filename: String },
    ListFilters,
}

pub struct CommandResult {
    pub effect: Option<CommandEffect>,
    pub status: String,
}

pub fn parse(input: &str) -> CommandResult;
pub fn complete(prefix: &str, index: usize) -> Option<(String, usize)>;
```

**Rationale:** 
- Separates parsing from execution
- App.rs orchestrates effects (knows about `update_filtered_logs()`)
- Commands are independently testable
- Reduces app.rs from ~1300 to ~1136 lines

### D4: Completion state tracking

**Decision:** Add `completion_index: Option<usize>` and `completion_prefix: String` to App struct.

- `completion_index: None` = no active completion session
- `completion_index: Some(n)` = currently at nth match
- `completion_prefix` = original prefix when cycling started
- Both reset on any non-Tab input

**Rationale:** Tracking the original prefix enables consistent cycling even as `input_buffer` changes.

### D5: Completion only on command portion

**Decision:** Only complete text before the first space.

**Rationale:** Argument completion would require context-aware logic (file paths, previous filters). Out of scope.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| No visual indication of cycling | Status could show "1 of 3 matches" in future; acceptable for MVP |
| Case sensitivity | Match case-insensitively for completion |
| `command.rs` needs `FilterKind` from model | Reuse existing type; acceptable coupling |
