use glob::glob;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

use como_log_viewer::{app::App, model::LogEntry, storage::loader::LogLoader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let log_paths: Vec<PathBuf> = if args.len() > 1 {
        let mut paths = Vec::new();
        for arg in &args[1..] {
            let path = PathBuf::from(arg);
            if path.is_dir() {
                let dir_files = find_log_files(&path)?;
                paths.extend(dir_files);
            } else if arg.contains('*') || arg.contains('?') {
                if let Ok(entries) = glob(arg) {
                    for entry in entries.flatten() {
                        paths.push(entry);
                    }
                }
            } else if path.exists() {
                paths.push(path);
            } else {
                eprintln!("Warning: {} not found, skipping", arg);
            }
        }
        paths.sort();
        paths.dedup();
        paths
    } else {
        find_log_files(".")?
    };

    if log_paths.is_empty() {
        eprintln!("No log files found!");
        eprintln!("Usage: como-log-viewer [log-files...]");
        std::process::exit(1);
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let (progress_tx, progress_rx): (mpsc::Sender<LoadProgress>, mpsc::Receiver<LoadProgress>) =
        mpsc::channel();
    let (logs_tx, logs_rx): (mpsc::Sender<Vec<LogEntry>>, mpsc::Receiver<Vec<LogEntry>>) =
        mpsc::channel();

    let log_paths_clone = log_paths.clone();
    thread::spawn(move || {
        let loader = LogLoader::new();
        let mut logs = Vec::new();

        for (i, path) in log_paths_clone.iter().enumerate() {
            let progress = LoadProgress {
                current_file: i,
                total_files: log_paths_clone.len(),
                entries_loaded: logs.len(),
            };
            let _ = progress_tx.send(progress);

            let _ = loader.load_file(path, &mut logs);
        }

        let _ = progress_tx.send(LoadProgress {
            current_file: log_paths_clone.len(),
            total_files: log_paths_clone.len(),
            entries_loaded: logs.len(),
        });

        let _ = logs_tx.send(logs);
    });

    let res = run_app(&mut terminal, &mut app, progress_rx, logs_rx);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

struct LoadProgress {
    current_file: usize,
    total_files: usize,
    entries_loaded: usize,
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    progress_rx: mpsc::Receiver<LoadProgress>,
    logs_rx: mpsc::Receiver<Vec<LogEntry>>,
) -> io::Result<()> {
    let mut last_tick = std::time::Instant::now();
    let tick_rate = Duration::from_millis(50);

    while !app.should_quit {
        while let Ok(progress) = progress_rx.try_recv() {
            app.loading_status = como_log_viewer::app::LoadingStatus::Loading {
                current: progress.current_file,
                total: progress.total_files,
            };
        }

        if matches!(
            app.loading_status,
            como_log_viewer::app::LoadingStatus::Loading { .. }
        ) {
            if let Ok(logs) = logs_rx.try_recv() {
                app.logs = logs;
                app.update_filtered_logs();
                app.loading_status = como_log_viewer::app::LoadingStatus::Complete;
                app.status_message = format!("Loaded {} entries", app.logs.len());
            }
        }

        terminal.draw(|f| como_log_viewer::ui::draw(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = crossterm::event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('c')
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            app.should_quit = true;
                        }
                        _ => {
                            app.handle_key(key);
                        }
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = std::time::Instant::now();
        }
    }

    Ok(())
}

fn find_log_files<P: AsRef<Path>>(dir: P) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let dir = std::fs::read_dir(dir)?;

    for entry in dir {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name() {
                let name = name.to_string_lossy();
                if name.starts_with("como-data-center") && name.ends_with(".log") {
                    files.push(path);
                }
            }
        }
    }

    files.sort();
    Ok(files)
}
