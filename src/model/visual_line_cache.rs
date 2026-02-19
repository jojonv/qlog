use std::collections::HashMap;

/// Cached visual line information for a single logical line.
#[derive(Debug, Clone, Copy)]
pub struct CachedVisualInfo {
    /// The visual line offset (starting position in visual lines)
    pub offset: usize,
    /// Number of visual lines this logical line spans
    pub count: usize,
}

/// LRU-style cache for visual line calculations.
/// Avoids recalculating visual lines for the entire file on every scroll.
#[derive(Debug)]
pub struct VisualLineCache {
    /// Cache of line index -> visual info
    cache: HashMap<usize, CachedVisualInfo>,
    /// Maximum number of entries to keep in cache
    capacity: usize,
    /// Viewport height for calculating visual lines
    viewport_width: usize,
    /// Whether wrapping is enabled
    wrap_mode: bool,
    /// Total number of visual lines (cached for quick access)
    total_visual_lines: usize,
}

impl VisualLineCache {
    /// Create a new visual line cache.
    pub fn new(capacity: usize, viewport_width: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(capacity),
            capacity,
            viewport_width,
            wrap_mode: true,
            total_visual_lines: 0,
        }
    }

    /// Set wrap mode.
    pub fn set_wrap_mode(&mut self, wrap_mode: bool) {
        if self.wrap_mode != wrap_mode {
            self.wrap_mode = wrap_mode;
            self.clear();
        }
    }

    /// Set viewport width.
    pub fn set_viewport_width(&mut self, viewport_width: usize) {
        if self.viewport_width != viewport_width {
            self.viewport_width = viewport_width;
            self.clear();
        }
    }

    /// Calculate the number of visual lines for a text string.
    pub fn calculate_visual_lines(&self, text: &str) -> usize {
        if !self.wrap_mode || self.viewport_width == 0 {
            return 1;
        }

        let text_width = text.chars().count();
        ((text_width + self.viewport_width - 1) / self.viewport_width).max(1)
    }

    /// Calculate the number of visual lines for bytes (for filtered indices calculation).
    pub fn calculate_visual_lines_bytes(&self, bytes: &[u8]) -> usize {
        if !self.wrap_mode || self.viewport_width == 0 {
            return 1;
        }

        // Estimate character width from UTF-8 bytes
        // This is approximate but efficient
        let char_count = match std::str::from_utf8(bytes) {
            Ok(s) => s.chars().count(),
            Err(_) => bytes.len(), // Fallback to byte count for invalid UTF-8
        };

        ((char_count + self.viewport_width - 1) / self.viewport_width).max(1)
    }

    /// Get cached visual info for a line, or calculate if not cached.
    /// `line_text_fn` is a closure that returns the text for the line.
    pub fn get_or_calculate<F>(&mut self, line_idx: usize, line_text_fn: F) -> CachedVisualInfo
    where
        F: FnOnce() -> String,
    {
        if let Some(&info) = self.cache.get(&line_idx) {
            return info;
        }

        // Calculate
        let text = line_text_fn();
        let count = self.calculate_visual_lines(&text);

        let info = CachedVisualInfo { offset: 0, count };

        // Insert into cache (with simple eviction if at capacity)
        if self.cache.len() >= self.capacity {
            // Simple eviction: clear half the cache
            let keys_to_remove: Vec<_> =
                self.cache.keys().take(self.capacity / 2).copied().collect();
            for key in keys_to_remove {
                self.cache.remove(&key);
            }
        }

        self.cache.insert(line_idx, info);
        info
    }

    /// Calculate visual line offset for a range of lines.
    /// Returns a vector of (line_idx, visual_offset) pairs.
    pub fn calculate_range<F>(
        &mut self,
        line_indices: &[usize],
        start_idx: usize,
        end_idx: usize,
        line_text_fn: F,
    ) -> Vec<(usize, usize)>
    where
        F: Fn(usize) -> Option<String>,
    {
        let mut result = Vec::new();
        let mut current_offset = 0usize;

        for &line_idx in line_indices
            .iter()
            .skip(start_idx)
            .take(end_idx - start_idx)
        {
            let count = if let Some(&info) = self.cache.get(&line_idx) {
                info.count
            } else {
                let text = line_text_fn(line_idx).unwrap_or_default();
                let count = self.calculate_visual_lines(&text);

                // Cache it
                if self.cache.len() >= self.capacity {
                    let keys_to_remove: Vec<_> =
                        self.cache.keys().take(self.capacity / 2).copied().collect();
                    for key in keys_to_remove {
                        self.cache.remove(&key);
                    }
                }
                self.cache.insert(
                    line_idx,
                    CachedVisualInfo {
                        offset: current_offset,
                        count,
                    },
                );
                count
            };

            result.push((line_idx, current_offset));
            current_offset += count;
        }

        self.total_visual_lines = current_offset;
        result
    }

    /// Get the visual line offset for a specific line.
    pub fn get_offset(&self, line_idx: usize) -> Option<usize> {
        self.cache.get(&line_idx).map(|info| info.offset)
    }

    /// Get the total number of visual lines.
    pub fn total_visual_lines(&self) -> usize {
        self.total_visual_lines
    }

    /// Clear the cache.
    pub fn clear(&mut self) {
        self.cache.clear();
        self.total_visual_lines = 0;
    }

    /// Get the number of cached entries.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Get cache capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get wrap mode.
    pub fn wrap_mode(&self) -> bool {
        self.wrap_mode
    }

    /// Get viewport width.
    pub fn viewport_width(&self) -> usize {
        self.viewport_width
    }
}

impl Default for VisualLineCache {
    fn default() -> Self {
        Self::new(1000, 80)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_visual_lines() {
        let cache = VisualLineCache::new(100, 10);

        // Short text fits in one line
        assert_eq!(cache.calculate_visual_lines("hello"), 1);

        // Text that wraps to multiple lines
        assert_eq!(cache.calculate_visual_lines("hello world this is long"), 3);

        // Exactly fits
        assert_eq!(cache.calculate_visual_lines("0123456789"), 1);

        // One char over
        assert_eq!(cache.calculate_visual_lines("0123456789a"), 2);
    }

    #[test]
    fn test_wrap_mode_disabled() {
        let mut cache = VisualLineCache::new(100, 10);
        cache.set_wrap_mode(false);

        // Always 1 line when wrapping is disabled
        assert_eq!(cache.calculate_visual_lines("very long text here"), 1);
    }

    #[test]
    fn test_viewport_width_change() {
        let mut cache = VisualLineCache::new(100, 10);
        cache.set_viewport_width(20);

        assert_eq!(cache.calculate_visual_lines("hello world this is long"), 2);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = VisualLineCache::new(100, 10);

        cache.get_or_calculate(0, || "test".to_string());
        assert!(!cache.is_empty());

        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_calculate_visual_lines_bytes() {
        let cache = VisualLineCache::new(100, 10);

        assert_eq!(cache.calculate_visual_lines_bytes(b"hello"), 1);
        assert_eq!(
            cache.calculate_visual_lines_bytes(b"hello world this is long"),
            3
        );
    }
}
