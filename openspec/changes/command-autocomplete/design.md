## Context

The qlog TUI application uses a command mode (triggered by `:`) for operations like filtering, writing files, and quitting. Currently, users must type complete command names without assistance. This is a self-contained feature within `src/app.rs` - no cross-module changes needed.

Current command handling (app.rs:329-347):
- `KeyCode::Char(c)` appends to `input_buffer`
- `KeyCode::Enter` calls `execute_command()` which parses and dispatches
- Commands are matched via string comparison in a large `match` block

## Goals / Non-Goals

**Goals:**
- Tab key cycles through matching command names
- Minimal state addition to `App` struct
- Preserve existing command behavior exactly

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

### D2: Command registry as const slice

**Decision:** Store commands in `const COMMANDS: &[&str]` sorted alphabetically.

```rust
const COMMANDS: &[&str] = &[
    "filter",
    "filter-clear", 
    "filter-out",
    "list-filters",
    "q",
    "quit",
    "w",
    "write",
];
```

**Rationale:** Zero runtime allocation, easy to maintain, naturally sorted for consistent cycling.

### D3: Completion state tracking

**Decision:** Add `completion_index: Option<usize>` to track cycle position.

```rust
// In App struct
completion_index: Option<usize>,
```

- `None` = no active completion session
- `Some(n)` = currently at nth match in filtered list
- Reset to `None` on any non-Tab input

**Rationale:** Minimal state, easily reset, clear semantics.

### D4: Completion only on command portion

**Decision:** Only complete text before the first space.

**Rationale:** Argument completion would require context-aware logic (file paths, previous filters). Out of scope.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Aliases (`w`/`write`, `q`/`quit`) appear as separate completions | Acceptable - users may prefer short forms |
| No visual indication of cycling | Status could show "1 of 3 matches" in future; acceptable for MVP |
| Case sensitivity | Commands are lowercase; input is as-typed. Match case-insensitively for completion. |
