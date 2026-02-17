## Capability

Export filtered logs to a file via command mode.

## Interface

### Commands

| Command | Description |
|---------|-------------|
| `:w` | Save filtered logs to default filename |
| `:w <filename>` | Save filtered logs to specified file |
| `:write` | Alias for `:w` |
| `:write <filename>` | Alias for `:w <filename>` |

### Default Filename

When no filename provided: `filtered-logs-YYYYMMDD-HHMMSS.log`

### Filename Handling

- Unquoted: `:w output.log` → `output.log`
- Quoted: `:w "my file.log"` → `my file.log`
- Path supported: `:w /tmp/logs/filtered.log`

## Behavior

### What Gets Saved

All entries currently in `filtered_logs` - the result of all active filters (text filter, date range, etc.).

### Output Format

Raw text - one line per log entry, exactly as originally parsed (`entry.raw`).

### File Handling

- Create new file if doesn't exist
- Overwrite (truncate) if exists
- Fail if parent directory doesn't exist
- Fail if permission denied

### Status Feedback

On success: Display line count and filename in status bar
On error: Display error message in status bar

## Constraints

- Writes synchronously (blocks UI briefly for large exports)
- No progress indicator for MVP
- No append mode for MVP
- No confirmation before overwrite

## Examples

```
:w                              → filtered-logs-20250217-143052.log
:w output.log                   → output.log
:w "/tmp/my logs/export.log"    → /tmp/my logs/export.log
:write data/export.txt          → data/export.txt
```
