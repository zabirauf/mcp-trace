use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use mcp_common::{IpcServer};
use ratatui::{prelude::*};
use std::io;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info};

mod app;
mod ui;

use app::{App, AppEvent, TabType};

pub struct MonitorArgs {
    pub ipc_socket: String,
    pub verbose: bool,
}

pub async fn run_monitor_app(args: MonitorArgs) -> Result<()> {
    // Initialize tracing
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("mcp_monitor={},mcp_common={}", log_level, log_level))
        .init();

    info!("Starting MCP Monitor");
    info!("IPC socket: {}", args.ipc_socket);

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
        if let Err(e) = run_ipc_server(&ipc_socket_path, event_tx).await {
            error!("IPC server error: {}", e);
        }
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
    info!("IPC server listening on: {}", socket_path);

    loop {
        match server.accept().await {
            Ok(mut connection) => {
                info!("New proxy connected");
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
                                
                                if let Err(e) = tx.send(event).await {
                                    error!("Failed to send app event: {}", e);
                                    break;
                                }
                            }
                            Ok(None) => {
                                info!("Proxy disconnected");
                                break;
                            }
                            Err(e) => {
                                error!("Failed to receive IPC message: {}", e);
                                break;
                            }
                        }
                    }
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
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
                    if app.show_detail_view {
                        // Handle detail view keyboard events
                        match key.code {
                            KeyCode::Esc => app.hide_detail_view(),
                            KeyCode::Char('w') | KeyCode::Char('W') => app.toggle_word_wrap(),
                            _ => {}
                        }
                    } else {
                        // Handle main view keyboard events
                        match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                            KeyCode::Char('c') => app.clear_logs(),
                            KeyCode::Char('r') => app.refresh(),
                            KeyCode::Up => app.scroll_up(),
                            KeyCode::Down => app.scroll_down(),
                            KeyCode::PageUp => app.page_up(),
                            KeyCode::PageDown => app.page_down(),
                            KeyCode::Home => app.scroll_to_top(),
                            KeyCode::End => app.scroll_to_bottom(),
                            KeyCode::Tab => app.next_tab(),
                            KeyCode::BackTab => app.prev_tab(),
                            KeyCode::Char('1') => app.switch_tab(TabType::All),
                            KeyCode::Char('2') => app.switch_tab(TabType::Messages),
                            KeyCode::Char('3') => app.switch_tab(TabType::Errors),
                            KeyCode::Char('4') => app.switch_tab(TabType::System),
                            KeyCode::Enter => {
                                app.select_log_at_cursor();
                                app.show_selected_log_detail();
                            },
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