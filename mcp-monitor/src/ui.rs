use mcp_common::{LogLevel, ProxyStatus};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::Title, *},
};

use crate::app::{App, TabType};

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
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
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
}

fn draw_proxy_list(f: &mut Frame, app: &App, area: Rect) {
    let proxies = app.get_proxy_list();
    
    let items: Vec<ListItem> = proxies
        .iter()
        .map(|proxy| {
            let status_symbol = match proxy.status {
                ProxyStatus::Running => "🟢",
                ProxyStatus::Starting => "🟡",
                ProxyStatus::Stopped => "🔴",
                ProxyStatus::Error(_) => "❌",
            };
            
            let text = format!(
                "{} {} ({})",
                status_symbol,
                proxy.name,
                proxy.stats.total_requests
            );
            
            ListItem::new(text)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("MCP Proxies")
                .border_set(border::ROUNDED),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">");

    f.render_widget(list, area);
}

fn draw_stats(f: &mut Frame, app: &App, area: Rect) {
    let total_stats = app.total_stats();
    let proxy_count = app.proxies.len();
    
    let stats_text = vec![
        Line::from(format!("Proxies: {}", proxy_count)),
        Line::from(format!("Total Requests: {}", total_stats.total_requests)),
        Line::from(format!("Successful: {}", total_stats.successful_requests)),
        Line::from(format!("Failed: {}", total_stats.failed_requests)),
        Line::from(format!("Active Connections: {}", total_stats.active_connections)),
        Line::from(format!("Bytes Transferred: {}", format_bytes(total_stats.bytes_transferred))),
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
        let tab_name = match tab {
            TabType::All => "All",
            TabType::Messages => "Messages",
            TabType::Errors => "Errors",
            TabType::System => "System",
        };
        
        let count = app.get_tab_log_count(tab);
        let tab_text = format!("{} ({})", tab_name, count);
        
        if tab == app.active_tab {
            Line::from(Span::styled(
                format!(" {} ", tab_text),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            ))
        } else {
            Line::from(Span::styled(
                format!(" {} ", tab_text),
                Style::default().fg(Color::Gray)
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

fn draw_logs(f: &mut Frame, app: &App, area: Rect) {
    let visible_logs = app.get_visible_logs(area.height as usize);
    
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
            let proxy_name = app.proxies
                .get(&log.proxy_id)
                .map(|p| p.name.as_str())
                .unwrap_or("unknown");
            
            let text = vec![
                Line::from(vec![
                    Span::styled(
                        format!("{} [{}] ", level_symbol, timestamp),
                        Style::default().fg(Color::Gray),
                    ),
                    Span::styled(
                        format!("[{}] ", proxy_name),
                        Style::default().fg(Color::Magenta),
                    ),
                    Span::styled(
                        &log.message,
                        Style::default().fg(level_color),
                    ),
                ]),
            ];
            
            ListItem::new(text)
        })
        .collect();

    let filtered_count = app.get_filtered_logs().len();
    let scroll_position = if filtered_count > 0 { app.scroll_offset + 1 } else { 0 };
    
    let logs_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Title::from("Logs").alignment(Alignment::Center))
                .title(Title::from(format!("({}/{})", scroll_position, filtered_count)).alignment(Alignment::Right).position(block::Position::Bottom))
                .border_set(border::ROUNDED),
        );

    f.render_widget(logs_list, area);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from("q: Quit | c: Clear logs | r: Refresh | ↑↓: Scroll | PgUp/PgDn: Page | Home/End: Top/Bottom"),
        Line::from("Tab/Shift+Tab: Switch tabs | 1-4: Direct tab selection"),
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