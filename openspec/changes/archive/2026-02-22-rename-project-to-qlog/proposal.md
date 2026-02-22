## Why

The current project name "como-log-viewer" is verbose and doesn't clearly communicate the tool's purpose. Renaming to "qlog" (quick log) provides a shorter, more memorable name that better reflects the tool's fast, efficient log viewing capabilities.

## What Changes

- **BREAKING**: Rename Cargo package from "como-log-viewer" to "qlog"
- **BREAKING**: Rename Rust crate from "como_log_viewer" to "qlog" (affects all `use` statements)
- **BREAKING**: Binary name changes from "como-log-viewer" to "qlog"
- Update all documentation (README.md, AGENTS.md) to reflect new name
- Update historical OpenSpec archive documents for consistency
- Clean build artifacts and regenerate Cargo.lock

## Capabilities

### New Capabilities
<!-- No new capabilities - this is a rename only -->
None

### Modified Capabilities
<!-- No spec-level behavior changes -->
None

## Impact

- **Binary**: `como-log-viewer` → `qlog`
- **Rust crate**: `como_log_viewer` → `qlog`
- **Cargo.toml**: Package name change
- **Source files**: Update `use` statements in main.rs, examples/, tests/
- **Documentation**: README.md, AGENTS.md updates
- **Archive**: 3 OpenSpec historical documents updated
- **Build**: Requires `cargo clean` and rebuild
