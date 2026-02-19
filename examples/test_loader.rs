use como_log_viewer::model::LogStorage;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_files: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();

    if log_files.is_empty() {
        eprintln!("Usage: {} <log files...>", std::env::args().next().unwrap());
        std::process::exit(1);
    }

    println!("Testing LogStorage with {} files...", log_files.len());

    let mut total_entries = 0;
    let mut loaded_files = 0;

    for path in &log_files {
        match LogStorage::from_file(path) {
            Ok(storage) => {
                loaded_files += 1;
                total_entries += storage.len();

                println!("\n✓ Loaded: {}", path.display());
                println!("  Lines: {}", storage.len());

                if storage.len() > 0 {
                    let first = storage.get_line(0).unwrap();
                    println!("  First line: {}", first.as_str_lossy().trim());

                    if let Some(info) = storage.get_line_info(0) {
                        if let Some(ts) = info.timestamp {
                            println!("  First timestamp: {}", ts);
                        }
                    }

                    let last = storage.get_line(storage.len() - 1).unwrap();
                    println!("  Last line: {}", last.as_str_lossy().trim());
                }
            }
            Err(e) => {
                eprintln!("✗ Error loading {}: {}", path.display(), e);
            }
        }
    }

    println!("\n--- Summary ---");
    println!("Files loaded: {}/{}", loaded_files, log_files.len());
    println!("Total entries: {}", total_entries);

    Ok(())
}
