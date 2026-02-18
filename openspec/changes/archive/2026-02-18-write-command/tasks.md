## 1. Command Parsing

- [x] 1.1 Add `parse_command` function to extract command and filename from input buffer
- [x] 1.2 Handle quoted filenames (detect opening/closing quotes)
- [x] 1.3 Generate default timestamped filename when none provided

## 2. File Writing

- [x] 2.1 Add `write_filtered_logs` method to App struct
- [x] 2.2 Iterate filtered_logs and write each entry.raw to file
- [x] 2.3 Return line count on success, error on failure

## 3. Command Integration

- [x] 3.1 Modify `handle_command_key` to parse and execute write on Enter
- [x] 3.2 Call `write_filtered_logs` with parsed filename
- [x] 3.3 Handle `:w` and `:write` commands
- [x] 3.4 Show "Unknown command" for unrecognized commands

## 4. Status Feedback

- [x] 4.1 Add `status_message` field to App struct (Option<String>)
- [x] 4.2 Set success message: "Saved N lines to <filename>"
- [x] 4.3 Set error message on write failure
- [x] 4.4 Display status_message in status bar
- [x] 4.5 Clear status_message on next key press in normal mode

## 5. Testing

- [ ] 5.1 Verify `:w default.log` creates file with filtered content
- [ ] 5.2 Verify `:w` generates timestamped filename
- [ ] 5.3 Verify `:w "file with spaces.log"` handles quoted path
- [ ] 5.4 Verify error handling (permission denied, bad path)
- [ ] 5.5 Verify status messages display correctly
