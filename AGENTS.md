# AGENTS.md - Coding Guidelines for qlog

## Project Overview

TUI application for viewing and filtering large log files with real-time updates.
Built in Rust using tokio (async runtime), ratatui (UI), crossterm (terminal), regex/memmap2 (file handling).

## Build Commands

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Binary location after build
target/release/qlog
```

## Test Commands

```bash
# Run all tests
cargo test

# Run a specific test by name
cargo test test_name

# Run tests in a specific file
cargo test --test filter_tests

# Run tests with output visible
cargo test -- --nocapture

# Run only tests matching pattern
cargo test filter::

# Run tests in release mode (slower compile, faster runtime)
cargo test --release
```

## Lint/Format Commands

```bash
# Format code
cargo fmt

# Check formatting without modifying
cargo fmt -- --check

# Run clippy (linter)
cargo clippy

# Run clippy with all features and warnings as errors
cargo clippy --all-features -- -D warnings

# Check for errors without building
cargo check
```

## Code Style Guidelines

### Naming Conventions

- **Types/Structs/Enums**: PascalCase (`LogEntry`, `FilterSet`)
- **Functions/Variables**: snake_case (`load_file`, `buffer_size`)
- **Constants**: SCREAMING_SNAKE_CASE (`DEFAULT_BUFFER_SIZE`)
- **Modules**: snake_case (`log_storage`, `filter`)

### Imports

```rust
// Standard library first
use std::collections::HashMap;
use std::sync::Arc;

// External crates second
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use ratatui::widgets::Paragraph;

// Internal modules third
use crate::model::{LogEntry, Filter};
```

### Types

- Use `Result<T, E>` for fallible operations
- Use `Option<T>` for nullable values
- Use `Arc<T>` for shared ownership across threads
- Use `Box<dyn Error>` for generic error propagation in main
- Use `#[derive(Debug, Clone)]` for most structs

### Documentation

```rust
/// Brief description of what this does.
///
/// Longer explanation with details.
/// Example usage:
/// ```
/// let filter = Filter::new("pattern");
/// ```
pub struct Filter {
    // ...
}
```

### Error Handling

```rust
// Use Result propagation with ?
let file = File::open(path)?;

// Pattern match when handling errors
match result {
    Ok(val) => val,
    Err(e) => eprintln!("Error: {}", e),
}

// For async: use tokio::sync::mpsc for channels
```

### Performance Patterns

- Use `thread_local!` for per-thread buffers
- Use `memmap2` for memory-mapped file access
- Use `rayon` for parallel processing
- Use `RefCell<T>` for interior mutability in single-threaded contexts

### Testing

```rust
// Inline tests (in src files)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_matching() {
        let filter = Filter::new("ERROR");
        assert!(filter.matches("ERROR: something happened"));
    }
}

// Integration tests (in tests/ directory)
// tests/filter_tests.rs uses standard patterns:
// use qlog::model::{Filter, FilterSet};
```

## Project-Specific Conventions

### Environment Variables

Configuration uses `COMO_` prefix:
- `COMO_CONFIG_DIR`
- `COMO_MAX_OPEN_DIRS`

### Module Structure

```rust
// In mod.rs
pub mod submodule;
pub use submodule::{PublicType, public_function};
```

### Constructors

Use `impl Into<String>` for flexibility:
```rust
impl Filter {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
        }
    }
}
```

### Async Patterns

- Use tokio runtime
- Prefer `tokio::spawn` for concurrent tasks
- Use `mpsc` channels for communication
- Use `RwLock` for shared async state

## Before Committing

Always run:
```bash
cargo fmt
cargo clippy
cargo test
```

Fix all warnings and ensure tests pass.
