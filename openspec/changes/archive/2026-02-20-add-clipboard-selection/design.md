# Design: Clipboard Integration with Helix-Style Selection

## Overview

This design implements a clipboard integration feature with Helix-style selection for the COMO log viewer. It follows SOLID principles with clear separation of concerns.

## Architecture

### Directory Structure

```
src/
├── model/
│   ├── selection.rs          # NEW: Selection domain model
│   └── mod.rs                # UPDATED: Export selection module
├── clipboard.rs              # NEW: Clipboard abstraction wrapper
├── lib.rs                    # UPDATED: Export clipboard module
└── app.rs                    # UPDATED: Wire up selection and clipboard
```

### Component Design

```
┌─────────────────────────────────────────────────────────────────┐
│                         COMPONENT DIAGRAM                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────┐                                             │
│  │   App           │                                             │
│  │  ┌─────────────┐│                                             │
│  │  │ selection:  ││                                             │
│  │  │ Selection   ││ ◄── composes ──┐                         │
│  │  └─────────────┘││                │                         │
│  │  ┌─────────────┐││                │                         │
│  │  │ clipboard:  │││                │                         │
│  │  │ Option<     │││                │                         │
│  │  │  Clipboard>│││                │                         │
│  │  └─────────────┘││                │                         │
│  │  └─────────────┘│                │                         │
│  └────────┬────────┘                │                         │
│             │                         │                         │
│             │ delegates               │                         │
│             ▼                         │                         │
│  ┌─────────────────┐                  │                         │
│  │   Selection     │                  │                         │
│  │  ┌─────────────┐│                  │                         │
│  │  │ anchor:     ││                  │                         │
│  │  │ Option<     ││                  │                         │
│  │  │ usize>      ││                  │                         │
│  │  └─────────────┘│                  │                         │
│  │  ┌─────────────┐│                  │                         │
│  │  │ direction:  ││                  │                         │
│  │  │ Option<     ││                  │                         │
│  │  │ Direction>  ││                  │                         │
│  │  └─────────────┘│                  │                         │
│  │  ├─ new()       │                  │                         │
│  │  ├─ is_active() │                  │                         │
│  │  ├─ start()     │                  │                         │
│  │  ├─ extend()    │                  │                         │
│  │  ├─ clear()     │                  │                         │
│  │  ├─ contains()   │                  │                         │
│  │  └─ range()     │                  │                         │
│  └─────────────────┘                  │                         │
│                                         │                         │
│             ┌───────────────────────────┘                         │
│             │                                                      │
│             ▼                                                      │
│  ┌─────────────────┐                                               │
│  │   Clipboard     │                                               │
│  │  ┌─────────────┐│                                               │
│  │  │ inner:       ││                                               │
│  │  │ arboard::    ││                                               │
│  │  │ Clipboard    ││                                               │
│  │  └─────────────┘│                                               │
│  │  ├─ new()       ││                                               │
│  │  └─ copy()      ││                                               │
│  └─────────────────┘                                               │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Data Models

### Selection (src/model/selection.rs)

```rust
/// Direction of selection extension
pub enum Direction {
    Up,
    Down,
}

/// Tracks selection state for Helix-style selection
pub struct Selection {
    /// Anchor point - start of selection (None = no selection)
    anchor: Option<usize>,
    /// Direction of last extension for repeat-x behavior
    direction: Option<Direction>,
}

impl Selection {
    /// Create new empty selection
    pub fn new() -> Self;
    
    /// Check if selection is active (anchor is set)
    pub fn is_active(&self) -> bool;
    
    /// Start selection at cursor position
    pub fn start(&mut self, cursor: usize);
    
    /// Extend selection toward cursor, recording direction
    pub fn extend(&mut self, cursor: usize, direction: Direction);
    
    /// Clear selection (return to single cursor state)
    pub fn clear(&mut self);
    
    /// Check if index is within selection range
    pub fn contains(&self, idx: usize, cursor: usize) -> bool;
    
    /// Get selection range (min, max) or None
    pub fn range(&self, cursor: usize) -> Option<(usize, usize)>;
}
```

### Clipboard (src/clipboard.rs)

```rust
use arboard::Clipboard as ArboardClipboard;

/// Wrapper around arboard clipboard with error handling
pub struct Clipboard {
    inner: ArboardClipboard,
}

impl Clipboard {
    /// Initialize clipboard (may fail on headless systems)
    pub fn new() -> Result<Self, ClipboardError>;
    
    /// Copy text to system clipboard
    pub fn copy(&mut self, text: &str) -> Result<(), ClipboardError>;
}

#[derive(Debug)]
pub enum ClipboardError {
    InitFailed(String),
    CopyFailed(String),
}
```

## Key Behavior Logic

### Key Handling Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                     KEY HANDLING LOGIC                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  NORMAL MODE                                                     │
│  ────────────                                                    │
│                                                                  │
│  ┌──────────┐                                                   │
│  │ Key 'x'  │                                                   │
│  └────┬─────┘                                                   │
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ IF selection.is_active():                               │   │
│  │   THEN selection.extend(cursor, direction)             │   │
│  │   ELSE selection.start(cursor)                         │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────┐                                                   │
│  │ Key 'j'  │                                                   │
│  └────┬─────┘                                                   │
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ cursor = min(cursor + 1, filtered_count - 1)              │   │
│  │ IF selection.is_active():                                 │   │
│  │   THEN selection.extend(cursor, Direction::Down)       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────┐                                                   │
│  │ Key 'k'  │                                                   │
│  └────┬─────┘                                                   │
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ cursor = max(cursor.saturating_sub(1), 0)               │   │
│  │ IF selection.is_active():                                 │   │
│  │   THEN selection.extend(cursor, Direction::Up)         │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────┐                                                   │
│  │ Key 'y'  │                                                   │
│  └────┬─────┘                                                   │
│       │                                                         │
│       ▼                                                         │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ IF NOT selection.is_active():                             │   │
│  │   THEN do nothing                                         │   │
│  │ ELIF clipboard.is_none():                                 │   │
│  │   THEN show_error("Clipboard unavailable")               │   │
│  │ ELSE:                                                     │   │
│  │   let (start, end) = selection.range(cursor).unwrap()   │   │
│  │   let lines = (start..=end).map(|i| get_raw_line(i))     │   │
│  │   let text = lines.join("\n")                            │   │
│  │   clipboard.copy(text)                                   │   │
│  │   show_status("Copied {count} lines to clipboard")       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────┐                                               │
│  │ Key 'Escape' │                                               │
│  └──────┬───────┘                                               │
│         │                                                       │
│         ▼                                                       │
│  ┌────────────────────────┐                                     │
│  │ selection.clear()       │                                     │
│  └────────────────────────┘                                     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Selection Clearing Conditions

| Event | Action | Rationale |
|-------|--------|-----------|
| Filter changes | Clear selection | Indices become invalid |
| Search jump (n/N) | Keep selection | User might want to copy around matches |
| Mode switch | Keep selection | Useful for command mode operations |
| Escape key | Clear selection | User intent to reset |

## Rendering

### Selection Visual Indication

```rust
// In src/ui/mod.rs
fn is_in_selection(idx: usize, app: &App) -> bool {
    app.selection.contains(idx, app.selected_line)
}

// When rendering each line:
let base_bg = if is_in_selection(idx, app) {
    Some(Color::DarkGray)
} else {
    None
};
```

## Error Handling

```
┌─────────────────────────────────────────────────────────────────┐
│                    ERROR HANDLING STRATEGY                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Clipboard Initialization:                                       │
│  ─────────────────────────                                       │
│  • Try to create Clipboard in App::new()                        │
│  • On failure: clipboard = None                                │
│  • App continues normally                                        │
│                                                                  │
│  Copy Error:                                                     │
│  ────────────                                                    │
│  • Show error message in status bar                             │
│  • "Clipboard unavailable - install display server"             │
│  • Selection remains intact                                      │
│                                                                  │
│  No Selection:                                                   │
│  ──────────────                                                  │
│  • Pressing 'y' with no selection: silent no-op                 │
│  • User must explicitly select first (x)                        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## SOLID Principles Applied

| Principle | Application |
|-----------|-------------|
| **S**ingle Responsibility | Selection tracks selection state only; Clipboard handles system clipboard only; App orchestrates |
| **O**pen/Closed | New functionality added via new modules; existing files unchanged |
| **L**iskov Substitution | N/A (no inheritance hierarchy) |
| **I**nterface Segregation | Selection exposes minimal API; caller uses only what needed |
| **D**ependency Inversion | App owns Selection/Clipboard instances; no global state |

## Integration Points

### Modified Files

| File | Changes |
|------|---------|
| `Cargo.toml` | Add `arboard = "3"` dependency |
| `src/model/mod.rs` | `pub mod selection; pub use selection::*;` |
| `src/lib.rs` | `pub mod clipboard; pub use clipboard::*;` |
| `src/app.rs` | Add fields, wire up key handlers, integrate selection clearing |
| `src/ui/mod.rs` | Update selection check to use `app.selection.contains()` |

### New Files

| File | Purpose |
|------|---------|
| `src/model/selection.rs` | Selection domain model |
| `src/clipboard.rs` | Clipboard abstraction wrapper |
