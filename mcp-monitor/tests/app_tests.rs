use mcp_common::*;
use mcp_monitor::*;

#[test]
fn test_app_creation() {
    let app = App::new();

    assert!(app.proxies.is_empty());
    assert!(app.logs.is_empty());
    assert_eq!(app.selected_index, 0);
    assert_eq!(app.viewport_offset, 0);
    assert!(app.selected_proxy.is_none());
    assert_eq!(app.proxy_selected_index, 0);
    assert_eq!(app.focus_area, FocusArea::LogView);
    assert_eq!(app.active_tab, TabType::Messages);
    assert!(app.selected_log_index.is_none());
    assert!(!app.show_detail_view);
    assert!(app.detail_word_wrap);
    assert_eq!(app.detail_scroll_offset, 0);
    assert_eq!(app.navigation_mode, NavigationMode::Follow);
    assert!(app.search_query.is_empty());
    assert!(app.search_results.is_empty());
    assert_eq!(app.search_cursor, 0);
}

#[test]
fn test_app_handle_proxy_connected() {
    let mut app = App::new();
    let proxy_id = ProxyId::new();

    let proxy_info = ProxyInfo {
        id: proxy_id.clone(),
        name: "Test Proxy".to_string(),
        listen_address: "127.0.0.1:8080".to_string(),
        target_command: vec!["python".to_string(), "server.py".to_string()],
        status: ProxyStatus::Running,
        stats: ProxyStats::default(),
    };

    app.handle_event(AppEvent::ProxyConnected(proxy_info.clone()));

    assert_eq!(app.proxies.len(), 1);
    assert!(app.proxies.contains_key(&proxy_id));
    assert_eq!(app.proxies[&proxy_id].name, "Test Proxy");
}

#[test]
fn test_app_handle_proxy_disconnected() {
    let mut app = App::new();
    let proxy_id = ProxyId::new();

    let proxy_info = ProxyInfo {
        id: proxy_id.clone(),
        name: "Test Proxy".to_string(),
        listen_address: "127.0.0.1:8080".to_string(),
        target_command: vec!["python".to_string(), "server.py".to_string()],
        status: ProxyStatus::Running,
        stats: ProxyStats::default(),
    };

    // Add proxy first
    app.handle_event(AppEvent::ProxyConnected(proxy_info));
    assert_eq!(app.proxies.len(), 1);

    // Set as selected proxy
    app.selected_proxy = Some(proxy_id.clone());

    // Disconnect proxy
    app.handle_event(AppEvent::ProxyDisconnected(proxy_id.clone()));

    assert!(app.proxies.is_empty());
    assert!(app.selected_proxy.is_none());
}

#[test]
fn test_app_handle_new_log_entry() {
    let mut app = App::new();
    let proxy_id = ProxyId::new();

    let log_entry = LogEntry::new(
        LogLevel::Request,
        "Test request".to_string(),
        proxy_id.clone(),
    );

    app.handle_event(AppEvent::NewLogEntry(log_entry.clone()));

    assert_eq!(app.logs.len(), 1);
    assert_eq!(app.logs[0].message, "Test request");
    assert_eq!(app.logs[0].proxy_id, proxy_id);
    assert_eq!(app.logs[0].level, LogLevel::Request);
}

#[test]
fn test_app_handle_stats_update() {
    let mut app = App::new();
    let proxy_id = ProxyId::new();

    // Add a proxy first
    let proxy_info = ProxyInfo {
        id: proxy_id.clone(),
        name: "Test Proxy".to_string(),
        listen_address: "127.0.0.1:8080".to_string(),
        target_command: vec!["python".to_string(), "server.py".to_string()],
        status: ProxyStatus::Running,
        stats: ProxyStats::default(),
    };
    app.handle_event(AppEvent::ProxyConnected(proxy_info));

    // Update stats
    let updated_stats = ProxyStats {
        proxy_id: proxy_id.clone(),
        total_requests: 100,
        successful_requests: 95,
        failed_requests: 5,
        active_connections: 3,
        uptime: std::time::Duration::from_secs(3600),
        bytes_transferred: 1024000,
    };

    app.handle_event(AppEvent::StatsUpdate(updated_stats.clone()));

    assert_eq!(app.proxies[&proxy_id].stats.total_requests, 100);
    assert_eq!(app.proxies[&proxy_id].stats.successful_requests, 95);
    assert_eq!(app.proxies[&proxy_id].stats.failed_requests, 5);
    assert_eq!(app.proxies[&proxy_id].stats.active_connections, 3);
    assert_eq!(app.proxies[&proxy_id].stats.bytes_transferred, 1024000);
}

#[test]
fn test_app_clear_logs() {
    let mut app = App::new();
    let proxy_id = ProxyId::new();

    // Switch to All tab to see all log types
    app.switch_tab(TabType::All);

    // Add some logs
    for i in 0..5 {
        let log_entry = LogEntry::new(LogLevel::Info, format!("Log entry {}", i), proxy_id.clone());
        app.handle_event(AppEvent::NewLogEntry(log_entry));
    }

    assert_eq!(app.logs.len(), 5);
    assert_ne!(app.selected_index, 0);

    app.clear_logs();

    assert!(app.logs.is_empty());
    assert_eq!(app.selected_index, 0);
    assert_eq!(app.viewport_offset, 0);
    assert_eq!(app.navigation_mode, NavigationMode::Follow);
}

#[test]
fn test_app_log_filtering_by_tab() {
    let mut app = App::new();
    let proxy_id = ProxyId::new();

    // Add logs of different types
    let log_types = vec![
        (LogLevel::Request, "Request log"),
        (LogLevel::Response, "Response log"),
        (LogLevel::Error, "Error log"),
        (LogLevel::Warning, "Warning log"),
        (LogLevel::Info, "Info log"),
        (LogLevel::Debug, "Debug log"),
    ];

    for (level, message) in log_types {
        let log_entry = LogEntry::new(level, message.to_string(), proxy_id.clone());
        app.handle_event(AppEvent::NewLogEntry(log_entry));
    }

    assert_eq!(app.logs.len(), 6);

    // Test All tab
    app.switch_tab(TabType::All);
    assert_eq!(app.get_filtered_logs().len(), 6);

    // Test Messages tab (Request + Response)
    app.switch_tab(TabType::Messages);
    assert_eq!(app.get_filtered_logs().len(), 2);

    // Test Errors tab (Error + Warning)
    app.switch_tab(TabType::Errors);
    assert_eq!(app.get_filtered_logs().len(), 2);

    // Test System tab (Info + Debug)
    app.switch_tab(TabType::System);
    assert_eq!(app.get_filtered_logs().len(), 2);
}

#[test]
fn test_app_log_filtering_by_proxy() {
    let mut app = App::new();
    let proxy_id1 = ProxyId::new();
    let proxy_id2 = ProxyId::new();

    // Switch to All tab to see all log types
    app.switch_tab(TabType::All);

    // Add logs from different proxies
    for i in 0..3 {
        let log_entry1 = LogEntry::new(
            LogLevel::Info,
            format!("Proxy1 log {}", i),
            proxy_id1.clone(),
        );
        let log_entry2 = LogEntry::new(
            LogLevel::Info,
            format!("Proxy2 log {}", i),
            proxy_id2.clone(),
        );
        app.handle_event(AppEvent::NewLogEntry(log_entry1));
        app.handle_event(AppEvent::NewLogEntry(log_entry2));
    }

    assert_eq!(app.logs.len(), 6);

    // No proxy filter - should see all logs
    app.selected_proxy = None;
    assert_eq!(app.get_filtered_logs().len(), 6);

    // Filter by proxy1
    app.selected_proxy = Some(proxy_id1.clone());
    let filtered = app.get_filtered_logs();
    assert_eq!(filtered.len(), 3);
    for log in filtered {
        assert_eq!(log.proxy_id, proxy_id1);
    }

    // Filter by proxy2
    app.selected_proxy = Some(proxy_id2.clone());
    let filtered = app.get_filtered_logs();
    assert_eq!(filtered.len(), 3);
    for log in filtered {
        assert_eq!(log.proxy_id, proxy_id2);
    }
}

#[test]
fn test_app_navigation_controls() {
    let mut app = App::new();
    let proxy_id = ProxyId::new();

    // Switch to All tab to see all log types
    app.switch_tab(TabType::All);

    // Add some logs
    for i in 0..10 {
        let log_entry = LogEntry::new(LogLevel::Info, format!("Log entry {}", i), proxy_id.clone());
        app.handle_event(AppEvent::NewLogEntry(log_entry));
    }

    // Should start in follow mode at the latest log
    assert_eq!(app.navigation_mode, NavigationMode::Follow);
    assert_eq!(app.selected_index, 9);

    // Scroll up should switch to navigate mode
    app.scroll_up();
    assert_eq!(app.navigation_mode, NavigationMode::Navigate);
    assert_eq!(app.selected_index, 8);

    // Scroll down
    app.scroll_down();
    assert_eq!(app.selected_index, 9);

    // Scroll to top
    app.scroll_to_top();
    assert_eq!(app.selected_index, 0);

    // Scroll to bottom
    app.scroll_to_bottom();
    assert_eq!(app.selected_index, 9);

    // Page up
    app.page_up();
    assert_eq!(app.selected_index, 0); // Can't go negative

    // Go to middle, then page down
    app.selected_index = 2;
    app.page_down();
    assert!(app.selected_index > 2);
}

#[test]
fn test_app_tab_switching() {
    let mut app = App::new();

    // Test initial state
    assert_eq!(app.active_tab, TabType::Messages);

    // Test next tab
    app.next_tab();
    assert_eq!(app.active_tab, TabType::Errors);

    app.next_tab();
    assert_eq!(app.active_tab, TabType::System);

    app.next_tab();
    assert_eq!(app.active_tab, TabType::All);

    app.next_tab();
    assert_eq!(app.active_tab, TabType::Messages); // Wrap around

    // Test previous tab
    app.prev_tab();
    assert_eq!(app.active_tab, TabType::All);

    app.prev_tab();
    assert_eq!(app.active_tab, TabType::System);

    // Test direct tab switching
    app.switch_tab(TabType::Errors);
    assert_eq!(app.active_tab, TabType::Errors);
}

#[test]
fn test_app_focus_area_switching() {
    let mut app = App::new();

    // Start with log view focus
    assert_eq!(app.focus_area, FocusArea::LogView);

    // Switch to proxy list
    app.switch_focus_to_proxy_list();
    assert_eq!(app.focus_area, FocusArea::ProxyList);

    // Switch back to logs
    app.switch_focus_to_logs();
    assert_eq!(app.focus_area, FocusArea::LogView);
}

#[test]
fn test_app_proxy_selection() {
    let mut app = App::new();
    let proxy_id1 = ProxyId::new();
    let proxy_id2 = ProxyId::new();

    // Add two proxies
    let proxy_info1 = ProxyInfo {
        id: proxy_id1.clone(),
        name: "Proxy A".to_string(),
        listen_address: "127.0.0.1:8080".to_string(),
        target_command: vec!["python".to_string(), "server1.py".to_string()],
        status: ProxyStatus::Running,
        stats: ProxyStats::default(),
    };
    let proxy_info2 = ProxyInfo {
        id: proxy_id2.clone(),
        name: "Proxy B".to_string(),
        listen_address: "127.0.0.1:8081".to_string(),
        target_command: vec!["python".to_string(), "server2.py".to_string()],
        status: ProxyStatus::Running,
        stats: ProxyStats::default(),
    };

    app.handle_event(AppEvent::ProxyConnected(proxy_info1));
    app.handle_event(AppEvent::ProxyConnected(proxy_info2));

    // Test proxy navigation
    assert_eq!(app.proxy_selected_index, 0);

    app.proxy_scroll_down();
    assert_eq!(app.proxy_selected_index, 1);

    app.proxy_scroll_up();
    assert_eq!(app.proxy_selected_index, 0);

    // Test proxy selection
    app.select_current_proxy();
    assert!(app.selected_proxy.is_some());

    // Clear selection
    app.clear_proxy_selection();
    assert!(app.selected_proxy.is_none());
}

#[test]
fn test_app_search_functionality() {
    let mut app = App::new();
    let proxy_id = ProxyId::new();

    // Switch to All tab to see all log types
    app.switch_tab(TabType::All);

    // Add some logs with searchable content
    let log_messages = vec![
        "User login successful",
        "Database connection established",
        "Error: User not found",
        "Processing user request",
        "Login attempt failed",
    ];

    for message in log_messages {
        let log_entry = LogEntry::new(LogLevel::Info, message.to_string(), proxy_id.clone());
        app.handle_event(AppEvent::NewLogEntry(log_entry));
    }

    assert_eq!(app.logs.len(), 5);

    // Enter search mode
    app.enter_search_mode();
    assert_eq!(app.navigation_mode, NavigationMode::Search);
    assert!(app.search_query.is_empty());
    assert!(app.search_results.is_empty());

    // Type search query
    for c in "user".chars() {
        app.search_input_char(c);
    }
    assert_eq!(app.search_query, "user");

    // Should find 3 matches (case insensitive)
    let search_filtered = app.get_search_filtered_logs();
    assert_eq!(search_filtered.len(), 3);

    // Test search cursor movement
    app.search_cursor_left();
    assert_eq!(app.search_cursor, 3);

    app.search_cursor_right();
    assert_eq!(app.search_cursor, 4);

    app.search_cursor_home();
    assert_eq!(app.search_cursor, 0);

    app.search_cursor_end();
    assert_eq!(app.search_cursor, 4);

    // Test backspace
    app.search_backspace();
    assert_eq!(app.search_query, "use");
    assert_eq!(app.search_cursor, 3);

    // Confirm search results
    app.confirm_search_results();
    assert_eq!(app.navigation_mode, NavigationMode::SearchResults);

    // Exit search mode
    app.exit_search_mode();
    assert_eq!(app.navigation_mode, NavigationMode::Navigate);
    assert!(app.search_query.is_empty());
    assert!(app.search_results.is_empty());
}

#[test]
fn test_app_log_detail_view() {
    let mut app = App::new();
    let proxy_id = ProxyId::new();

    // Add a log entry with JSON content
    let json_content = r#"{"method": "test", "params": {"key": "value"}}"#;
    let log_entry = LogEntry::new(
        LogLevel::Request,
        json_content.to_string(),
        proxy_id.clone(),
    );
    app.handle_event(AppEvent::NewLogEntry(log_entry));

    // Select the log
    app.select_log_at_cursor();
    assert!(app.selected_log_index.is_some());

    // Show detail view
    app.show_selected_log_detail();
    assert!(app.show_detail_view);

    // Test word wrap toggle
    assert!(app.detail_word_wrap);
    app.toggle_word_wrap();
    assert!(!app.detail_word_wrap);

    // Test scrolling
    assert_eq!(app.detail_scroll_offset, 0);
    app.detail_scroll_down();
    assert!(app.detail_scroll_offset > 0);

    app.detail_scroll_up();
    // Should go back down (saturating_sub)

    // Hide detail view
    app.hide_detail_view();
    assert!(!app.show_detail_view);
    assert!(app.selected_log_index.is_none());
    assert_eq!(app.detail_scroll_offset, 0);
}

#[test]
fn test_app_total_stats() {
    let mut app = App::new();
    let proxy_id1 = ProxyId::new();
    let proxy_id2 = ProxyId::new();

    // Add two proxies with different stats
    let proxy_info1 = ProxyInfo {
        id: proxy_id1.clone(),
        name: "Proxy 1".to_string(),
        listen_address: "127.0.0.1:8080".to_string(),
        target_command: vec!["python".to_string(), "server1.py".to_string()],
        status: ProxyStatus::Running,
        stats: ProxyStats {
            proxy_id: proxy_id1.clone(),
            total_requests: 100,
            successful_requests: 95,
            failed_requests: 5,
            active_connections: 2,
            uptime: std::time::Duration::from_secs(3600),
            bytes_transferred: 1024000,
        },
    };

    let proxy_info2 = ProxyInfo {
        id: proxy_id2.clone(),
        name: "Proxy 2".to_string(),
        listen_address: "127.0.0.1:8081".to_string(),
        target_command: vec!["python".to_string(), "server2.py".to_string()],
        status: ProxyStatus::Running,
        stats: ProxyStats {
            proxy_id: proxy_id2.clone(),
            total_requests: 50,
            successful_requests: 48,
            failed_requests: 2,
            active_connections: 1,
            uptime: std::time::Duration::from_secs(1800),
            bytes_transferred: 512000,
        },
    };

    app.handle_event(AppEvent::ProxyConnected(proxy_info1));
    app.handle_event(AppEvent::ProxyConnected(proxy_info2));

    let total_stats = app.total_stats();
    assert_eq!(total_stats.total_requests, 150);
    assert_eq!(total_stats.successful_requests, 143);
    assert_eq!(total_stats.failed_requests, 7);
    assert_eq!(total_stats.active_connections, 3);
    assert_eq!(total_stats.bytes_transferred, 1536000);
}

#[test]
fn test_app_log_size_limit() {
    let mut app = App::new();
    let proxy_id = ProxyId::new();

    // Add more than the max log limit (10,000 entries)
    for i in 0..10005 {
        let log_entry = LogEntry::new(LogLevel::Info, format!("Log entry {}", i), proxy_id.clone());
        app.handle_event(AppEvent::NewLogEntry(log_entry));
    }

    // Should be limited to 10,000 entries
    assert_eq!(app.logs.len(), 10000);

    // The first 5 entries should have been removed, so log should start with "Log entry 5"
    assert!(app.logs[0].message.starts_with("Log entry 5"));
    assert!(app
        .logs
        .last()
        .unwrap()
        .message
        .starts_with("Log entry 10004"));
}
