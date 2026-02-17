## Context

The app has two primary interaction areas: the log content view and the filter bar. The `filter-groups` change added navigation keybindings (j/k/h/l) for filters, but these were placed in `Mode::Normal`, which is also the mode for scrolling logs. This creates a conflict where users cannot scroll content.

Current state:
- `Mode::Normal`: j/k/h/l navigate filters (wrong - should be for content)
- `Mode::Filter`: exists but empty (no keybindings)
- `selected_line` exists but has no keybindings to control it
- `scroll_offset` and `horizontal_scroll` exist but aren't connected to keys

## Goals / Non-Goals

**Goals:**
- Enable log scrolling with j/k in Content mode
- Enable horizontal scrolling with h/l in Content mode
- Move filter navigation to Filter mode
- Provide clear mode switching with `t` and `Esc`
- Highlight current line in content view

**Non-Goals:**
- Changing filter functionality (f/F/d/Space behavior stays the same)
- Adding new filter operations
- Changing FilterInput mode behavior

## Decisions

### D1: Mode-based navigation (modal approach)

**Decision**: Use two modes with distinct j/k/h/l semantics.

```
┌─────────────────────────────────────────────────────────────┐
│   Content Mode (default, starts here)                       │
│   ─────────────────────────────                             │
│   j/k    → scroll logs (selected_line ± 1)                  │
│   h/l    → horizontal scroll                                │
│   g/G    → jump top/bottom                                  │
│   t      → enter Filter mode                                │
│                                                             │
│   Filter Mode                                               │
│   ─────────────────────────────                             │
│   j/k    → select filter within group                       │
│   h/l    → switch between groups                            │
│   f      → add filter to current group                      │
│   F      → create new group + add filter                    │
│   d      → delete selected filter                           │
│   Space  → toggle filter on/off                             │
│   t/Esc  → return to Content mode                           │
└─────────────────────────────────────────────────────────────┘
```

**Rationale**: Matches vim-style modal editing. Users familiar with modal editors will find this intuitive. Alternative considered: Tab to cycle focus (like lazygit), but modal feels more aligned with the existing vim-like keybindings (g, G, d, etc.).

**Alternatives rejected**:
- Tab cycling: Less explicit, harder to see which area is focused
- Leader keys (Space+j/k for filter): Too many keypresses
- Alt modifiers: Terminal compatibility issues

### D2: Mode enum semantics

**Decision**: Keep `Mode::Normal` and `Mode::Filter` names, but:
- `Mode::Normal` → Content mode behavior (scroll)
- `Mode::Filter` → Filter mode behavior (navigate filters)

**Rationale**: Renaming to `Mode::Content` would require more changes. The semantics are clear from context.

### D3: Startup mode

**Decision**: Start in Content mode (`Mode::Normal`).

**Rationale**: Primary use case is viewing logs. Filter mode is secondary.

### D4: Current line highlighting

**Decision**: Add visual highlight to `selected_line` in content view.

**Rationale**: Users need feedback about their position when scrolling. The `selected_line` field exists but isn't visually indicated.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Users confused by modal system | Status bar shows current mode and available keys |
| Muscle memory from old j/k behavior | Document as breaking change in proposal |
| Forgetting which mode they're in | Visual indicator in status bar |

## Open Questions

None - design is complete based on exploration session.
