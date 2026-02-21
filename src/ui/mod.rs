use crate::app::{App, LoadingStatus, Mode};
use crate::model::filter::FilterKind;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
    },
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
            Mode::SearchInput => vec![
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
            Mode::FilterList => vec![
                Constraint::Length(3),
                Constraint::Length(12),
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
        Mode::FilterList => {
            draw_filter_list(frame, app, chunks[1]);
            main_chunk = chunks[2];
            status_chunk = chunks[3];
        }
        Mode::Command => {
            draw_command_input(frame, app, chunks[1]);
            main_chunk = chunks[2];
            status_chunk = chunks[3];
        }
        Mode::SearchInput => {
            draw_search_input(frame, app, chunks[1]);
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
    let filter_count = app.filters.len();
    let mut spans: Vec<Span> = Vec::new();

    if filter_count == 0 {
        spans.push(Span::styled(
            "No filters active",
            Style::default().fg(Color::DarkGray),
        ));
    } else {
        spans.push(Span::styled(
            format!("{} filter(s) active", filter_count),
            Style::default().fg(Color::Cyan),
        ));
    }

    let filter_bar = Paragraph::new(Line::from(spans))
        .block(Block::default().title("Filters").borders(Borders::ALL));
    frame.render_widget(filter_bar, area);
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

fn draw_search_input(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let cursor_style = Style::default().bg(Color::White).fg(Color::Black);

    let line = Line::from(vec![
        Span::styled("/", Style::default().fg(Color::Yellow)),
        Span::styled(&app.input_buffer, Style::default().fg(Color::White)),
        Span::styled(" ", cursor_style),
    ]);

    let input_box =
        Paragraph::new(line).block(Block::default().title("Search Input").borders(Borders::ALL));
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

    // Collect line data first to avoid borrow issues
    let line_data: Vec<(
        usize,
        String,
        Option<chrono::DateTime<chrono::Utc>>,
        Option<Color>,
    )> = (app.scroll_offset..app.scroll_offset + entries_to_take)
        .filter_map(|idx| {
            app.get_filtered_entry(idx).map(|mmap_str| {
                let line_text = mmap_str.as_str_lossy().to_string();
                let line_fg_color = app.get_line_color(&line_text);
                let timestamp = app.get_filtered_timestamp(idx);
                (idx, line_text, timestamp, line_fg_color)
            })
        })
        .collect();

    // Pre-compute matches for all visible lines
    let line_matches: Vec<(usize, Vec<(usize, usize)>)> = line_data
        .iter()
        .map(|(idx, _, _, _)| {
            let matches = if app.has_search() {
                app.get_line_matches(*idx)
            } else {
                Vec::new()
            };
            (*idx, matches)
        })
        .collect();

    // Build log lines with highlighting
    let log_lines: Vec<Line> = line_data
        .into_iter()
        .zip(line_matches.into_iter())
        .filter_map(
            |((idx, line_text, timestamp, line_fg_color), (_, matches))| {
                let is_selected = idx == app.selected_line;
                let is_in_selection = app.selection.contains(idx, app.selected_line);

                // Selection takes precedence - set background
                // Use DarkGray for cursor line, Gray for other selected lines
                let base_bg = if is_selected {
                    Some(Color::DarkGray)
                } else if is_in_selection {
                    Some(Color::Gray)
                } else {
                    None
                };

                let mut spans = Vec::new();

                // Add timestamp if available - always cyan
                if let Some(ts) = timestamp {
                    let ts_style = match base_bg {
                        Some(bg) => Style::default().fg(Color::Cyan).bg(bg),
                        None => Style::default().fg(Color::Cyan),
                    };
                    spans.push(Span::styled(
                        ts.format("%Y-%m-%d %H:%M:%S ").to_string(),
                        ts_style,
                    ));
                }

                if matches.is_empty() {
                    // No matches - add the whole line as one span
                    let text_style = match (line_fg_color, base_bg) {
                        (Some(fg), Some(bg)) => Style::default().fg(fg).bg(bg),
                        (Some(fg), None) => Style::default().fg(fg),
                        (None, Some(bg)) => Style::default().bg(bg),
                        (None, None) => Style::default(),
                    };
                    spans.push(Span::styled(line_text, text_style));
                } else {
                    // Split line into spans around matches
                    let line_bytes = line_text.as_bytes();
                    let mut last_end = 0;

                    for (match_start, match_end) in matches {
                        // Add text before match
                        if match_start > last_end {
                            let before_text =
                                String::from_utf8_lossy(&line_bytes[last_end..match_start]);
                            let text_style = match (line_fg_color, base_bg) {
                                (Some(fg), Some(bg)) => Style::default().fg(fg).bg(bg),
                                (Some(fg), None) => Style::default().fg(fg),
                                (None, Some(bg)) => Style::default().bg(bg),
                                (None, None) => Style::default(),
                            };
                            spans.push(Span::styled(before_text.to_string(), text_style));
                        }

                        // Add match span with highlight
                        let match_text =
                            String::from_utf8_lossy(&line_bytes[match_start..match_end]);
                        let is_current = app.is_current_match(idx, match_start);

                        let match_style = if let Some(search_config) = app.search_config() {
                            if is_current {
                                let style = search_config
                                    .current_style
                                    .fg(search_config.current_fg)
                                    .bg(search_config.current_bg);
                                if base_bg.is_some() {
                                    // Don't override selection bg
                                    let base_style = style.bg(base_bg.unwrap());
                                    base_style
                                } else {
                                    style
                                }
                            } else {
                                let style = search_config
                                    .match_style
                                    .fg(search_config.match_fg)
                                    .bg(search_config.match_bg);
                                if base_bg.is_some() {
                                    // Don't override selection bg
                                    let base_style = style.bg(base_bg.unwrap());
                                    base_style
                                } else {
                                    style
                                }
                            }
                        } else {
                            // Fallback colors
                            if is_current {
                                Style::default().fg(Color::Black).bg(Color::LightYellow)
                            } else {
                                Style::default().fg(Color::Black).bg(Color::Yellow)
                            }
                        };

                        spans.push(Span::styled(match_text.to_string(), match_style));
                        last_end = match_end;
                    }

                    // Add remaining text after last match
                    if last_end < line_bytes.len() {
                        let after_text = String::from_utf8_lossy(&line_bytes[last_end..]);
                        let text_style = match (line_fg_color, base_bg) {
                            (Some(fg), Some(bg)) => Style::default().fg(fg).bg(bg),
                            (Some(fg), None) => Style::default().fg(fg),
                            (None, Some(bg)) => Style::default().bg(bg),
                            (None, None) => Style::default(),
                        };
                        spans.push(Span::styled(after_text.to_string(), text_style));
                    }
                }

                Some(Line::from(spans))
            },
        )
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
        Mode::FilterList => "FILTERS",
        Mode::Command => "COMMAND",
        Mode::DateRange => "DATE",
        Mode::SearchInput => "SEARCH",
    };

    let help_text = match app.mode {
        Mode::Normal => "j/k: Scroll | h/l: H-scroll | w: Wrap | g/G: Top/Bottom | /: Search | n/N: Next/Prev match | q: Quit",
        Mode::FilterList => "j/k: Select filter | d: Delete | q: Close",
        Mode::Command => "Enter: Execute | Esc: Cancel",
        Mode::DateRange => "Date range mode (unused)",
        Mode::SearchInput => "Enter: Execute search | Esc: Cancel | Backspace: Delete char",
    };

    let mode_style = match app.mode {
        Mode::Normal => Style::default().fg(Color::Green),
        Mode::FilterList => Style::default().fg(Color::Cyan),
        Mode::Command => Style::default().fg(Color::Magenta),
        Mode::DateRange => Style::default().fg(Color::Red),
        Mode::SearchInput => Style::default().fg(Color::Yellow),
    };

    let status_text = if !app.status_message.is_empty() {
        format!("[{}] {}", mode_name, app.status_message)
    } else {
        // Build status parts
        let mut parts = vec![format!("[{}]", mode_name)];

        // Line position
        parts.push(format!(
            "Line {}/{}",
            app.selected_line + 1,
            app.filtered_len()
        ));

        // Search status if active
        if let Some(query) = app.get_search_query() {
            if let Some(match_display) = app.current_match_display() {
                parts.push(format!("Search: '{}' {}", query, match_display));
            } else {
                parts.push(format!("Search: '{}' (0 matches)", query));
            }
        }

        parts.push(help_text.to_string());

        parts.join(" | ")
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

/// Draw the filter list overlay
pub fn draw_filter_list(frame: &mut Frame, app: &App, area: Rect) {
    // Clear the area
    frame.render_widget(Clear, area);

    // Build the filter list content
    let filter_list = &app.filters;
    let mut lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Active Filters (", Style::default().fg(Color::Cyan)),
            Span::styled(
                filter_list.len().to_string(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(")", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
    ];

    if filter_list.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  No active filters",
            Style::default().fg(Color::DarkGray),
        )]));
    } else {
        for (idx, rule) in filter_list.iter() {
            let is_selected = idx == app.filter_list_selected;
            let kind = rule.kind();

            let kind_style = match kind {
                FilterKind::Include => Style::default().fg(Color::Green),
                FilterKind::Exclude => Style::default().fg(Color::Red),
            };

            let prefix = if is_selected { ">" } else { " " };

            let kind_text = match kind {
                FilterKind::Include => "INCLUDE",
                FilterKind::Exclude => "EXCLUDE",
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("{}{} ", prefix, idx + 1),
                    if is_selected {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    },
                ),
                Span::styled(
                    kind_text.to_string(),
                    kind_style.add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(rule.pattern(), Style::default().fg(Color::White)),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("j/k", Style::default().fg(Color::Yellow)),
        Span::raw(" navigate, "),
        Span::styled("d", Style::default().fg(Color::Yellow)),
        Span::raw(" delete, "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw("/"),
        Span::styled("Esc", Style::default().fg(Color::Yellow)),
        Span::raw(" close"),
    ]));

    let filter_block = Block::default()
        .title(" Filter List ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let filter_paragraph = Paragraph::new(lines)
        .block(filter_block)
        .alignment(Alignment::Left);

    frame.render_widget(filter_paragraph, area);
}
