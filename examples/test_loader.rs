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
                if let Some(ts) = &entries[0].timestamp {
                    println!("  Timestamp: {}", ts);
                } else {
                    println!("  Timestamp: (none detected)");
                }
                let msg = if entries[0].raw.len() > 80 {
                    format!("{:.80}...", entries[0].raw)
                } else {
                    entries[0].raw.clone()
                };
                println!("  Raw: {}", msg);

                println!("\nLast entry:");
                let last = entries.len() - 1;
                if let Some(ts) = &entries[last].timestamp {
                    println!("  Timestamp: {}", ts);
                } else {
                    println!("  Timestamp: (none detected)");
                }
                let msg = if entries[last].raw.len() > 80 {
                    format!("{:.80}...", entries[last].raw)
                } else {
                    entries[last].raw.clone()
                };
                println!("  Raw: {}", msg);

                let with_timestamp = entries.iter().filter(|e| e.timestamp.is_some()).count();
                println!("\nTimestamp detection:");
                println!("  With timestamp: {}/{}", with_timestamp, entries.len());

                if entries.len() >= 2 {
                    let first_ts = entries[0].timestamp.as_ref();
                    let last_ts = entries[last].timestamp.as_ref();
                    if let (Some(f), Some(l)) = (first_ts, last_ts) {
                        println!(
                            "\nDate range: {} to {}",
                            f.format("%Y-%m-%d %H:%M:%S"),
                            l.format("%Y-%m-%d %H:%M:%S")
                        );
                    }
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
