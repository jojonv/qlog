use crate::model::{detect_timestamp, LogEntry};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::sync::Mutex;

pub struct LogLoader {
    loaded_count: Mutex<usize>,
    total_files: usize,
}

#[derive(Debug, Clone)]
pub struct LoadStat {
    pub total_files: usize,
    pub loaded_files: usize,
    pub total_lines: usize,
}

impl LogLoader {
    pub fn new() -> Self {
        Self {
            loaded_count: Mutex::new(0),
            total_files: 0,
        }
    }

    pub fn load_logs(
        &self,
        logs: &mut Vec<LogEntry>,
        paths: &[impl AsRef<Path>],
    ) -> io::Result<LoadStat> {
        let mut total_lines = 0;
        let mut loaded_files = 0;

        for path in paths {
            match self.load_file(path.as_ref(), logs) {
                Ok(lines) => {
                    total_lines += lines;
                    loaded_files += 1;
                }
                Err(e) => {
                    eprintln!("Error loading {}: {}", path.as_ref().display(), e);
                }
            }
        }

        logs.sort_by(|a, b| match (&a.timestamp, &b.timestamp) {
            (Some(ta), Some(tb)) => ta.cmp(tb),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });

        Ok(LoadStat {
            total_files: paths.len(),
            loaded_files,
            total_lines,
        })
    }

    pub fn load_file(&self, path: &Path, logs: &mut Vec<LogEntry>) -> io::Result<usize> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines() {
            if let Ok(line) = line {
                if !line.trim().is_empty() {
                    let timestamp = detect_timestamp(&line);
                    logs.push(LogEntry::new(line, timestamp));
                    count += 1;
                }
            }
        }

        Ok(count)
    }
}

impl Default for LogLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_file_with_timestamp() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "2026-02-13T10:00:00Z This is a log message").unwrap();

        let loader = LogLoader::new();
        let mut logs = Vec::new();
        let lines = loader.load_file(temp_file.path(), &mut logs).unwrap();

        assert_eq!(lines, 1);
        assert_eq!(logs.len(), 1);
        assert!(logs[0].timestamp.is_some());
    }

    #[test]
    fn test_load_file_without_timestamp() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "This is a log message without timestamp").unwrap();

        let loader = LogLoader::new();
        let mut logs = Vec::new();
        let lines = loader.load_file(temp_file.path(), &mut logs).unwrap();

        assert_eq!(lines, 1);
        assert_eq!(logs.len(), 1);
        assert!(logs[0].timestamp.is_none());
    }
}
