use crate::app::{App, LoadingStatus, Mode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App) {
    // Show loading screen if still loading
    if let LoadingStatus::Loading { current, total } = &app.loading_status {
        draw_loading_screen(frame, *current, *total, app.logs.len());
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Filter bar
            Constraint::Min(0),    // Main view
            Constraint::Length(3), // Status bar
        ])
        .split(frame.size());

    // Filter bar
    let filter_spans: Vec<Span> = app
        .filters
        .filters
        .iter()
        .enumerate()
        .flat_map(|(_i, (filter, enabled))| {
            let style = if *enabled {
                Style::default()
                    .fg(match filter {
                        crate::model::Filter::Level(l) => match l {
                            crate::model::LogLevel::Error => Color::Red,
                            crate::model::LogLevel::Warning => Color::Yellow,
                            crate::model::LogLevel::Information => Color::Green,
                        },
                        _ => Color::Cyan,
                    })
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            vec![
                Span::styled(format!("[{}]", filter.display_name()), style),
                Span::raw(" "),
            ]
        })
        .collect();

    let filter_bar = Paragraph::new(Line::from(filter_spans))
        .block(Block::default().title("Filters").borders(Borders::ALL));
    frame.render_widget(filter_bar, chunks[0]);

    // Main view - log list
    let log_lines: Vec<Line> = app
        .filtered_logs
        .iter()
        .skip(app.scroll_offset)
        .take(chunks[1].height as usize)
        .map(|entry| {
            let level_color = match entry.level {
                crate::model::LogLevel::Error => Color::Red,
                crate::model::LogLevel::Warning => Color::Yellow,
                crate::model::LogLevel::Information => Color::White,
            };

            Line::from(vec![
                Span::styled(
                    entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(" "),
                Span::styled(
                    entry.level.as_str(),
                    Style::default()
                        .fg(level_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(entry.source(), Style::default().fg(Color::Blue)),
                Span::raw(" "),
                Span::raw(entry.message.clone()),
            ])
        })
        .collect();

    let main_view = Paragraph::new(log_lines)
        .block(
            Block::default()
                .title(format!(
                    "Logs ({} total, {} filtered)",
                    app.logs.len(),
                    app.filtered_logs.len()
                ))
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true })
        .scroll((0, app.horizontal_scroll as u16));
    frame.render_widget(main_view, chunks[1]);

    // Status bar
    let status_text = format!(
        "{} | Line {}/{} | h/j/k/l: Navigate | q: Quit | f: Filter | Space: Toggle",
        app.status_message,
        app.selected_line + 1,
        app.filtered_logs.len()
    );

    let status_bar = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL))
        .style(match app.mode {
            Mode::Normal => Style::default(),
            _ => Style::default().fg(Color::Yellow),
        });
    frame.render_widget(status_bar, chunks[2]);
}

fn draw_loading_screen(frame: &mut Frame, current: usize, total: usize, entries: usize) {
    let area = frame.size();

    // Calculate progress percentage
    let progress_pct = if total > 0 {
        (current * 100) / total
    } else {
        0
    };

    // Create loading text
    let loading_text = Text::from(vec![
        Line::from(vec![Span::styled(
            "Loading Como Log Viewer...",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::raw("File: "),
            Span::styled(
                format!("{} / {}", current, total),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::raw("Entries: "),
            Span::styled(entries.to_string(), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("[", Style::default().fg(Color::Cyan)),
            Span::styled(
                ">".repeat(progress_pct / 5).to_string(),
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                " ".repeat(20 - progress_pct / 5).to_string(),
                Style::default(),
            ),
            Span::styled("]", Style::default().fg(Color::Cyan)),
            Span::raw(" "),
            Span::styled(
                format!("{}%", progress_pct),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press 'q' to cancel",
            Style::default().fg(Color::Gray),
        )]),
    ]);

    let loading_paragraph = Paragraph::new(loading_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title("Loading")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(loading_paragraph, area);
}
