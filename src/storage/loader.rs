use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::model::LogStorage;

/// Statistics about the loading process.
#[derive(Debug, Clone)]
pub struct LoadStat {
    pub total_files: usize,
    pub loaded_files: usize,
    pub total_lines: usize,
}

/// Handles loading of log files using memory-mapped storage.
pub struct LogLoader {
    /// Counter for tracking progress
    loaded_count: Arc<Mutex<usize>>,
    /// Total number of files to load
    total_files: usize,
}

impl LogLoader {
    /// Create a new LogLoader.
    pub fn new(total_files: usize) -> Self {
        Self {
            loaded_count: Arc::new(Mutex::new(0)),
            total_files,
        }
    }

    /// Load logs from the given paths into LogStorage instances.
    /// Returns a vector of LogStorage objects and load statistics.
    pub fn load_logs<P: AsRef<Path>>(
        &self,
        paths: &[P],
    ) -> Result<(Vec<LogStorage>, LoadStat), Box<dyn std::error::Error>> {
        let mut storages = Vec::new();
        let mut total_lines = 0;

        for path in paths {
            let path = path.as_ref();
            if !path.exists() {
                eprintln!("Warning: File not found: {}", path.display());
                continue;
            }

            match self.load_file(path) {
                Ok(storage) => {
                    total_lines += storage.len();
                    storages.push(storage);
                }
                Err(e) => {
                    eprintln!("Error loading {}: {}", path.display(), e);
                }
            }

            // Update progress
            if let Ok(mut count) = self.loaded_count.lock() {
                *count += 1;
            }
        }

        let loaded_files = storages.len();
        let stat = LoadStat {
            total_files: self.total_files,
            loaded_files,
            total_lines,
        };

        Ok((storages, stat))
    }

    /// Load a single file into a LogStorage.
    fn load_file<P: AsRef<Path>>(&self, path: P) -> Result<LogStorage, Box<dyn std::error::Error>> {
        LogStorage::from_file(path)
    }

    /// Get current loading progress.
    pub fn progress(&self) -> (usize, usize) {
        let loaded = match self.loaded_count.lock() {
            Ok(guard) => *guard,
            Err(poisoned) => *poisoned.into_inner(),
        };
        (loaded, self.total_files)
    }

    /// Check if loading is complete.
    pub fn is_complete(&self) -> bool {
        let loaded = match self.loaded_count.lock() {
            Ok(guard) => *guard,
            Err(poisoned) => *poisoned.into_inner(),
        };
        loaded >= self.total_files
    }
}

/// Create a LogLoader for the given paths.
pub fn create_loader<P: AsRef<Path>>(paths: &[P]) -> LogLoader {
    LogLoader::new(paths.len())
}

/// Load logs from a single file.
pub fn load_single_file<P: AsRef<Path>>(path: P) -> Result<LogStorage, Box<dyn std::error::Error>> {
    LogStorage::from_file(path)
}

/// Load logs from multiple files.
pub fn load_multiple_files<P: AsRef<Path>>(
    paths: &[P],
) -> Result<(Vec<LogStorage>, LoadStat), Box<dyn std::error::Error>> {
    let loader = create_loader(paths);
    loader.load_logs(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_single_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Line 1").unwrap();
        writeln!(temp_file, "Line 2").unwrap();
        writeln!(temp_file, "Line 3").unwrap();

        let storage = load_single_file(temp_file.path()).unwrap();

        assert_eq!(storage.len(), 3);
        assert_eq!(storage.get_line(0).unwrap().as_str_lossy().trim(), "Line 1");
    }

    #[test]
    fn test_load_multiple_files() {
        let mut temp_file1 = NamedTempFile::new().unwrap();
        writeln!(temp_file1, "File1 Line1").unwrap();
        writeln!(temp_file1, "File1 Line2").unwrap();

        let mut temp_file2 = NamedTempFile::new().unwrap();
        writeln!(temp_file2, "File2 Line1").unwrap();

        let paths = vec![temp_file1.path(), temp_file2.path()];
        let (storages, stat) = load_multiple_files(&paths).unwrap();

        assert_eq!(storages.len(), 2);
        assert_eq!(stat.total_files, 2);
        assert_eq!(stat.loaded_files, 2);
        assert_eq!(stat.total_lines, 3); // 2 + 1

        assert_eq!(storages[0].len(), 2);
        assert_eq!(storages[1].len(), 1);
    }

    #[test]
    fn test_loader_progress() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test").unwrap();

        let loader = LogLoader::new(1);
        assert_eq!(loader.progress(), (0, 1));

        let _ = loader.load_file(temp_file.path());
        // Progress updated during load
    }
}
