use glob::glob;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};

use como_log_viewer::{
    model::{Filter, FilterSet, LogEntry, LogLevel},
    storage::loader::LogLoader,
};

pub struct App {
    pub logs: Vec<LogEntry>,
    pub filtered_logs: Vec<LogEntry>,
    pub filters: FilterSet,
    pub scroll_offset: usize,
    pub horizontal_scroll: usize,
    pub should_quit: bool,
    pub loading: bool,
    pub loading_progress: LoadingProgress,
    pub status_message: String,
}

#[derive(Clone)]
pub struct LoadingProgress {
    pub current_file: usize,
    pub total_files: usize,
    pub current_lines: usize,
    pub total_lines: usize,
}

impl Default for LoadingProgress {
    fn default() -> Self {
        Self {
            current_file: 0,
            total_files: 0,
            current_lines: 0,
            total_lines: 0,
        }
    }
}

impl App {
    pub fn new() -> Self {
        let mut filters = FilterSet::new();
        // Add default filters
        filters.add(Filter::Level(LogLevel::Error));
        filters.add(Filter::Level(LogLevel::Warning));

        Self {
            logs: Vec::new(),
            filtered_logs: Vec::new(),
            filters,
            scroll_offset: 0,
            horizontal_scroll: 0,
            should_quit: false,
            loading: true,
            loading_progress: LoadingProgress::default(),
            status_message: String::new(),
        }
    }

    pub fn update_filtered_logs(&mut self) {
        self.filtered_logs = self
            .logs
            .iter()
            .filter(|log| self.filters.matches(log))
            .cloned()
            .collect();
    }

    fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    fn scroll_down(&mut self, amount: usize) {
        let max = self.filtered_logs.len().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + amount).min(max);
    }

    fn scroll_left(&mut self, amount: usize) {
        self.horizontal_scroll = self.horizontal_scroll.saturating_sub(amount);
    }

    fn scroll_right(&mut self, amount: usize) {
        self.horizontal_scroll += amount;
    }

    fn go_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    fn go_to_bottom(&mut self) {
        if !self.filtered_logs.is_empty() {
            self.scroll_offset = self.filtered_logs.len() - 1;
        }
    }

    fn page_up(&mut self, height: usize) {
        self.scroll_up(height.saturating_sub(3));
    }

    fn page_down(&mut self, height: usize) {
        self.scroll_down(height.saturating_sub(3));
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('h') | KeyCode::Left => self.scroll_left(1),
            KeyCode::Char('j') | KeyCode::Down => self.scroll_down(1),
            KeyCode::Char('k') | KeyCode::Up => self.scroll_up(1),
            KeyCode::Char('l') | KeyCode::Right => self.scroll_right(1),
            KeyCode::Char('g') => self.go_to_top(),
            KeyCode::Char('G') => self.go_to_bottom(),
            KeyCode::Char('f')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.page_down(20)
            }
            KeyCode::Char('b')
                if key
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::CONTROL) =>
            {
                self.page_up(20)
            }
            KeyCode::PageDown => self.page_down(20),
            KeyCode::PageUp => self.page_up(20),
            KeyCode::Home => self.go_to_top(),
            KeyCode::End => self.go_to_bottom(),
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse arguments
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

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and channel for loading progress
    let mut app = App::new();
    let (progress_tx, progress_rx): (
        mpsc::Sender<LoadingProgress>,
        mpsc::Receiver<LoadingProgress>,
    ) = mpsc::channel();
    let (logs_tx, logs_rx): (mpsc::Sender<Vec<LogEntry>>, mpsc::Receiver<Vec<LogEntry>>) =
        mpsc::channel();

    // Start loading in background thread
    let log_paths_clone = log_paths.clone();
    thread::spawn(move || {
        let loader = LogLoader::new();
        let mut logs = Vec::new();

        for (i, path) in log_paths_clone.iter().enumerate() {
            // Update progress before loading each file
            let progress = LoadingProgress {
                current_file: i,
                total_files: log_paths_clone.len(),
                current_lines: logs.len(),
                total_lines: log_paths_clone.len() * 250000, // Rough estimate
            };
            let _ = progress_tx.send(progress);

            // Load the file
            let _ = loader.load_file(path, &mut logs);
        }

        // Final progress
        let _ = progress_tx.send(LoadingProgress {
            current_file: log_paths_clone.len(),
            total_files: log_paths_clone.len(),
            current_lines: logs.len(),
            total_lines: logs.len(),
        });

        // Send completed logs
        let _ = logs_tx.send(logs);
    });

    // Run event loop with loading
    let res = run_app(&mut terminal, &mut app, progress_rx, logs_rx);

    // Restore terminal
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

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    progress_rx: mpsc::Receiver<LoadingProgress>,
    logs_rx: mpsc::Receiver<Vec<LogEntry>>,
) -> io::Result<()> {
    let mut last_tick = std::time::Instant::now();
    let tick_rate = Duration::from_millis(50); // 20 FPS for smoother loading

    while !app.should_quit {
        // Check for loading progress updates
        while let Ok(progress) = progress_rx.try_recv() {
            app.loading_progress = progress;
        }

        // Check if logs are done loading
        if app.loading {
            if let Ok(logs) = logs_rx.try_recv() {
                app.logs = logs;
                app.update_filtered_logs();
                app.loading = false;
                app.status_message = format!("Loaded {} entries", app.logs.len());
            }
        }

        // Draw UI
        terminal.draw(|f| draw(f, app))?;

        // Handle input with timeout
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = crossterm::event::read()? {
                app.handle_key(key);
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = std::time::Instant::now();
        }
    }

    Ok(())
}

fn draw(frame: &mut Frame, app: &App) {
    let size = frame.size();

    if app.loading {
        draw_loading_screen(frame, app, size);
    } else {
        draw_main_ui(frame, app, size);
    }
}

fn draw_loading_screen(frame: &mut Frame, app: &App, area: Rect) {
    let progress = &app.loading_progress;
    let percent = if progress.total_files > 0 {
        (progress.current_file as f64 / progress.total_files as f64 * 100.0) as u16
    } else {
        0
    };

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Loading Como Log Viewer...",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!(
            "File: {} / {}",
            progress.current_file, progress.total_files
        )),
        Line::from(format!("Entries: {}", progress.current_lines)),
        Line::from(""),
        Line::from(format!(
            "[{}] {}%",
            "=".repeat((percent as usize / 5)).to_string() + ">",
            percent
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press 'q' to cancel",
            Style::default().fg(Color::Gray),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(ratatui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });

    // Center the widget
    let area = centered_rect(60, 40, area);
    frame.render_widget(Clear, area);
    frame.render_widget(paragraph, area);
}

fn draw_main_ui(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Filter bar
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    // Filter bar
    let filter_text = if app.filters.is_empty() {
        "No filters active".to_string()
    } else {
        format!("Filters: {:?}", app.filters)
    };
    let filter_bar =
        Paragraph::new(filter_text).block(Block::default().borders(Borders::ALL).title("Filters"));
    frame.render_widget(filter_bar, chunks[0]);

    // Main content - log list
    let log_text: Vec<Line> = if app.filtered_logs.is_empty() {
        vec![Line::from("No logs match the current filters")]
    } else {
        app.filtered_logs
            .iter()
            .skip(app.scroll_offset)
            .take(chunks[1].height as usize)
            .map(|log| {
                let level_color = match log.level {
                    LogLevel::Error => Color::Red,
                    LogLevel::Warning => Color::Yellow,
                    LogLevel::Information => Color::Green,
                };

                let timestamp = log.timestamp.format("%Y-%m-%d %H:%M:%S");
                let msg_start = app.horizontal_scroll.min(log.message.len());
                let msg = if msg_start < log.message.len() {
                    &log.message[msg_start..]
                } else {
                    ""
                };

                Line::from(vec![
                    Span::styled(format!("{} ", timestamp), Style::default().fg(Color::Cyan)),
                    Span::styled(
                        format!("{:?} ", log.level),
                        Style::default().fg(level_color),
                    ),
                    Span::raw(msg.chars().take(100).collect::<String>()),
                ])
            })
            .collect()
    };

    let logs_widget = Paragraph::new(log_text)
        .block(Block::default().borders(Borders::ALL).title("Logs"))
        .wrap(Wrap { trim: false });
    frame.render_widget(logs_widget, chunks[1]);

    // Status bar
    let status = format!(
        "{} | Lines: {}/{} | Scroll: {} | {}",
        if app.filters.is_empty() {
            "All"
        } else {
            "Filtered"
        },
        app.filtered_logs.len(),
        app.logs.len(),
        app.scroll_offset,
        app.status_message
    );
    let status_bar = Paragraph::new(status);
    frame.render_widget(status_bar, chunks[2]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
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
