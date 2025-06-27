use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use mcp_common::IpcServer;
use ratatui::prelude::*;
use std::io;
use std::time::Duration;
use tokio::sync::mpsc;
// Remove unused tracing imports that interfere with TUI

mod app;
mod ui;

// Export for testing and internal use
pub use app::{App, AppEvent, FocusArea, NavigationMode, TabType};

pub struct MonitorArgs {
    pub ipc_socket: String,
    pub verbose: bool,
}

pub async fn run_monitor_app(args: MonitorArgs) -> Result<()> {
    // Initialize tracing to write to a file instead of stdout/stderr to avoid TUI interference
    let log_level = if args.verbose { "debug" } else { "info" };

    if args.verbose {
        // Only initialize file logging if verbose is requested
        use std::fs::OpenOptions;

        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/mcp-monitor.log")
            .unwrap_or_else(|_| std::fs::File::create("/dev/null").unwrap());

        tracing_subscriber::fmt()
            .with_env_filter(format!(
                "mcp_monitor={},mcp_common={}",
                log_level, log_level
            ))
            .with_writer(log_file)
            .init();
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let app = App::new();

    // Channel for IPC events
    let (event_tx, event_rx) = mpsc::channel(100);

    // Start IPC server in background
    let ipc_socket_path = args.ipc_socket.clone();
    tokio::spawn(async move {
        let _ = run_ipc_server(&ipc_socket_path, event_tx).await;
        // Remove error logging to avoid TUI interference
    });

    // Run the app
    let result = run_app(&mut terminal, app, event_rx).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_ipc_server(socket_path: &str, event_tx: mpsc::Sender<AppEvent>) -> Result<()> {
    let server = IpcServer::bind(socket_path).await?;
    // Remove logging that interferes with TUI

    loop {
        match server.accept().await {
            Ok(mut connection) => {
                // Remove "New proxy connected" log
                let tx = event_tx.clone();

                tokio::spawn(async move {
                    loop {
                        match connection.receive_message().await {
                            Ok(Some(envelope)) => {
                                let event = match envelope.message {
                                    mcp_common::IpcMessage::ProxyStarted(info) => {
                                        AppEvent::ProxyConnected(info)
                                    }
                                    mcp_common::IpcMessage::ProxyStopped(id) => {
                                        AppEvent::ProxyDisconnected(id)
                                    }
                                    mcp_common::IpcMessage::LogEntry(entry) => {
                                        AppEvent::NewLogEntry(entry)
                                    }
                                    mcp_common::IpcMessage::StatsUpdate(stats) => {
                                        AppEvent::StatsUpdate(stats)
                                    }
                                    _ => continue,
                                };

                                if tx.send(event).await.is_err() {
                                    // Remove error logging
                                    break;
                                }
                            }
                            Ok(None) => {
                                // Remove "Proxy disconnected" log
                                break;
                            }
                            Err(_e) => {
                                // Remove error logging
                                break;
                            }
                        }
                    }
                });
            }
            Err(_e) => {
                // Remove error logging
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    mut event_rx: mpsc::Receiver<AppEvent>,
) -> Result<()> {
    let mut last_tick = std::time::Instant::now();
    let tick_rate = Duration::from_millis(250);

    loop {
        // Draw UI
        terminal.draw(|f| ui::draw(f, &mut app))?;

        // Handle events
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.show_help_dialog {
                        // Handle help dialog keyboard events
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('?') => app.show_help_dialog = false,
                            _ => {}
                        }
                    } else if app.show_detail_view {
                        // Handle detail view keyboard events
                        match key.code {
                            KeyCode::Esc => app.hide_detail_view(),
                            KeyCode::Char('w') | KeyCode::Char('W') => app.toggle_word_wrap(),
                            KeyCode::Up => app.detail_scroll_up(),
                            KeyCode::Down => app.detail_scroll_down(),
                            KeyCode::PageUp => {
                                for _ in 0..10 {
                                    app.detail_scroll_up();
                                }
                            }
                            KeyCode::PageDown => {
                                for _ in 0..10 {
                                    app.detail_scroll_down();
                                }
                            }
                            KeyCode::Home => app.detail_scroll_offset = 0,
                            KeyCode::End => app.detail_scroll_offset = 1000, // Large number to scroll to bottom
                            _ => {}
                        }
                    } else if app.navigation_mode == NavigationMode::Search {
                        // Handle search mode keyboard events
                        match key.code {
                            KeyCode::Esc => app.exit_search_mode(),
                            KeyCode::Char(c) => app.search_input_char(c),
                            KeyCode::Backspace => app.search_backspace(),
                            KeyCode::Delete => app.search_delete(),
                            KeyCode::Left => app.search_cursor_left(),
                            KeyCode::Right => app.search_cursor_right(),
                            KeyCode::Home => app.search_cursor_home(),
                            KeyCode::End => app.search_cursor_end(),
                            KeyCode::Up => app.scroll_up(),
                            KeyCode::Down => app.scroll_down(),
                            KeyCode::PageUp => app.page_up(),
                            KeyCode::PageDown => app.page_down(),
                            KeyCode::Enter => {
                                // Confirm search results and switch to navigate mode while keeping results
                                app.confirm_search_results();
                            }
                            _ => {}
                        }
                    } else {
                        // Handle main view keyboard events
                        match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                break
                            }
                            KeyCode::Char('c') => app.clear_logs(),
                            KeyCode::Char('r') => app.refresh(),
                            KeyCode::Left => app.switch_focus_to_proxy_list(),
                            KeyCode::Right => app.switch_focus_to_logs(),
                            KeyCode::Up => match app.focus_area {
                                FocusArea::ProxyList => app.proxy_scroll_up(),
                                FocusArea::LogView => app.scroll_up(),
                            },
                            KeyCode::Down => match app.focus_area {
                                FocusArea::ProxyList => app.proxy_scroll_down(),
                                FocusArea::LogView => app.scroll_down(),
                            },
                            KeyCode::PageUp => {
                                if app.focus_area == FocusArea::LogView {
                                    app.page_up();
                                }
                            }
                            KeyCode::PageDown => {
                                if app.focus_area == FocusArea::LogView {
                                    app.page_down();
                                }
                            }
                            KeyCode::Home => {
                                if app.focus_area == FocusArea::LogView {
                                    app.scroll_to_top();
                                }
                            }
                            KeyCode::End => {
                                if app.focus_area == FocusArea::LogView {
                                    app.scroll_to_bottom();
                                }
                            }
                            KeyCode::Esc => match app.focus_area {
                                FocusArea::ProxyList => app.clear_proxy_selection(),
                                FocusArea::LogView => app.exit_navigation_mode(),
                            },
                            KeyCode::Tab => app.next_tab(),
                            KeyCode::BackTab => app.prev_tab(),
                            KeyCode::Char('1') => app.switch_tab(TabType::All),
                            KeyCode::Char('2') => app.switch_tab(TabType::Messages),
                            KeyCode::Char('3') => app.switch_tab(TabType::Errors),
                            KeyCode::Char('4') => app.switch_tab(TabType::System),
                            KeyCode::Char('/') => {
                                if app.focus_area == FocusArea::LogView {
                                    app.enter_search_mode();
                                }
                            }
                            KeyCode::Enter => match app.focus_area {
                                FocusArea::ProxyList => app.select_current_proxy(),
                                FocusArea::LogView => {
                                    app.select_log_at_cursor();
                                    app.show_selected_log_detail();
                                }
                            },
                            KeyCode::Char('?') => app.show_help_dialog = true,
                            _ => {}
                        }
                    }
                }
            }
        }

        // Handle IPC events
        while let Ok(event) = event_rx.try_recv() {
            app.handle_event(event);
        }

        // Tick
        if last_tick.elapsed() >= tick_rate {
            app.tick();
            last_tick = std::time::Instant::now();
        }
    }

    Ok(())
}
