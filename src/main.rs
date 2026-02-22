use std::env;
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
use ratatui::{backend::CrosstermBackend, Terminal};
use walkdir::WalkDir;

use qlog::{
    app::{App, LoadingStatus},
    model::LogStorage,
    storage::loader::LogLoader,
};

const DEFAULT_MAX_OPEN_DIRS: usize = 10;
const MMAP_THRESHOLD: u64 = 10 * 1024 * 1024;
const MAX_RETRIES: usize = 3;
const INITIAL_RETRY_MS: u64 = 100;

fn get_max_open_dirs() -> usize {
    env::var("COMO_MAX_OPEN_DIRS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_MAX_OPEN_DIRS)
}

fn is_fd_exhaustion_error(e: &(dyn std::error::Error + 'static)) -> bool {
    e.downcast_ref::<io::Error>()
        .map(|e| matches!(e.raw_os_error(), Some(24) | Some(23)))
        .unwrap_or(false)
}

fn get_fd_limit() -> Option<usize> {
    #[cfg(unix)]
    {
        unsafe {
            let mut rlimit: libc::rlimit = std::mem::zeroed();
            if libc::getrlimit(libc::RLIMIT_NOFILE, &mut rlimit) == 0 {
                return Some(rlimit.rlim_cur as usize);
            }
        }
    }
    None
}

fn check_fd_warning() {
    if let Some(limit) = get_fd_limit() {
        let current = get_fd_count().unwrap_or(0);
        if current > (limit as f64 * 0.8) as usize {
            eprintln!(
                "Warning: FD usage at {}% ({} of {}). Consider: ulimit -n 65536",
                (current * 100 / limit),
                current,
                limit
            );
        }
    }
}

#[cfg(unix)]
fn get_fd_count() -> Option<usize> {
    std::fs::read_dir("/proc/self/fd").ok().map(|d| d.count())
}

#[cfg(not(unix))]
fn get_fd_count() -> Option<usize> {
    None
}

#[derive(Debug, Clone)]
pub struct LoadProgress {
    pub current_file: usize,
    pub total_files: usize,
    pub entries_loaded: usize,
    pub current_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct LoadStats {
    pub files_loaded: usize,
    pub files_failed: usize,
    pub entries_loaded: usize,
    pub failed_paths: Vec<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let max_open_dirs = get_max_open_dirs();

    let (progress_tx, progress_rx): (mpsc::Sender<LoadProgress>, mpsc::Receiver<LoadProgress>) =
        mpsc::channel();
    let (logs_tx, logs_rx): (
        mpsc::Sender<(LogStorage, LoadStats)>,
        mpsc::Receiver<(LogStorage, LoadStats)>,
    ) = mpsc::channel();
    let (incremental_tx, incremental_rx): (mpsc::Sender<LogStorage>, mpsc::Receiver<LogStorage>) =
        mpsc::channel();

    let args_clone = args.clone();
    thread::spawn(move || {
        let loader = LogLoader::new(0); // Will be updated with actual count
        let mut all_storages: Vec<LogStorage> = Vec::new();
        let mut stats = LoadStats::default();
        let mut file_count = 0usize;

        // First pass: collect all paths
        let paths: Vec<PathBuf> = if args_clone.len() > 1 {
            collect_paths(&args_clone[1..], max_open_dirs)
        } else {
            WalkDir::new(".")
                .max_open(max_open_dirs)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .map(|e| e.path().to_path_buf())
                .filter(|p| is_log_file(p))
                .collect()
        };

        let total_files = paths.len();

        // Send progress with total count
        let _ = progress_tx.send(LoadProgress {
            current_file: 0,
            total_files,
            entries_loaded: 0,
            current_path: None,
        });

        for path in paths {
            file_count += 1;

            let progress = LoadProgress {
                current_file: file_count,
                total_files,
                entries_loaded: all_storages.iter().map(|s| s.len()).sum(),
                current_path: Some(path.clone()),
            };
            let _ = progress_tx.send(progress);

            check_fd_warning();

            let mut attempt = 0;
            let mut delay = INITIAL_RETRY_MS;

            loop {
                match LogStorage::from_file(&path) {
                    Ok(storage) => {
                        let entry_count = storage.len();
                        stats.files_loaded += 1;
                        stats.entries_loaded += entry_count;

                        // Store for later combination
                        all_storages.push(storage);
                        break;
                    }
                    Err(e) if is_fd_exhaustion_error(&*e) && attempt < MAX_RETRIES => {
                        eprintln!(
                            "FD exhaustion on {}, retry {}/{} after {}ms",
                            path.display(),
                            attempt + 1,
                            MAX_RETRIES,
                            delay
                        );
                        thread::sleep(Duration::from_millis(delay));
                        delay *= 2;
                        attempt += 1;
                    }
                    Err(e) if is_fd_exhaustion_error(&*e) => {
                        eprintln!(
                            "Failed to load {} after {} retries: FD limit reached. Try: ulimit -n 65536",
                            path.display(),
                            MAX_RETRIES
                        );
                        stats.files_failed += 1;
                        if stats.failed_paths.len() < 5 {
                            stats.failed_paths.push(path);
                        }
                        break;
                    }
                    Err(e) => {
                        eprintln!("Error loading {}: {}", path.display(), e);
                        stats.files_failed += 1;
                        if stats.failed_paths.len() < 5 {
                            stats.failed_paths.push(path);
                        }
                        break;
                    }
                }
            }
        }

        // Combine all storages into one
        let combined_storage = combine_storages(all_storages);

        let _ = logs_tx.send((combined_storage, stats));
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(
        &mut terminal,
        &mut app,
        progress_rx,
        logs_rx,
        incremental_rx,
    );

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

/// Combine multiple LogStorage instances into one.
fn combine_storages(storages: Vec<LogStorage>) -> LogStorage {
    LogStorage::merge(storages)
}

fn collect_paths(args: &[String], max_open_dirs: usize) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    for arg in args {
        let path = PathBuf::from(arg);
        if path.is_dir() {
            paths.extend(
                WalkDir::new(&path)
                    .max_open(max_open_dirs)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                    .map(|e| e.path().to_path_buf())
                    .filter(|p| is_log_file(p)),
            );
        } else if arg.contains('*') || arg.contains('?') {
            let pattern = arg.clone();
            paths.extend(
                WalkDir::new(".")
                    .max_open(max_open_dirs)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                    .map(|e| e.path().to_path_buf())
                    .filter(|p| matches_glob_pattern(p, &pattern)),
            );
        } else if path.exists() {
            paths.push(path);
        } else {
            eprintln!("Warning: {} not found, skipping", arg);
        }
    }

    paths
}

fn matches_glob_pattern(path: &Path, pattern: &str) -> bool {
    let path_str = path.to_string_lossy();
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();

    if pattern.contains('/') {
        glob_match(&path_str, pattern)
    } else {
        glob_match(&file_name, pattern)
    }
}

fn glob_match(text: &str, pattern: &str) -> bool {
    let text_chars: Vec<char> = text.chars().collect();
    let pattern_chars: Vec<char> = pattern.chars().collect();

    fn match_helper(text: &[char], pattern: &[char]) -> bool {
        match (text.first(), pattern.first()) {
            (None, None) => true,
            (Some(_), None) => false,
            (None, Some('*')) => match_helper(&[], &pattern[1..]),
            (None, Some(_)) => false,
            (Some(_), Some('*')) => {
                match_helper(text, &pattern[1..]) || match_helper(&text[1..], pattern)
            }
            (Some(t), Some(p)) if *p == '?' || t == p => match_helper(&text[1..], &pattern[1..]),
            (Some(_), Some(_)) => false,
        }
    }

    match_helper(&text_chars, &pattern_chars)
}

fn is_log_file(path: &Path) -> bool {
    if let Some(name) = path.file_name() {
        let name = name.to_string_lossy();
        name.starts_with("como-data-center") && name.ends_with(".log")
    } else {
        false
    }
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    progress_rx: mpsc::Receiver<LoadProgress>,
    logs_rx: mpsc::Receiver<(LogStorage, LoadStats)>,
    incremental_rx: mpsc::Receiver<LogStorage>,
) -> io::Result<()> {
    let mut last_tick = std::time::Instant::now();
    let tick_rate = Duration::from_millis(50);
    let mut stats: Option<LoadStats> = None;

    while !app.should_quit {
        while let Ok(progress) = progress_rx.try_recv() {
            app.loading_status = LoadingStatus::Loading {
                current: progress.current_file,
                total: if progress.total_files > 0 {
                    progress.total_files
                } else {
                    progress.current_file
                },
            };
        }

        while let Ok(storage) = incremental_rx.try_recv() {
            app.set_storage(storage);
        }

        if let Ok((final_storage, final_stats)) = logs_rx.try_recv() {
            stats = Some(final_stats);
            app.loading_status = LoadingStatus::Complete;
            app.set_storage(final_storage);
        }

        if let Some(ref s) = stats {
            if app.status_message.is_empty() {
                app.status_message = format!(
                    "Loaded {} entries from {} files ({} failed)",
                    s.entries_loaded, s.files_loaded, s.files_failed
                );
                if !s.failed_paths.is_empty() {
                    eprintln!("Failed files: {:?}", s.failed_paths);
                }
            }
        }

        terminal.draw(|f| qlog::ui::draw(f, app))?;

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
