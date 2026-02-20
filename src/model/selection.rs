/// Direction of selection extension
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

/// Tracks selection state for Helix-style selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    /// Anchor point - start of selection (None = no selection)
    anchor: Option<usize>,
    /// Direction of last extension for repeat-x behavior
    direction: Option<Direction>,
}

impl Selection {
    /// Create new empty selection
    pub fn new() -> Self {
        Self {
            anchor: None,
            direction: None,
        }
    }

    /// Check if selection is active (anchor is set)
    pub fn is_active(&self) -> bool {
        self.anchor.is_some()
    }

    /// Start selection at cursor position
    pub fn start(&mut self, cursor: usize) {
        self.anchor = Some(cursor);
        self.direction = None;
    }

    /// Extend selection toward cursor, recording direction
    pub fn extend(&mut self, _cursor: usize, direction: Direction) {
        self.direction = Some(direction);
        // The anchor stays fixed, and the selection extends from anchor to cursor
        // This is handled in contains() and range() methods
    }

    /// Clear selection (return to single cursor state)
    pub fn clear(&mut self) {
        self.anchor = None;
        self.direction = None;
    }

    /// Check if index is within selection range
    /// Takes the current cursor position to determine the active selection range
    pub fn contains(&self, idx: usize, cursor: usize) -> bool {
        let Some(anchor) = self.anchor else {
            return false;
        };

        let (start, end) = if anchor <= cursor {
            (anchor, cursor)
        } else {
            (cursor, anchor)
        };

        idx >= start && idx <= end
    }

    /// Get selection range (min, max) or None
    /// Takes the current cursor position to determine the active selection range
    pub fn range(&self, cursor: usize) -> Option<(usize, usize)> {
        let anchor = self.anchor?;

        let (start, end) = if anchor <= cursor {
            (anchor, cursor)
        } else {
            (cursor, anchor)
        };

        Some((start, end))
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_selection_is_inactive() {
        let sel = Selection::new();
        assert!(!sel.is_active());
        assert!(sel.range(0).is_none());
        assert!(!sel.contains(0, 0));
    }

    #[test]
    fn test_start_activates_selection() {
        let mut sel = Selection::new();
        sel.start(5);
        assert!(sel.is_active());
        assert_eq!(sel.range(5), Some((5, 5)));
    }

    #[test]
    fn test_clear_deactivates_selection() {
        let mut sel = Selection::new();
        sel.start(5);
        sel.clear();
        assert!(!sel.is_active());
    }

    #[test]
    fn test_contains_with_forward_selection() {
        let mut sel = Selection::new();
        sel.start(3);
        // Selection from 3 to 5 (cursor at 5)
        assert!(sel.contains(3, 5));
        assert!(sel.contains(4, 5));
        assert!(sel.contains(5, 5));
        assert!(!sel.contains(2, 5));
        assert!(!sel.contains(6, 5));
    }

    #[test]
    fn test_contains_with_backward_selection() {
        let mut sel = Selection::new();
        sel.start(5);
        // Selection from 3 to 5 (cursor at 3, moved up)
        assert!(sel.contains(3, 3));
        assert!(sel.contains(4, 3));
        assert!(sel.contains(5, 3));
        assert!(!sel.contains(2, 3));
        assert!(!sel.contains(6, 3));
    }

    #[test]
    fn test_range_with_forward_selection() {
        let mut sel = Selection::new();
        sel.start(3);
        assert_eq!(sel.range(7), Some((3, 7)));
    }

    #[test]
    fn test_range_with_backward_selection() {
        let mut sel = Selection::new();
        sel.start(7);
        assert_eq!(sel.range(3), Some((3, 7)));
    }

    #[test]
    fn test_extend_sets_direction() {
        let mut sel = Selection::new();
        sel.start(5);
        sel.extend(6, Direction::Down);
        // Direction is tracked but doesn't change the range logic
        assert_eq!(sel.range(6), Some((5, 6)));
    }
}
