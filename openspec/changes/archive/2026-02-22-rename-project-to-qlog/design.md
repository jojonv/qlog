## Context

The project currently uses the name "como-log-viewer" throughout:
- Cargo package name: `como-log-viewer`
- Rust crate name: `como_log_viewer` 
- Binary name: `como-log-viewer`
- Documentation references in README.md, AGENTS.md
- Historical OpenSpec archive documents (3 files)

The rename to "qlog" requires updating all these references while maintaining existing functionality.

## Goals / Non-Goals

**Goals:**
- Rename package to "qlog" in Cargo.toml
- Rename crate to "qlog" (affects Rust code)
- Rename binary from "como-log-viewer" to "qlog"
- Update all documentation to reflect new name
- Update historical OpenSpec documents for consistency
- Regenerate Cargo.lock via clean build

**Non-Goals:**
- No functional changes to the application
- No API changes (beyond crate name in imports)
- No new features
- No behavior modifications

## Decisions

**Decision: Use "qlog" for both binary and crate name**
- Rationale: Simple, clean, follows Rust conventions. The short name is appropriate for a CLI tool.
- Alternative: Keep "como-log-viewer" as crate name, only rename binary. Rejected: inconsistent naming creates confusion.

**Decision: Update historical OpenSpec documents**
- Rationale: User requested consistency across all references.
- Alternative: Leave archives as-is. Rejected: Creates inconsistency in project history.

**Decision: Full `cargo clean` before rebuild**
- Rationale: Ensures no stale artifacts reference old name. Clean state prevents subtle issues.

## Risks / Trade-offs

**Risk**: Breaking change for anyone building from source → Binary name changes, imports change
- **Mitigation**: This is acceptable per user confirmation. The tool is personal/team use, not a public library.

**Risk**: Missing references → Some files might still contain old name
- **Mitigation**: Comprehensive search for both "como-log-viewer" (kebab) and "como_log_viewer" (snake) patterns

**Risk**: Cargo.lock conflicts → Lockfile references old package name
- **Mitigation**: `cargo clean` removes old artifacts; `cargo build` regenerates lockfile

## Migration Plan

1. Update Cargo.toml (package name)
2. Update Rust source files (use statements)
3. Update documentation (README.md, AGENTS.md)
4. Update OpenSpec archives (3 historical documents)
5. `cargo clean`
6. `cargo build --release`
7. Verify binary exists at `target/release/qlog`

## Open Questions

None. Rename scope is well-defined from codebase analysis.
