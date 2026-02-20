use crate::app::{App, LoadingStatus, Mode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

/// Calculate how many visual lines a text will occupy when wrapped.
fn count_visual_lines(text_width: usize, viewport_width: usize) -> usize {
    if viewport_width == 0 || text_width == 0 {
        return 1;
    }
    // Ceiling division: (text_width + viewport_width - 1) / viewport_width
    ((text_width + viewport_width - 1) / viewport_width).max(1)
}

/// Main draw function that routes to appropriate screen based on app state.
pub fn draw(frame: &mut Frame, app: &mut App) {
    // Check for loaded logs first
    app.check_for_loaded_logs();

    if let LoadingStatus::Loading { current, total } = &app.loading_status {
        draw_loading_screen(frame, *current, *total, app.total_lines());
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(match app.mode {
            Mode::FilterInput => vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ],
            Mode::Command => vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ],
            _ => vec![
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ],
        })
        .split(frame.size());

    draw_filter_bar(frame, app, chunks[0]);

    let main_chunk;
    let status_chunk;

    match app.mode {
        Mode::FilterInput => {
            draw_filter_input(frame, app, chunks[1]);
            main_chunk = chunks[2];
            status_chunk = chunks[3];
        }
        Mode::Command => {
            draw_command_input(frame, app, chunks[1]);
            main_chunk = chunks[2];
            status_chunk = chunks[3];
        }
        _ => {
            main_chunk = chunks[1];
            status_chunk = chunks[2];
        }
    }

    draw_main_view(frame, app, main_chunk);
    draw_status_bar(frame, app, status_chunk);
}

fn draw_filter_bar(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let mut spans: Vec<Span> = Vec::new();

    for (gi, group) in app.filters.groups().iter().enumerate() {
        if gi > 0 {
            spans.push(Span::styled(" AND ", Style::default().fg(Color::Yellow)));
        }

        if group.filters().is_empty() {
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
            for (fi, filter) in group.filters().iter().enumerate() {
                if fi > 0 {
                    spans.push(Span::styled(" OR ", Style::default().fg(Color::Green)));
                }

                let is_selected = app.mode == Mode::Filter
                    && app.selected_group == gi
                    && app.selected_filter == fi;

                let base_style = if filter.enabled() {
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

                let prefix = if filter.enabled() { "" } else { "⊘ " };
                spans.push(Span::styled(format!("{}{}", prefix, filter.text()), style));
            }
        }
    }

    if app.filters.groups().is_empty() {
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

fn draw_command_input(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let cursor_style = Style::default().bg(Color::White).fg(Color::Black);

    let line = Line::from(vec![
        Span::styled(":", Style::default().fg(Color::Magenta)),
        Span::styled(&app.input_buffer, Style::default().fg(Color::White)),
        Span::styled(" ", cursor_style),
    ]);

    let input_box =
        Paragraph::new(line).block(Block::default().title("Command").borders(Borders::ALL));
    frame.render_widget(input_box, area);
}

fn draw_main_view(frame: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let inner_area = area.inner(&Margin {
        vertical: 1,
        horizontal: 1,
    });

    let content_height = inner_area.height as usize;
    let viewport_width = inner_area.width as usize;
    app.viewport_height.set(content_height);
    app.viewport_width.set(viewport_width);

    // Update visual cache viewport settings
    if app.visual_cache().viewport_width() != viewport_width {
        app.visual_cache_mut().set_viewport_width(viewport_width);
    }

    // Calculate how many entries fit in the viewport, accounting for wrap mode
    let mut entries_to_take = 0usize;
    let mut total_visual_lines = 0usize;

    for idx in app.scroll_offset..app.filtered_len() {
        if let Some(mmap_str) = app.get_filtered_entry(idx) {
            let text = mmap_str.as_str_lossy();
            let ts_len = app
                .get_filtered_timestamp(idx)
                .as_ref()
                .map(|_| 20)
                .unwrap_or(0);
            let text_width = ts_len + text.chars().count();

            let visual_lines = if app.wrap_mode {
                count_visual_lines(text_width, viewport_width)
            } else {
                1
            };

            if total_visual_lines + visual_lines > content_height {
                break;
            }

            total_visual_lines += visual_lines;
            entries_to_take += 1;
        }
    }

    // Ensure we take at least 1 entry if there are any
    if entries_to_take == 0 && app.filtered_len() > app.scroll_offset {
        entries_to_take = 1;
    }

    let log_lines: Vec<Line> = (app.scroll_offset..app.scroll_offset + entries_to_take)
        .filter_map(|idx| {
            app.get_filtered_entry(idx).map(|mmap_str| {
                let is_selected = idx == app.selected_line;

                // Selection takes precedence - set background
                let base_bg = if is_selected {
                    Some(Color::DarkGray)
                } else {
                    None
                };

                // Get line color from config (for foreground)
                let line_text = mmap_str.as_str_lossy();
                let line_fg_color = app.get_line_color(&line_text);

                let mut spans = Vec::new();

                // Add timestamp if available - always cyan
                if let Some(ts) = app.get_filtered_timestamp(idx) {
                    let ts_style = match base_bg {
                        Some(bg) => Style::default().fg(Color::Cyan).bg(bg),
                        None => Style::default().fg(Color::Cyan),
                    };
                    spans.push(Span::styled(
                        ts.format("%Y-%m-%d %H:%M:%S ").to_string(),
                        ts_style,
                    ));
                }

                // Add the log line text with optional color from config
                let text_style = match (line_fg_color, base_bg) {
                    (Some(fg), Some(bg)) => Style::default().fg(fg).bg(bg),
                    (Some(fg), None) => Style::default().fg(fg),
                    (None, Some(bg)) => Style::default().bg(bg),
                    (None, None) => Style::default(),
                };
                spans.push(Span::styled(line_text.to_string(), text_style));

                Line::from(spans)
            })
        })
        .collect();

    // Calculate approximate max line width for scrollbar
    let max_line_width = if let Some(storage) = &app.storage {
        storage
            .iter()
            .take(1000) // Sample first 1000 lines
            .map(|mmap_str| {
                let text = mmap_str.as_str_lossy();
                text.chars().count()
            })
            .max()
            .unwrap_or(0)
    } else {
        0
    };

    let wrap_indicator = if app.wrap_mode { "[WRAP]" } else { "[nowrap]" };
    let title = format!(
        "Logs ({} total, {} filtered) {} [vw:{}]",
        app.total_lines(),
        app.filtered_len(),
        wrap_indicator,
        inner_area.width
    );

    let mut main_view = Paragraph::new(log_lines)
        .block(Block::default().title(title).borders(Borders::ALL))
        .scroll((0, app.horizontal_scroll as u16));

    if app.wrap_mode {
        main_view = main_view.wrap(Wrap { trim: true });
    }

    frame.render_widget(main_view, area);

    // Fast scrollbar calculation - use entry counts, not visual lines
    let total_entries = app.filtered_len();
    let scroll_position = app.scroll_offset;

    let show_vertical = total_entries > content_height;
    let show_horizontal = !app.wrap_mode && max_line_width > viewport_width;

    if show_vertical {
        let vertical_scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .track_symbol(Some("│"))
            .end_symbol(Some("▼"));

        let mut v_scroll_state = ScrollbarState::new(total_entries)
            .viewport_content_length(content_height)
            .position(scroll_position);

        frame.render_stateful_widget(vertical_scrollbar, area, &mut v_scroll_state);
    }

    if show_horizontal {
        // Render horizontal scrollbar on area that excludes vertical scrollbar space
        let h_area = if show_vertical {
            ratatui::layout::Rect {
                x: area.x,
                y: area.y + area.height.saturating_sub(1),
                width: area.width.saturating_sub(1),
                height: 1,
            }
        } else {
            area
        };

        let horizontal_scrollbar = Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
            .begin_symbol(Some("◄"))
            .track_symbol(Some("─"))
            .end_symbol(Some("►"));

        let mut h_scroll_state = ScrollbarState::new(max_line_width)
            .viewport_content_length(viewport_width)
            .position(app.horizontal_scroll);

        frame.render_stateful_widget(horizontal_scrollbar, h_area, &mut h_scroll_state);
    }
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let mode_name = match app.mode {
        Mode::Normal => "CONTENT",
        Mode::Filter => "FILTER",
        Mode::FilterInput => "INPUT",
        Mode::Command => "COMMAND",
        Mode::DateRange => "DATE",
    };

    let help_text = match app.mode {
        Mode::Normal => "j/k: Scroll | h/l: H-scroll | w: Wrap | g/G: Top/Bottom | t: Filter mode | q: Quit",
        Mode::Filter => "j/k: Select filter | h/l: Switch group | f/F: Add filter | d: Delete | Space: Toggle | t/Esc: Content mode",
        Mode::FilterInput => "Enter: Confirm | Esc: Cancel",
        Mode::Command => "Enter: Execute | Esc: Cancel",
        Mode::DateRange => "Date range mode (unused)",
    };

    let mode_style = match app.mode {
        Mode::Normal => Style::default().fg(Color::Green),
        Mode::Filter => Style::default().fg(Color::Cyan),
        Mode::FilterInput => Style::default().fg(Color::Yellow),
        Mode::Command => Style::default().fg(Color::Magenta),
        Mode::DateRange => Style::default().fg(Color::Red),
    };

    let status_text = if app.status_message.is_empty() {
        format!(
            "[{}] Line {}/{} | {}",
            mode_name,
            app.selected_line + 1,
            app.filtered_len(),
            help_text
        )
    } else {
        format!("[{}] {}", mode_name, app.status_message)
    };

    let status_bar = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL))
        .style(mode_style);
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
