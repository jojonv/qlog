## 1. Command Parsing

- [ ] 1.1 Add `parse_command` function to extract command and filename from input buffer
- [ ] 1.2 Handle quoted filenames (detect opening/closing quotes)
- [ ] 1.3 Generate default timestamped filename when none provided

## 2. File Writing

- [ ] 2.1 Add `write_filtered_logs` method to App struct
- [ ] 2.2 Iterate filtered_logs and write each entry.raw to file
- [ ] 2.3 Return line count on success, error on failure

## 3. Command Integration

- [ ] 3.1 Modify `handle_command_key` to parse and execute write on Enter
- [ ] 3.2 Call `write_filtered_logs` with parsed filename
- [ ] 3.3 Handle `:w` and `:write` commands
- [ ] 3.4 Show "Unknown command" for unrecognized commands

## 4. Status Feedback

- [ ] 4.1 Add `status_message` field to App struct (Option<String>)
- [ ] 4.2 Set success message: "Saved N lines to <filename>"
- [ ] 4.3 Set error message on write failure
- [ ] 4.4 Display status_message in status bar
- [ ] 4.5 Clear status_message on next key press in normal mode

## 5. Testing

- [ ] 5.1 Verify `:w default.log` creates file with filtered content
- [ ] 5.2 Verify `:w` generates timestamped filename
- [ ] 5.3 Verify `:w "file with spaces.log"` handles quoted path
- [ ] 5.4 Verify error handling (permission denied, bad path)
- [ ] 5.5 Verify status messages display correctly
