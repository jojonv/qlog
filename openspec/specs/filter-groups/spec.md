# Filter Groups Capability

## Status

Current State: **Partially Implemented**

## Overview

Filter groups provide the ability to filter displayed log entries based on user-defined criteria. The filter system uses a hierarchical structure where filters are organized into groups, and multiple groups can be combined with AND logic. Multiple filters within the same group use OR logic - meaning a line must match at least one filter in the group.

## Requirements

### FR-1: Filter Commands

The application SHALL support command-based filter management accessible through the Command mode (`:` key).

**FR-1.1: Filter-In Command**  
The application SHALL support the `:filter <pattern>` command to add an include filter. Lines must contain the pattern to be displayed.

**FR-1.2: Filter-Out Command**  
The application SHALL support the `:filter-out <pattern>` command to add an exclude filter. Lines matching the pattern SHALL be hidden.

**FR-1.3: Filter-Clear Command**  
The application SHALL support the `:filter-clear` command to remove all active filters.

**FR-1.4: List-Filters Command**  
The application SHALL support the `:list-filters` command to display all active filters in an interactive list view.

**FR-1.5: Filter Selection**  
In the filter list view (triggered by `:list-filters`), the user SHALL be able to navigate filters using `j`/`k` keys and delete the selected filter using the `d` key.

### FR-2: Filter Matching Logic

**FR-2.1: Include Filter Logic**  
When multiple include filters are active, a log entry MUST match ALL include filters to be displayed (AND logic between includes).

**FR-2.2: Exclude Filter Logic**  
When exclude filters are active, a log entry MUST NOT match ANY exclude filter to be displayed (OR logic between excludes - matching one excludes the entry).

**FR-2.3: Combined Logic**  
The overall filter logic SHALL be: `(matches ALL includes) AND (matches NO excludes)`.

**FR-2.4: Case-Insensitive Matching**  
All text-based filter matching SHALL be case-insensitive.

**FR-2.5: Substring Matching**  
Filter patterns SHALL match substrings within log entries (not requiring full-line matches).

## Dependencies

- Requires: Log Storage and Display System (to filter displayed entries)
- Future Enhancement: Date/Time filtering capabilities
- Future Enhancement: Regex pattern support

## Visual Design

**Status Bar Integration:**  
The filter status SHALL be displayed in the status bar, showing the number of active filters.

**Filter List Popup:**  
The `:list-filters` command SHALL display a centered popup overlay with:
- Title: "Filter List"
- List of active filters showing kind (INCLUDE/EXCLUDE) and pattern
- Current selection indicator
- Help text for navigation keys

## Future Enhancements

- **FR-3.1:** Date/time range filtering with `after:` and `before:` qualifiers
- **FR-3.2:** Regex pattern matching support
- **FR-3.3:** Filter persistence across sessions
- **FR-3.4:** Named filters for quick recall

## Changelog

### 2026-02-21

**Breaking Change:** Reworked filter system from visual group-based to command-based approach.

**Removed:**
- Visual filter mode (`t` key)
- Filter group navigation (h/l keys)
- Filter toggle (space key)
- Filter input mode for visual editing

**Added:**
- Command-based filter management (`:filter`, `:filter-out`, `:filter-clear`, `:list-filters`)
- FilterList mode for interactive filter management
- Simplified filter logic: includes (AND) + excludes (OR NOT)

**Changed:**
- Filter matching now uses case-insensitive substring matching with Boyer-Moore-Horspool algorithm
- Performance optimized with thread-local buffers for case conversion
