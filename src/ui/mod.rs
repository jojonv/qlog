use crate::app::{App, LoadingStatus, Mode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App) {
    if let LoadingStatus::Loading { current, total } = &app.loading_status {
        draw_loading_screen(frame, *current, *total, app.logs.len());
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if matches!(app.mode, Mode::FilterInput) {
            vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ]
        } else {
            vec![
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ]
        })
        .split(frame.size());

    draw_filter_bar(frame, app, chunks[0]);

    let main_chunk;
    let status_chunk;

    if matches!(app.mode, Mode::FilterInput) {
        draw_filter_input(frame, app, chunks[1]);
        main_chunk = chunks[2];
        status_chunk = chunks[3];
    } else {
        main_chunk = chunks[1];
        status_chunk = chunks[2];
    }

    draw_main_view(frame, app, main_chunk);
    draw_status_bar(frame, app, status_chunk);
}

fn draw_filter_bar(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let mut spans: Vec<Span> = Vec::new();

    for (gi, group) in app.filters.groups.iter().enumerate() {
        if gi > 0 {
            spans.push(Span::styled(" | ", Style::default().fg(Color::Yellow)));
        }

        if group.filters.is_empty() {
            let is_selected = app.mode == Mode::Filter && app.selected_group == gi;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            spans.push(Span::styled("(empty)", style));
        } else {
            for (fi, filter) in group.filters.iter().enumerate() {
                if fi > 0 {
                    spans.push(Span::styled(" OR ", Style::default().fg(Color::Green)));
                }

                let is_selected = app.mode == Mode::Filter
                    && app.selected_group == gi
                    && app.selected_filter == fi;

                let base_style = if filter.enabled {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::CROSSED_OUT)
                };

                let style = if is_selected {
                    base_style
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::UNDERLINED)
                } else {
                    base_style
                };

                let prefix = if filter.enabled { "" } else { "⊘ " };
                spans.push(Span::styled(format!("{}{}", prefix, filter.text), style));
            }
        }
    }

    if app.filters.groups.is_empty() {
        spans.push(Span::styled(
            "No filters (press 'f' to add)",
            Style::default().fg(Color::DarkGray),
        ));
    }

    let filter_bar = Paragraph::new(Line::from(spans))
        .block(Block::default().title("Filters").borders(Borders::ALL));
    frame.render_widget(filter_bar, area);
}

fn draw_filter_input(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let prefix = if app.pending_new_group {
        "[New Group] "
    } else {
        "[Add Filter] "
    };

    let cursor_style = Style::default().bg(Color::White).fg(Color::Black);

    let line = Line::from(vec![
        Span::styled(prefix, Style::default().fg(Color::Yellow)),
        Span::styled(&app.input_buffer, Style::default().fg(Color::White)),
        Span::styled(" ", cursor_style),
    ]);

    let input_box =
        Paragraph::new(line).block(Block::default().title("Filter Input").borders(Borders::ALL));
    frame.render_widget(input_box, area);
}

fn draw_main_view(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let log_lines: Vec<Line> = app
        .filtered_logs
        .iter()
        .skip(app.scroll_offset)
        .take(area.height as usize)
        .enumerate()
        .map(|(idx, entry)| {
            let is_selected = app.scroll_offset + idx == app.selected_line;
            let base_style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let mut spans = Vec::new();

            if let Some(ts) = &entry.timestamp {
                spans.push(Span::styled(
                    ts.format("%Y-%m-%d %H:%M:%S ").to_string(),
                    base_style.fg(Color::Cyan),
                ));
            }

            spans.push(Span::styled(&entry.raw, base_style));

            Line::from(spans)
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
    frame.render_widget(main_view, area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let help_text = match app.mode {
        Mode::Normal => "h/j/k/l: Navigate | f: Add filter | F: New group+filter | Space: Toggle filter mode",
        Mode::Filter => "j/k: Move in group | h/l: Change group | d: Delete | Space: Toggle | Esc: Cancel | f: Add",
        Mode::FilterInput => "Enter: Confirm | Esc: Cancel",
        Mode::Command => "Command mode",
        Mode::DateRange => "Date range mode (unused)",
    };

    let status_text = format!(
        "{} | Line {}/{} | {}",
        app.status_message,
        app.selected_line + 1,
        app.filtered_logs.len(),
        help_text
    );

    let status_bar = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL))
        .style(match app.mode {
            Mode::Normal => Style::default(),
            Mode::Filter => Style::default().fg(Color::Cyan),
            Mode::FilterInput => Style::default().fg(Color::Yellow),
            _ => Style::default().fg(Color::Yellow),
        });
    frame.render_widget(status_bar, area);
}

fn draw_loading_screen(frame: &mut Frame, current: usize, total: usize, entries: usize) {
    let area = frame.size();

    let progress_pct = if total > 0 {
        (current * 100) / total
    } else {
        0
    };

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
