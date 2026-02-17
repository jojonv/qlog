## Approach

Parse command input in `handle_command_key` when Enter is pressed. Split input on whitespace, handle quoted filenames, and delegate to a write function. Keep command parsing simple - just enough for `:w` and `:write`.

## Command Flow

```
User presses ':'
    ↓
Mode::Command activated, input_buffer starts capturing
    ↓
User types "w output.log" or "write output.log"
    ↓
User presses Enter
    ↓
handle_command_key():
    1. Parse input_buffer
    2. Extract command ("w" or "write")
    3. Extract filename (rest of input, trimmed)
    4. If no filename: generate default
    5. Call write_filtered_logs(filename)
    6. Set status message (success/error)
    7. Return to Mode::Normal
```

## Default Filename

Format: `filtered-logs-YYYYMMDD-HHMMSS.log`

Example: `filtered-logs-20250217-154230.log`

Generated using `chrono::Local::now()` if chrono is available, otherwise `std::time::SystemTime` with manual formatting.

## Quoted Filenames

Support paths with spaces via double quotes:
- `:w "my logs/output.log"` → filename is `my logs/output.log`
- `:w output.log` → filename is `output.log`

Simple quote parsing: if input starts with `"`, find closing `"` and extract. No escape sequences for MVP.

## File Writing

```rust
fn write_filtered_logs(&self, filename: &str) -> Result<usize, std::io::Error> {
    let mut file = std::fs::File::create(filename)?;
    let mut count = 0;
    for entry in &self.filtered_logs {
        writeln!(file, "{}", entry.raw)?;
        count += 1;
    }
    Ok(count)
}
```

Use `File::create` which creates new or truncates existing.

## Error Handling

Display errors in status bar with red styling:
- Permission denied
- Path doesn't exist (parent directory)
- Disk full

On error: stay in command mode or return to normal? Return to normal with error status.

## Status Feedback

Success: `Saved 1,247 lines to output.log`
Error: `Error: Permission denied`

Status persists until next user action (cleared on any key press in normal mode).

## Code Locations

| Component | Location |
|-----------|----------|
| Command parsing | `src/app.rs::handle_command_key` |
| Write function | `src/app.rs::App::write_filtered_logs` |
| Status display | `src/ui/mod.rs::draw_status_bar` |

## Future Extension Points

The command parsing structure allows easy addition of:
- `:q` / `:quit` - exit application
- `:q!` - force quit
- `:sort <field>` - sort logs
- `:w >> file` - append mode

For now, unknown commands show: `Unknown command: <cmd>`
