use chrono::{DateTime, Utc};

/// Information about a single line in the log file.
/// Stores only metadata (16 bytes per line) instead of full content.
#[derive(Debug, Clone, Copy)]
pub struct LineInfo {
    pub offset: u64,
    pub length: u32,
    pub file_index: u32,
    pub timestamp: Option<DateTime<Utc>>,
}

impl LineInfo {
    /// Create a new LineInfo without timestamp.
    pub fn new(file_index: u32, offset: u64, length: u32) -> Self {
        Self {
            offset,
            length,
            file_index,
            timestamp: None,
        }
    }

    /// Create a new LineInfo with timestamp.
    pub fn with_timestamp(
        file_index: u32,
        offset: u64,
        length: u32,
        timestamp: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            offset,
            length,
            file_index,
            timestamp,
        }
    }

    /// Get the end offset (exclusive) of this line.
    pub fn end_offset(&self) -> u64 {
        self.offset + self.length as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_info_basic() {
        let info = LineInfo::new(0, 100, 50);

        assert_eq!(info.file_index, 0);
        assert_eq!(info.offset, 100);
        assert_eq!(info.length, 50);
        assert_eq!(info.end_offset(), 150);
        assert!(info.timestamp.is_none());
    }

    #[test]
    fn test_line_info_with_timestamp() {
        let timestamp = Utc::now();
        let info = LineInfo::with_timestamp(1, 200, 100, Some(timestamp));

        assert_eq!(info.file_index, 1);
        assert_eq!(info.offset, 200);
        assert_eq!(info.length, 100);
        assert_eq!(info.timestamp, Some(timestamp));
    }
}
