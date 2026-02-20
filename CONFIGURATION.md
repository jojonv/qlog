# Log Coloring Configuration

This document describes how to configure log line coloring in Como Log Viewer.

## Configuration File Location

Como Log Viewer looks for configuration files in the following locations (in order):

1. `./.qlog/qlog.toml` - Current directory (takes precedence)
2. `~/.qlog/qlog.toml` - Home directory (fallback)

If no configuration file is found, log lines are displayed without custom coloring.

## Configuration Format

The configuration uses TOML format with a `[colors]` section containing pattern-color mappings:

```toml
[colors]
error = "red"
warn = "yellow"
success = "green"
"*TODO*" = "magenta"
```

## Pattern Matching

Patterns support simple wildcards for flexible matching:

| Pattern | Meaning | Example |
|---------|---------|---------|
| `text` | Contains "text" (case-insensitive) | `error` matches "Error: something failed" |
| `*text` | Ends with "text" | `*error` matches "got ERROR" |
| `text*` | Starts with "text" | `error*` matches "ERROR: failed" |
| `*text*` | Contains "text" | `*error*` matches "some ERROR here" |

All matching is **case-insensitive**.

### First Match Wins

When multiple patterns could match a line, the first matching pattern in the configuration file determines the color:

```toml
[colors]
error = "red"        # Line "error warning" gets red
warn = "yellow"      # Only used if "error" doesn't match
```

## Supported Colors

### Basic Colors
- `red`
- `green`
- `blue`
- `yellow`
- `magenta`
- `cyan`
- `white`
- `black`
- `gray` (or `grey`)

### Extended Colors
- `dark_gray` (or `dark_grey`)
- `light_red`
- `light_green`
- `light_blue`
- `light_yellow`
- `light_magenta`
- `light_cyan`

## UI Behavior

- **Timestamps**: Always displayed in cyan, regardless of line color
- **Selection Highlight**: Dark gray background takes precedence over line color
- **Line Color**: Applied to the log text foreground only

## Examples

### Error/Warning Highlighting

```toml
[colors]
error = "red"
exception = "light_red"
warn = "yellow"
warning = "yellow"
```

### Development Markers

```toml
[colors]
"*TODO*" = "magenta"
"*FIXME*" = "magenta"
"*HACK*" = "yellow"
"*XXX*" = "red"
```

### Log Level Coloring

```toml
[colors]
fatal = "light_red"
error = "red"
warn = "yellow"
info = "green"
debug = "dark_gray"
trace = "dark_gray"
```

### Status-Based Coloring

```toml
[colors]
success = "green"
failed = "red"
completed = "cyan"
pending = "yellow"
```

## Error Handling

If the configuration file contains errors, they are logged to stderr and the application continues without custom coloring. Common errors include:

- Invalid TOML syntax
- Unknown color names
- Missing `[colors]` section
- Empty configuration file
