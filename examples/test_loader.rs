use como_log_viewer::model::LogEntry;
use como_log_viewer::storage::loader::LogLoader;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_files: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();

    if log_files.is_empty() {
        eprintln!("Usage: {} <log files...>", std::env::args().next().unwrap());
        std::process::exit(1);
    }

    println!("Testing loader with {} files...", log_files.len());

    let loader = LogLoader::new();
    let mut entries: Vec<LogEntry> = Vec::new();

    match loader.load_logs(&mut entries, &log_files) {
        Ok(stats) => {
            println!("\n✓ Successfully loaded:");
            println!(
                "  Files: {}/{} loaded",
                stats.loaded_files, stats.total_files
            );
            println!("  Total lines: {}", stats.total_lines);
            println!("  Parsed entries: {}", entries.len());

            if !entries.is_empty() {
                println!("\nFirst entry:");
                println!("  Timestamp: {}", entries[0].timestamp);
                println!("  Level: {:?}", entries[0].level);
                println!("  Source: {}", entries[0].source());
                let msg = if entries[0].message.len() > 80 {
                    format!("{:.80}...", entries[0].message)
                } else {
                    entries[0].message.clone()
                };
                println!("  Message: {}", msg);

                println!("\nLast entry:");
                let last = entries.len() - 1;
                println!("  Timestamp: {}", entries[last].timestamp);
                println!("  Level: {:?}", entries[last].level);
                println!("  Source: {}", entries[last].source());
                let msg = if entries[last].message.len() > 80 {
                    format!("{:.80}...", entries[last].message)
                } else {
                    entries[last].message.clone()
                };
                println!("  Message: {}", msg);

                // Count by level
                let mut info_count = 0usize;
                let mut warn_count = 0usize;
                let mut error_count = 0usize;

                for entry in &entries {
                    match entry.level {
                        como_log_viewer::model::LogLevel::Information => info_count += 1,
                        como_log_viewer::model::LogLevel::Warning => warn_count += 1,
                        como_log_viewer::model::LogLevel::Error => error_count += 1,
                    }
                }

                println!("\nLevel breakdown:");
                println!("  Information: {}", info_count);
                println!("  Warning: {}", warn_count);
                println!("  Error: {}", error_count);

                // Test date range
                if entries.len() >= 2 {
                    println!(
                        "\nDate range: {} to {}",
                        entries[0].timestamp.format("%Y-%m-%d %H:%M:%S"),
                        entries[entries.len() - 1]
                            .timestamp
                            .format("%Y-%m-%d %H:%m:%S")
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Error loading files: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
