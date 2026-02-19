use memmap2::Mmap;
use std::path::Path;

use crate::model::line_info::LineInfo;
use crate::model::mmap_str::MmapStr;
use crate::model::timestamp::detect_timestamp;

/// Memory-mapped log storage with line index.
/// Supports multiple files - each line stores which file (mmap) it belongs to.
#[derive(Debug)]
pub struct LogStorage {
    /// Multiple memory-mapped files
    mmaps: Vec<Mmap>,
    /// Index of line positions across all files
    lines: Vec<LineInfo>,
}

impl LogStorage {
    /// Create an empty LogStorage with no data.
    pub fn empty() -> Self {
        Self {
            mmaps: Vec::new(),
            lines: Vec::new(),
        }
    }

    /// Create a new LogStorage by memory-mapping a file and building the line index.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        let lines = Self::build_line_index(&mmap, 0); // file_index = 0 for single file

        Ok(Self {
            mmaps: vec![mmap],
            lines,
        })
    }

    /// Build the line index by scanning for newlines.
    fn build_line_index(mmap: &Mmap, file_index: u32) -> Vec<LineInfo> {
        let mut lines = Vec::new();
        let mut offset: u64 = 0;
        let mut line_start: u64 = 0;

        for &byte in mmap.iter() {
            if byte == b'\n' {
                let length = (offset - line_start) as u32;
                let line_data = &mmap[line_start as usize..offset as usize];
                let timestamp = detect_timestamp(&String::from_utf8_lossy(line_data));

                lines.push(LineInfo::with_timestamp(
                    file_index, line_start, length, timestamp,
                ));
                line_start = offset + 1;
            }
            offset += 1;
        }

        // Handle last line if file doesn't end with newline
        if line_start < mmap.len() as u64 {
            let length = (mmap.len() as u64 - line_start) as u32;
            let line_data = &mmap[line_start as usize..];
            let timestamp = detect_timestamp(&String::from_utf8_lossy(line_data));

            lines.push(LineInfo::with_timestamp(
                file_index, line_start, length, timestamp,
            ));
        }

        lines
    }

    /// Get the number of lines in the storage.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Check if the storage is empty.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Get a zero-copy view of the line at the given index.
    pub fn get_line(&self, idx: usize) -> Option<MmapStr<'_>> {
        let info = self.lines.get(idx)?;
        let mmap = self.mmaps.get(info.file_index as usize)?;
        let start = info.offset as usize;
        let end = start + info.length as usize;
        Some(MmapStr::new(&mmap[start..end]))
    }

    /// Get the LineInfo at the given index.
    pub fn get_line_info(&self, idx: usize) -> Option<&LineInfo> {
        self.lines.get(idx)
    }

    /// Iterate over all lines as MmapStr views.
    pub fn iter(&self) -> impl Iterator<Item = MmapStr> + '_ {
        self.lines.iter().map(move |info| {
            let mmap = &self.mmaps[info.file_index as usize];
            let start = info.offset as usize;
            let end = start + info.length as usize;
            MmapStr::new(&mmap[start..end])
        })
    }

    /// Iterate over lines with their indices.
    pub fn iter_enumerated(&self) -> impl Iterator<Item = (usize, MmapStr)> + '_ {
        self.lines.iter().enumerate().map(move |(idx, info)| {
            let mmap = &self.mmaps[info.file_index as usize];
            let start = info.offset as usize;
            let end = start + info.length as usize;
            (idx, MmapStr::new(&mmap[start..end]))
        })
    }

    /// Get raw bytes from the mmap at the given offset and length.
    /// Uses the file_index from the line info.
    pub fn get_bytes(&self, file_idx: usize, offset: u64, length: u32) -> Option<&[u8]> {
        let mmap = self.mmaps.get(file_idx)?;
        let start = offset as usize;
        let end = start + length as usize;
        Some(&mmap[start..end])
    }

    /// Get the number of memory-mapped files.
    pub fn file_count(&self) -> usize {
        self.mmaps.len()
    }

    /// Get the line index (for advanced use).
    pub fn line_index(&self) -> &[LineInfo] {
        &self.lines
    }

    /// Merge multiple LogStorage instances into one.
    /// All lines are combined with updated file indices.
    pub fn merge(storages: Vec<LogStorage>) -> Self {
        if storages.is_empty() {
            return Self::empty();
        }

        let total_lines: usize = storages.iter().map(|s| s.lines.len()).sum();
        let mut mmaps = Vec::with_capacity(storages.len());
        let mut lines = Vec::with_capacity(total_lines);

        for (file_idx, storage) in storages.into_iter().enumerate() {
            // Add all mmaps from this storage
            mmaps.extend(storage.mmaps);

            // Re-index lines to use the new file index
            for line in storage.lines {
                lines.push(LineInfo::with_timestamp(
                    file_idx as u32,
                    line.offset,
                    line.length,
                    line.timestamp,
                ));
            }
        }

        Self { mmaps, lines }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_storage_basic() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Line 1").unwrap();
        writeln!(temp_file, "Line 2").unwrap();
        writeln!(temp_file, "Line 3").unwrap();

        let storage = LogStorage::from_file(temp_file.path()).unwrap();

        assert_eq!(storage.len(), 3);
        assert!(!storage.is_empty());

        let line0 = storage.get_line(0).unwrap();
        assert_eq!(line0.as_str_lossy().trim(), "Line 1");

        let line1 = storage.get_line(1).unwrap();
        assert_eq!(line1.as_str_lossy().trim(), "Line 2");

        let line2 = storage.get_line(2).unwrap();
        assert_eq!(line2.as_str_lossy().trim(), "Line 3");
    }

    #[test]
    fn test_log_storage_no_trailing_newline() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "Only line").unwrap();

        let storage = LogStorage::from_file(temp_file.path()).unwrap();

        assert_eq!(storage.len(), 1);
        let line = storage.get_line(0).unwrap();
        assert_eq!(line.as_str_lossy(), "Only line");
    }

    #[test]
    fn test_log_storage_empty() {
        let temp_file = NamedTempFile::new().unwrap();

        let storage = LogStorage::from_file(temp_file.path()).unwrap();

        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());
        assert!(storage.get_line(0).is_none());
    }

    #[test]
    fn test_log_storage_iter() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "A").unwrap();
        writeln!(temp_file, "B").unwrap();

        let storage = LogStorage::from_file(temp_file.path()).unwrap();

        let lines: Vec<String> = storage
            .iter()
            .map(|s| s.as_str_lossy().trim().to_string())
            .collect();

        assert_eq!(lines, vec!["A", "B"]);
    }

    #[test]
    fn test_log_storage_empty_constructor() {
        let storage = LogStorage::empty();

        assert_eq!(storage.len(), 0);
        assert!(storage.is_empty());
        assert!(storage.get_line(0).is_none());
        assert_eq!(storage.file_count(), 0);
    }

    #[test]
    fn test_log_storage_iter_enumerated() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "A").unwrap();
        writeln!(temp_file, "B").unwrap();

        let storage = LogStorage::from_file(temp_file.path()).unwrap();

        let lines: Vec<(usize, String)> = storage
            .iter_enumerated()
            .map(|(idx, s)| (idx, s.as_str_lossy().trim().to_string()))
            .collect();

        assert_eq!(lines, vec![(0, "A".to_string()), (1, "B".to_string())]);
    }

    #[test]
    fn test_log_storage_merge() {
        let mut temp1 = NamedTempFile::new().unwrap();
        writeln!(temp1, "File1-Line1").unwrap();
        writeln!(temp1, "File1-Line2").unwrap();

        let mut temp2 = NamedTempFile::new().unwrap();
        writeln!(temp2, "File2-Line1").unwrap();

        let storage1 = LogStorage::from_file(temp1.path()).unwrap();
        let storage2 = LogStorage::from_file(temp2.path()).unwrap();

        let merged = LogStorage::merge(vec![storage1, storage2]);

        assert_eq!(merged.len(), 3);
        assert_eq!(merged.file_count(), 2);

        // Check lines from both files
        let line0 = merged.get_line(0).unwrap();
        assert_eq!(line0.as_str_lossy().trim(), "File1-Line1");

        let line1 = merged.get_line(1).unwrap();
        assert_eq!(line1.as_str_lossy().trim(), "File1-Line2");

        let line2 = merged.get_line(2).unwrap();
        assert_eq!(line2.as_str_lossy().trim(), "File2-Line1");
    }

    #[test]
    fn test_log_storage_merge_empty() {
        let merged = LogStorage::merge(vec![]);
        assert_eq!(merged.len(), 0);
        assert_eq!(merged.file_count(), 0);
    }
}
