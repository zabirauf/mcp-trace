use mcp_common::{LogLevel, ProxyStatus};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::Title, *},
};

use crate::app::{App, FocusArea, NavigationMode, TabType};

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.size();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(0)])
        .split(size);

    // Left panel: Proxy list and stats
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(8)])
        .split(chunks[0]);

    // Right panel: Tabs, Logs, Help
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(chunks[1]);

    // Draw proxy list
    draw_proxy_list(f, app, left_chunks[0]);

    // Draw stats
    draw_stats(f, app, left_chunks[1]);

    // Draw tabs
    draw_tabs(f, app, right_chunks[0]);

    // Draw logs
    draw_logs(f, app, right_chunks[1]);

    // Draw help
    draw_help(f, right_chunks[2]);

    // Draw detail view overlay if active
    if app.show_detail_view {
        draw_detail_view(f, app, size);
    }

    // Draw search dialog overlay if in search mode
    if app.navigation_mode == NavigationMode::Search {
        draw_search_dialog(f, app, size);
    }

    // Draw help dialog overlay if active
    if app.show_help_dialog {
        draw_help_dialog(f, app, size);
    }
}

fn draw_proxy_list(f: &mut Frame, app: &App, area: Rect) {
    let proxies = app.get_proxy_list();

    let items: Vec<ListItem> = proxies
        .iter()
        .enumerate()
        .map(|(_index, proxy)| {
            let status_symbol = match proxy.status {
                ProxyStatus::Running => "🟢",
                ProxyStatus::Starting => "🟡",
                ProxyStatus::Stopped => "🔴",
                ProxyStatus::Error(_) => "❌",
            };

            // Add filter indicator if this proxy is selected for filtering
            let filter_indicator = if app.selected_proxy.as_ref() == Some(&proxy.id) {
                " [FILTERED]"
            } else {
                ""
            };

            let text = format!(
                "{} {} ({}){}",
                status_symbol, proxy.name, proxy.stats.total_requests, filter_indicator
            );

            // Highlight the filtered proxy
            if app.selected_proxy.as_ref() == Some(&proxy.id) {
                ListItem::new(text).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ListItem::new(text)
            }
        })
        .collect();

    // Create focus indicator for the title - keep it shorter
    let (title_text, title_color) = match app.focus_area {
        FocusArea::ProxyList => ("Proxies *", Color::Green),
        FocusArea::LogView => ("Proxies", Color::Gray),
    };

    // Add concise instructions for the narrow panel
    let instructions = if app.focus_area == FocusArea::ProxyList {
        "↑↓ Enter Esc"
    } else {
        "← to focus"
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Title::from(Span::styled(
                    title_text,
                    Style::default()
                        .fg(title_color)
                        .add_modifier(Modifier::BOLD),
                )))
                .title(
                    Title::from(instructions)
                        .alignment(Alignment::Left)
                        .position(block::Position::Bottom),
                )
                .border_set(border::ROUNDED),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">");

    // Create selection state for the proxy list
    let mut state = ListState::default();
    if app.focus_area == FocusArea::ProxyList && !proxies.is_empty() {
        state.select(Some(
            app.proxy_selected_index
                .min(proxies.len().saturating_sub(1)),
        ));
    }

    f.render_stateful_widget(list, area, &mut state);
}

fn draw_stats(f: &mut Frame, app: &App, area: Rect) {
    let total_stats = app.total_stats();
    let proxy_count = app.proxies.len();

    let stats_text = vec![
        Line::from(format!("Proxies: {}", proxy_count)),
        Line::from(format!("Total Requests: {}", total_stats.total_requests)),
        Line::from(format!("Successful: {}", total_stats.successful_requests)),
        Line::from(format!("Failed: {}", total_stats.failed_requests)),
        Line::from(format!(
            "Active Connections: {}",
            total_stats.active_connections
        )),
        Line::from(format!(
            "Bytes Transferred: {}",
            format_bytes(total_stats.bytes_transferred)
        )),
    ];

    let paragraph = Paragraph::new(stats_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Statistics")
                .border_set(border::ROUNDED),
        )
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let tabs: Vec<Line> = vec![
        TabType::All,
        TabType::Messages,
        TabType::Errors,
        TabType::System,
    ]
    .iter()
    .map(|&tab| {
        let (tab_name, emoji, fallback) = match tab {
            TabType::All => ("All", "📊", "A"),
            TabType::Messages => ("Messages", "💬", "M"),
            TabType::Errors => ("Errors", "❗", "E"),
            TabType::System => ("System", "⚡", "S"),
        };

        // Use emoji with fallback for limited terminals
        let tab_icon = if std::env::var("TERM")
            .unwrap_or_default()
            .contains("256color")
            || std::env::var("COLORTERM").is_ok()
        {
            emoji
        } else {
            fallback
        };

        let count = app.get_tab_log_count(tab);
        let tab_text = format!("{} {} ({})", tab_icon, tab_name, count);

        if tab == app.active_tab {
            Line::from(Span::styled(
                format!(" {} ", tab_text),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::LightBlue)
                    .add_modifier(Modifier::BOLD),
            ))
        } else {
            Line::from(Span::styled(
                format!(" {} ", tab_text),
                Style::default().fg(Color::Gray),
            ))
        }
    })
    .collect();

    let tabs_widget = Tabs::new(tabs)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Filters")
                .border_set(border::ROUNDED),
        )
        .style(Style::default())
        .highlight_style(Style::default().fg(Color::White))
        .select(match app.active_tab {
            TabType::All => 0,
            TabType::Messages => 1,
            TabType::Errors => 2,
            TabType::System => 3,
        });

    f.render_widget(tabs_widget, area);
}

fn draw_logs(f: &mut Frame, app: &mut App, area: Rect) {
    // Prepare viewport first
    let visible_height = area.height.saturating_sub(2) as usize;
    app.prepare_viewport(visible_height);

    // Get data for rendering
    let visible_logs = app.get_visible_logs(visible_height);
    let relative_selection = app.get_relative_selection(visible_height);
    let filtered_count = app.get_search_filtered_logs().len();
    let display_position = if filtered_count > 0 {
        app.selected_index + 1
    } else {
        0
    };

    let items: Vec<ListItem> = visible_logs
        .iter()
        .map(|log| {
            let level_color = match log.level {
                LogLevel::Error => Color::Red,
                LogLevel::Warning => Color::Yellow,
                LogLevel::Info => Color::Blue,
                LogLevel::Debug => Color::Gray,
                LogLevel::Request => Color::Green,
                LogLevel::Response => Color::Cyan,
            };

            let level_symbol = match log.level {
                LogLevel::Error => "❌",
                LogLevel::Warning => "⚠️",
                LogLevel::Info => "ℹ️",
                LogLevel::Debug => "🐛",
                LogLevel::Request => "📤",
                LogLevel::Response => "📥",
            };

            let timestamp = log.timestamp.format("%H:%M:%S%.3f");
            let proxy_name = app
                .proxies
                .get(&log.proxy_id)
                .map(|p| p.name.as_str())
                .unwrap_or("unknown");

            let text = vec![Line::from(vec![
                Span::styled(
                    format!("{} [{}] ", level_symbol, timestamp),
                    Style::default().fg(Color::Gray),
                ),
                Span::styled(
                    format!("[{}] ", proxy_name),
                    Style::default().fg(Color::Magenta),
                ),
                Span::styled(&log.message, Style::default().fg(level_color)),
            ])];

            ListItem::new(text)
        })
        .collect();

    // Create mode indicator
    let (mode_text, mode_color) = match app.navigation_mode {
        NavigationMode::Follow => ("FOLLOW", Color::Green),
        NavigationMode::Navigate => ("NAVIGATE", Color::Yellow),
        NavigationMode::Search => ("SEARCH", Color::Cyan),
        NavigationMode::SearchResults => ("SEARCH RESULTS", Color::Magenta),
    };

    // Create focus indicator for logs
    let logs_title = match app.focus_area {
        FocusArea::LogView => "Logs [FOCUSED]",
        FocusArea::ProxyList => "Logs",
    };

    // Add proxy filter indication to title
    let proxy_filter_text = if let Some(ref proxy_id) = app.selected_proxy {
        if let Some(proxy) = app.proxies.get(proxy_id) {
            format!(" | Filtered by: {}", proxy.name)
        } else {
            " | Filtered".to_string()
        }
    } else {
        String::new()
    };

    // Add search query to title if in search results mode
    let search_text =
        if app.navigation_mode == NavigationMode::SearchResults && !app.search_query.is_empty() {
            format!(" | Search: \"{}\"", app.search_query)
        } else {
            String::new()
        };

    let logs_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Title::from(logs_title).alignment(Alignment::Center))
                .title(
                    Title::from(Span::styled(
                        format!("[{}]{}{}", mode_text, proxy_filter_text, search_text),
                        Style::default().fg(mode_color).add_modifier(Modifier::BOLD),
                    ))
                    .alignment(Alignment::Left),
                )
                .title(
                    Title::from(format!(
                        "({}/{}) [Enter: View Details] | →: Focus here",
                        display_position, filtered_count
                    ))
                    .alignment(Alignment::Right)
                    .position(block::Position::Bottom),
                )
                .border_set(border::ROUNDED),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">");

    let mut state = ListState::default();
    state.select(relative_selection);

    f.render_stateful_widget(logs_list, area, &mut state);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from("q/Ctrl+C: Quit | c: Clear logs | r: Refresh | ←→: Switch focus | ↑↓: Navigate | Esc: Follow/Clear filter | Enter: Select | /: Search"),
        Line::from("Tab/Shift+Tab: Switch tabs | 1-4: Direct tab selection | PgUp/PgDn: Page | Home/End: Top/Bottom"),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help")
                .border_set(border::ROUNDED),
        )
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

fn draw_detail_view(f: &mut Frame, app: &App, area: Rect) {
    // Create a centered popup that covers most of the screen
    let popup_area = centered_rect(90, 85, area);

    // Clear the background completely first
    let clear = Clear;
    f.render_widget(clear, popup_area);

    // Draw a solid background block to create visual separation
    let background = Block::default()
        .borders(Borders::ALL)
        .border_set(border::DOUBLE)
        .border_style(Style::default().fg(Color::White))
        .style(Style::default().bg(Color::Black));
    f.render_widget(background, popup_area);

    if let Some(log) = app.get_selected_log() {
        let content = app.format_log_content(log);

        // Create the main content area (with margin to avoid overlapping the border)
        let inner_area = Rect {
            x: popup_area.x + 1,
            y: popup_area.y + 1,
            width: popup_area.width.saturating_sub(2),
            height: popup_area.height.saturating_sub(2),
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(inner_area);

        // Header with log info
        let header_text = vec![Line::from(format!(
            "Log Details - {} | {} | {}",
            match log.level {
                mcp_common::LogLevel::Request => "📤 Request",
                mcp_common::LogLevel::Response => "📥 Response",
                mcp_common::LogLevel::Error => "❌ Error",
                mcp_common::LogLevel::Warning => "⚠️ Warning",
                mcp_common::LogLevel::Info => "ℹ️ Info",
                mcp_common::LogLevel::Debug => "🐛 Debug",
            },
            log.timestamp.format("%H:%M:%S%.3f"),
            log.request_id.as_deref().unwrap_or("N/A")
        ))];

        let header = Paragraph::new(header_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Detail View")
                    .border_set(border::THICK)
                    .border_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                    .style(Style::default().bg(Color::Rgb(20, 20, 20))),
            )
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);

        // Content area with word wrap toggle
        let wrap_indicator = if app.detail_word_wrap { "ON" } else { "OFF" };

        // Create text from content with proper line breaks
        let text = if app.detail_word_wrap {
            // When word wrap is on, create a single text block that will be wrapped
            Text::from(content)
        } else {
            // When word wrap is off, split into lines to preserve formatting
            let lines: Vec<Line> = content
                .lines()
                .map(|line| Line::from(line.to_string()))
                .collect();
            Text::from(lines)
        };

        let content_paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(
                        "Content [Word Wrap: {}] [W: Toggle]",
                        wrap_indicator
                    ))
                    .border_set(border::THICK)
                    .border_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                    .style(Style::default().bg(Color::Rgb(20, 20, 20))),
            )
            .style(Style::default().fg(Color::White))
            .wrap(if app.detail_word_wrap {
                Wrap { trim: true }
            } else {
                Wrap { trim: false }
            })
            .scroll((app.detail_scroll_offset, 0)); // Use scroll offset

        // Footer with controls
        let footer_text = vec![
            Line::from("ESC: Close | W: Toggle Word Wrap | ↑↓: Scroll | PgUp/PgDn: Page scroll | Home/End: Top/Bottom")
        ];

        let footer = Paragraph::new(footer_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Controls")
                    .border_set(border::THICK)
                    .border_style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                    .style(Style::default().bg(Color::Rgb(20, 20, 20))),
            )
            .style(Style::default().fg(Color::LightCyan))
            .alignment(Alignment::Center);

        f.render_widget(header, chunks[0]);
        f.render_widget(content_paragraph, chunks[1]);
        f.render_widget(footer, chunks[2]);
    }
}

// Helper function to create a centered rectangle
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

fn draw_search_dialog(f: &mut Frame, app: &App, area: Rect) {
    // Create a smaller centered dialog for search
    let dialog_area = centered_rect(60, 20, area);

    // Clear the background completely first
    let clear = Clear;
    f.render_widget(clear, dialog_area);

    // Draw a solid background block to create visual separation
    let background = Block::default()
        .borders(Borders::ALL)
        .border_set(border::DOUBLE)
        .border_style(Style::default().fg(Color::White))
        .style(Style::default().bg(Color::Black));
    f.render_widget(background, dialog_area);

    // Create layout for the dialog (with margin to avoid overlapping the border)
    let inner_area = Rect {
        x: dialog_area.x + 1,
        y: dialog_area.y + 1,
        width: dialog_area.width.saturating_sub(2),
        height: dialog_area.height.saturating_sub(2),
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(inner_area);

    // Search input field
    let search_input = format!("Search: {}", app.search_query);
    let search_paragraph = Paragraph::new(search_input.clone())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search Logs")
                .border_set(border::THICK)
                .border_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .style(Style::default().bg(Color::Rgb(20, 20, 20))),
        )
        .style(Style::default().fg(Color::White));

    // Results info
    let results_count = app.search_results.len();
    let results_text = if app.search_query.is_empty() {
        "Type to search...".to_string()
    } else if results_count == 0 {
        "No results found".to_string()
    } else {
        format!(
            "{} result{} found",
            results_count,
            if results_count == 1 { "" } else { "s" }
        )
    };

    let results_paragraph = Paragraph::new(results_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Results")
                .border_set(border::THICK)
                .border_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .style(Style::default().bg(Color::Rgb(20, 20, 20))),
        )
        .style(Style::default().fg(Color::LightYellow));

    // Instructions
    let instructions = vec![
        Line::from("ESC: Exit search | Enter: Navigate to results | ↑↓: Navigate results"),
        Line::from("Type to filter logs by message, proxy name, or log level"),
    ];

    let instructions_paragraph = Paragraph::new(instructions)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Instructions")
                .border_set(border::THICK)
                .border_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .style(Style::default().bg(Color::Rgb(20, 20, 20))),
        )
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center);

    f.render_widget(search_paragraph, chunks[0]);
    f.render_widget(results_paragraph, chunks[1]);
    f.render_widget(instructions_paragraph, chunks[2]);

    // Set cursor position in the search input
    if !app.search_query.is_empty() || app.search_cursor > 0 {
        let cursor_x = chunks[0].x + 9 + app.search_cursor as u16; // "Search: " = 8 chars + 1 for border
        let cursor_y = chunks[0].y + 1; // 1 for top border
        f.set_cursor(cursor_x, cursor_y);
    } else {
        // Position cursor after "Search: "
        let cursor_x = chunks[0].x + 9; // "Search: " = 8 chars + 1 for border
        let cursor_y = chunks[0].y + 1; // 1 for top border
        f.set_cursor(cursor_x, cursor_y);
    }
}

fn draw_help_dialog(f: &mut Frame, app: &App, area: Rect) {
    // Create a centered dialog for help
    let dialog_area = centered_rect(70, 80, area);

    // Clear the background
    let clear = Clear;
    f.render_widget(clear, dialog_area);

    // Draw background block
    let background = Block::default()
        .borders(Borders::ALL)
        .border_set(border::DOUBLE)
        .border_style(Style::default().fg(Color::White))
        .style(Style::default().bg(Color::Black));
    f.render_widget(background, dialog_area);

    // Create inner area with margin
    let inner_area = Rect {
        x: dialog_area.x + 1,
        y: dialog_area.y + 1,
        width: dialog_area.width.saturating_sub(2),
        height: dialog_area.height.saturating_sub(2),
    };

    // Build context-aware help content
    let mut help_sections = vec![];

    // Global shortcuts
    help_sections.push(Line::from(Span::styled(
        "━━━ Global Shortcuts ━━━",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
    help_sections.push(Line::from(""));
    help_sections.push(Line::from(vec![
        Span::styled(
            "q/Ctrl+C",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  Quit application"),
    ]));
    help_sections.push(Line::from(vec![
        Span::styled(
            "?",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("         Show this help dialog"),
    ]));
    help_sections.push(Line::from(vec![
        Span::styled(
            "c",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("         Clear all logs"),
    ]));
    help_sections.push(Line::from(vec![
        Span::styled(
            "r",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("         Refresh proxy connections"),
    ]));
    help_sections.push(Line::from(vec![
        Span::styled(
            "/",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("         Open search dialog"),
    ]));
    help_sections.push(Line::from(vec![
        Span::styled(
            "←/→",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("       Switch focus between panels"),
    ]));
    help_sections.push(Line::from(""));

    // Tab navigation
    help_sections.push(Line::from(Span::styled(
        "━━━ Tab Navigation ━━━",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
    help_sections.push(Line::from(""));
    help_sections.push(Line::from(vec![
        Span::styled(
            "Tab",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("       Next tab"),
    ]));
    help_sections.push(Line::from(vec![
        Span::styled(
            "Shift+Tab",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Previous tab"),
    ]));
    help_sections.push(Line::from(vec![
        Span::styled(
            "1-4",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("       Direct tab selection (1=All, 2=Messages, 3=Errors, 4=System)"),
    ]));
    help_sections.push(Line::from(""));

    // Context-specific shortcuts
    match app.focus_area {
        FocusArea::ProxyList => {
            help_sections.push(Line::from(Span::styled(
                "━━━ Proxy List (Current Focus) ━━━",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )));
            help_sections.push(Line::from(""));
            help_sections.push(Line::from(vec![
                Span::styled(
                    "↑/↓",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("       Navigate proxy list"),
            ]));
            help_sections.push(Line::from(vec![
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("     Filter logs by selected proxy"),
            ]));
            help_sections.push(Line::from(vec![
                Span::styled(
                    "Esc",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("       Clear proxy filter"),
            ]));
        }
        FocusArea::LogView => {
            help_sections.push(Line::from(Span::styled(
                "━━━ Log View (Current Focus) ━━━",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )));
            help_sections.push(Line::from(""));
            help_sections.push(Line::from(vec![
                Span::styled(
                    "↑/↓",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("       Navigate logs"),
            ]));
            help_sections.push(Line::from(vec![
                Span::styled(
                    "PgUp/PgDn",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Page up/down"),
            ]));
            help_sections.push(Line::from(vec![
                Span::styled(
                    "Home/End",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  Jump to top/bottom"),
            ]));
            help_sections.push(Line::from(vec![
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("     View log details"),
            ]));
            help_sections.push(Line::from(vec![
                Span::styled(
                    "Esc",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("       Return to follow mode"),
            ]));
        }
    }

    help_sections.push(Line::from(""));

    // Mode-specific help
    match app.navigation_mode {
        NavigationMode::Follow => {
            help_sections.push(Line::from(Span::styled(
                "━━━ Follow Mode (Active) ━━━",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )));
            help_sections.push(Line::from(""));
            help_sections.push(Line::from(
                "Automatically scrolls to show new logs as they arrive",
            ));
            help_sections.push(Line::from("Press ↑/↓ to enter Navigate mode"));
        }
        NavigationMode::Navigate => {
            help_sections.push(Line::from(Span::styled(
                "━━━ Navigate Mode (Active) ━━━",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )));
            help_sections.push(Line::from(""));
            help_sections.push(Line::from("Manual navigation through logs"));
            help_sections.push(Line::from("Press Esc to return to Follow mode"));
        }
        NavigationMode::Search => {
            help_sections.push(Line::from(Span::styled(
                "━━━ Search Mode (Active) ━━━",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )));
            help_sections.push(Line::from(""));
            help_sections.push(Line::from("Type to filter logs"));
            help_sections.push(Line::from("Enter to navigate results, Esc to exit"));
        }
        NavigationMode::SearchResults => {
            help_sections.push(Line::from(Span::styled(
                "━━━ Search Results (Active) ━━━",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            )));
            help_sections.push(Line::from(""));
            help_sections.push(Line::from("Navigating filtered search results"));
            help_sections.push(Line::from("Press / to search again, Esc to clear"));
        }
    }

    // Special view shortcuts
    if app.show_detail_view {
        help_sections.push(Line::from(""));
        help_sections.push(Line::from(Span::styled(
            "━━━ Detail View Shortcuts ━━━",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )));
        help_sections.push(Line::from(""));
        help_sections.push(Line::from(vec![
            Span::styled(
                "W",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("         Toggle word wrap"),
        ]));
        help_sections.push(Line::from(vec![
            Span::styled(
                "↑/↓",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("       Scroll content"),
        ]));
        help_sections.push(Line::from(vec![
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("       Close detail view"),
        ]));
    }

    // Create scrollable paragraph
    let help_paragraph = Paragraph::new(help_sections)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Keyboard Shortcuts ")
                .title(
                    Title::from(" Press ESC or ? to close ")
                        .alignment(Alignment::Right)
                        .position(block::Position::Bottom),
                )
                .border_set(border::THICK)
                .border_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .style(Style::default().bg(Color::Rgb(20, 20, 20))),
        )
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left);

    f.render_widget(help_paragraph, inner_area);
}
