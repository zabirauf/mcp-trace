use mcp_common::*;
use mcp_monitor::{App, AppEvent};
use mcp_proxy::BufferedIpcClient;
use tempfile::tempdir;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_end_to_end_proxy_monitor_communication() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("e2e.sock")
        .to_string_lossy()
        .to_string();

    // Start the monitor's IPC server
    let server = IpcServer::bind(&socket_path).await.unwrap();

    // Create app instance (like the monitor would)
    let mut app = App::new();

    // Set up multiple proxies connecting to the monitor
    let num_proxies = 3;
    let mut proxy_clients = Vec::new();

    for _i in 0..num_proxies {
        let socket_path_clone = socket_path.clone();
        let proxy_client = BufferedIpcClient::new(socket_path_clone).await;
        proxy_clients.push(proxy_client);
    }

    // Give clients time to connect
    sleep(Duration::from_millis(200)).await;

    // Simulate proxy registration and activity
    let proxy_ids: Vec<ProxyId> = (0..num_proxies).map(|_| ProxyId::new()).collect();

    for (i, proxy_id) in proxy_ids.iter().enumerate() {
        // Each proxy registers itself
        let proxy_info = ProxyInfo {
            id: proxy_id.clone(),
            name: format!("Test Proxy {}", i),
            listen_address: format!("127.0.0.1:808{}", i),
            target_command: vec!["python".to_string(), format!("server{}.py", i)],
            status: ProxyStatus::Running,
            stats: ProxyStats::default(),
        };

        proxy_clients[i]
            .send(IpcMessage::ProxyStarted(proxy_info.clone()))
            .await
            .unwrap();
    }

    // Accept connections and simulate monitor processing
    let mut connections = Vec::new();
    for _ in 0..num_proxies {
        let connection = server.accept().await.unwrap();
        connections.push(connection);
    }

    // Process proxy registration messages
    for i in 0..num_proxies {
        if let Some(envelope) = connections[i].receive_message().await.unwrap() {
            match envelope.message {
                IpcMessage::ProxyStarted(proxy_info) => {
                    app.handle_event(AppEvent::ProxyConnected(proxy_info));
                }
                _ => panic!("Expected ProxyStarted message"),
            }
        }
    }

    // Verify all proxies are registered in the app
    assert_eq!(app.proxies.len(), num_proxies);
    for proxy_id in &proxy_ids {
        assert!(app.proxies.contains_key(proxy_id));
    }

    // Simulate active MCP traffic from all proxies
    for iteration in 0..5 {
        for (i, proxy_id) in proxy_ids.iter().enumerate() {
            // Simulate request/response pair
            let request = LogEntry::new(
                LogLevel::Request,
                format!(
                    "{{\"id\": \"{}\", \"method\": \"tools/list\", \"params\": {{}}}}",
                    iteration
                ),
                proxy_id.clone(),
            );

            let response = LogEntry::new(
                LogLevel::Response,
                format!(
                    "{{\"id\": \"{}\", \"result\": {{\"tools\": []}}}}",
                    iteration
                ),
                proxy_id.clone(),
            );

            proxy_clients[i]
                .send(IpcMessage::LogEntry(request))
                .await
                .unwrap();
            proxy_clients[i]
                .send(IpcMessage::LogEntry(response))
                .await
                .unwrap();

            // Update stats
            let stats = ProxyStats {
                proxy_id: proxy_id.clone(),
                total_requests: (iteration + 1) as u64,
                successful_requests: (iteration + 1) as u64,
                failed_requests: 0,
                active_connections: 1,
                uptime: Duration::from_secs((iteration + 1) * 10),
                bytes_transferred: (iteration + 1) * 256,
            };

            proxy_clients[i]
                .send(IpcMessage::StatsUpdate(stats))
                .await
                .unwrap();

            sleep(Duration::from_millis(10)).await;
        }
    }

    // Process all messages through the monitor
    let expected_messages_per_proxy = 5 * 3; // 5 iterations × (2 log entries + 1 stats update)
    let total_expected_messages = num_proxies * expected_messages_per_proxy;

    let mut total_processed = 0;

    while total_processed < total_expected_messages {
        for connection in &mut connections {
            match tokio::time::timeout(Duration::from_millis(100), connection.receive_message())
                .await
            {
                Ok(Ok(Some(envelope))) => {
                    let event = match envelope.message {
                        IpcMessage::LogEntry(entry) => AppEvent::NewLogEntry(entry),
                        IpcMessage::StatsUpdate(stats) => AppEvent::StatsUpdate(stats),
                        _ => continue,
                    };

                    app.handle_event(event);
                    total_processed += 1;
                }
                _ => {} // Timeout or error, continue
            }
        }

        sleep(Duration::from_millis(1)).await;
    }

    // Verify the monitor state
    assert_eq!(total_processed, total_expected_messages);

    // Check that logs were recorded
    // We sent 5 iterations × 2 log entries × 3 proxies = 30 log entries
    assert_eq!(app.logs.len(), 30);

    // Verify stats were updated for all proxies
    for proxy_id in &proxy_ids {
        let proxy = &app.proxies[proxy_id];
        assert_eq!(proxy.stats.total_requests, 5);
        assert_eq!(proxy.stats.successful_requests, 5);
        assert_eq!(proxy.stats.failed_requests, 0);
    }

    // Test total stats aggregation
    let total_stats = app.total_stats();
    assert_eq!(total_stats.total_requests, 15); // 5 requests × 3 proxies
    assert_eq!(total_stats.successful_requests, 15);
    assert_eq!(total_stats.failed_requests, 0);
    assert_eq!(total_stats.active_connections, 3); // 1 per proxy

    // Test log filtering by different tabs
    app.switch_tab(mcp_monitor::TabType::All);
    assert_eq!(app.get_filtered_logs().len(), 30);

    app.switch_tab(mcp_monitor::TabType::Messages);
    assert_eq!(app.get_filtered_logs().len(), 30); // All are Request/Response

    app.switch_tab(mcp_monitor::TabType::Errors);
    assert_eq!(app.get_filtered_logs().len(), 0); // No errors

    app.switch_tab(mcp_monitor::TabType::System);
    assert_eq!(app.get_filtered_logs().len(), 0); // No system logs

    // Test proxy-specific filtering
    app.switch_tab(mcp_monitor::TabType::All);
    app.selected_proxy = Some(proxy_ids[0].clone());
    let proxy_0_logs = app.get_filtered_logs();
    assert_eq!(proxy_0_logs.len(), 10); // 5 iterations × 2 log entries

    for log in proxy_0_logs {
        assert_eq!(log.proxy_id, proxy_ids[0]);
    }

    // Simulate proxy disconnection
    proxy_clients[0]
        .send(IpcMessage::ProxyStopped(proxy_ids[0].clone()))
        .await
        .unwrap();

    // Process disconnection
    if let Some(envelope) = connections[0].receive_message().await.unwrap() {
        match envelope.message {
            IpcMessage::ProxyStopped(proxy_id) => {
                app.handle_event(AppEvent::ProxyDisconnected(proxy_id));
            }
            _ => panic!("Expected ProxyStopped message"),
        }
    }

    // Verify proxy was removed
    assert_eq!(app.proxies.len(), num_proxies - 1);
    assert!(!app.proxies.contains_key(&proxy_ids[0]));

    // Verify selected proxy is cleared if it was the disconnected one
    assert!(app.selected_proxy.is_none());

    // Clean up remaining proxy clients
    for _client in proxy_clients {
        // Note: Can't call shutdown() due to ownership, but they'll be dropped
    }
}

#[tokio::test]
async fn test_error_handling_end_to_end() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("e2e_error.sock")
        .to_string_lossy()
        .to_string();

    let server = IpcServer::bind(&socket_path).await.unwrap();
    let mut app = App::new();
    app.switch_tab(mcp_monitor::TabType::All); // See all log types

    // Create proxy client
    let proxy_client = BufferedIpcClient::new(socket_path.clone()).await;

    // Give client time to connect
    sleep(Duration::from_millis(100)).await;

    let proxy_id = ProxyId::new();

    // Register proxy
    let proxy_info = ProxyInfo {
        id: proxy_id.clone(),
        name: "Error Test Proxy".to_string(),
        listen_address: "127.0.0.1:8080".to_string(),
        target_command: vec!["python".to_string(), "error_server.py".to_string()],
        status: ProxyStatus::Running,
        stats: ProxyStats::default(),
    };

    proxy_client
        .send(IpcMessage::ProxyStarted(proxy_info.clone()))
        .await
        .unwrap();

    // Accept connection
    let mut connection = server.accept().await.unwrap();

    // Process registration
    if let Some(envelope) = connection.receive_message().await.unwrap() {
        match envelope.message {
            IpcMessage::ProxyStarted(info) => {
                app.handle_event(AppEvent::ProxyConnected(info));
            }
            _ => panic!("Expected ProxyStarted"),
        }
    }

    // Simulate various error scenarios
    let error_scenarios = vec![
        (
            LogLevel::Request,
            r#"{"id": "1", "method": "invalid_method", "params": {}}"#,
        ),
        (
            LogLevel::Response,
            r#"{"id": "1", "error": {"code": -32601, "message": "Method not found"}}"#,
        ),
        (LogLevel::Error, "Connection timeout to MCP server"),
        (
            LogLevel::Warning,
            "Server response took longer than expected",
        ),
        (
            LogLevel::Request,
            r#"{"id": "2", "method": "tools/call", "params": {"name": "nonexistent"}}"#,
        ),
        (
            LogLevel::Response,
            r#"{"id": "2", "error": {"code": -32602, "message": "Invalid params"}}"#,
        ),
    ];

    for (level, message) in error_scenarios {
        let log_entry = LogEntry::new(level, message.to_string(), proxy_id.clone());
        proxy_client
            .send(IpcMessage::LogEntry(log_entry))
            .await
            .unwrap();
    }

    // Update stats to reflect errors
    let error_stats = ProxyStats {
        proxy_id: proxy_id.clone(),
        total_requests: 2,
        successful_requests: 0,
        failed_requests: 2,
        active_connections: 1,
        uptime: Duration::from_secs(300),
        bytes_transferred: 1024,
    };

    proxy_client
        .send(IpcMessage::StatsUpdate(error_stats.clone()))
        .await
        .unwrap();

    // Process all messages
    for _ in 0..7 {
        // 6 log entries + 1 stats update
        if let Some(envelope) = connection.receive_message().await.unwrap() {
            let event = match envelope.message {
                IpcMessage::LogEntry(entry) => AppEvent::NewLogEntry(entry),
                IpcMessage::StatsUpdate(stats) => AppEvent::StatsUpdate(stats),
                _ => continue,
            };
            app.handle_event(event);
        }
    }

    // Verify error logs were recorded
    assert_eq!(app.logs.len(), 6);

    // Test filtering by error types
    app.switch_tab(mcp_monitor::TabType::Errors);
    let error_logs = app.get_filtered_logs();
    assert_eq!(error_logs.len(), 2); // 1 Error + 1 Warning

    app.switch_tab(mcp_monitor::TabType::Messages);
    let message_logs = app.get_filtered_logs();
    assert_eq!(message_logs.len(), 4); // 2 Requests + 2 Responses

    // Verify stats show failures
    let proxy = &app.proxies[&proxy_id];
    assert_eq!(proxy.stats.total_requests, 2);
    assert_eq!(proxy.stats.successful_requests, 0);
    assert_eq!(proxy.stats.failed_requests, 2);

    // Search functionality with error content
    app.switch_tab(mcp_monitor::TabType::All);
    app.enter_search_mode();

    // Search for "timeout" which should only match the error log
    for c in "timeout".chars() {
        app.search_input_char(c);
    }

    let search_results = app.get_search_filtered_logs();
    assert!(search_results.len() >= 1); // Should find timeout error message

    // Verify search results contain timeout-related content
    for log in &search_results {
        assert!(
            log.message.to_lowercase().contains("timeout") || log.level == LogLevel::Error,
            "Log doesn't match search criteria: level={:?}, message='{}'",
            log.level,
            log.message
        );
    }

    app.exit_search_mode();
}

#[tokio::test]
async fn test_high_throughput_end_to_end() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("e2e_throughput.sock")
        .to_string_lossy()
        .to_string();

    let server = IpcServer::bind(&socket_path).await.unwrap();
    let mut app = App::new();
    app.switch_tab(mcp_monitor::TabType::All);

    let proxy_client = BufferedIpcClient::new(socket_path.clone()).await;

    // Give client time to connect
    sleep(Duration::from_millis(100)).await;

    let proxy_id = ProxyId::new();

    // Register proxy
    let proxy_info = ProxyInfo {
        id: proxy_id.clone(),
        name: "High Throughput Proxy".to_string(),
        listen_address: "127.0.0.1:8080".to_string(),
        target_command: vec![
            "python".to_string(),
            "high_throughput_server.py".to_string(),
        ],
        status: ProxyStatus::Running,
        stats: ProxyStats::default(),
    };

    proxy_client
        .send(IpcMessage::ProxyStarted(proxy_info.clone()))
        .await
        .unwrap();

    let mut connection = server.accept().await.unwrap();

    // Process registration
    if let Some(envelope) = connection.receive_message().await.unwrap() {
        match envelope.message {
            IpcMessage::ProxyStarted(info) => {
                app.handle_event(AppEvent::ProxyConnected(info));
            }
            _ => panic!("Expected ProxyStarted"),
        }
    }

    // Generate high volume of traffic (reduced for test performance)
    let num_requests = 100;

    // Send many requests rapidly
    for i in 0..num_requests {
        let request = LogEntry::new(
            LogLevel::Request,
            format!(
                "{{\"id\": \"{}\", \"method\": \"tools/list\", \"params\": {{}}}}",
                i
            ),
            proxy_id.clone(),
        );

        let response = LogEntry::new(
            LogLevel::Response,
            format!("{{\"id\": \"{}\", \"result\": {{\"tools\": []}}}}", i),
            proxy_id.clone(),
        );

        proxy_client
            .send(IpcMessage::LogEntry(request))
            .await
            .unwrap();
        proxy_client
            .send(IpcMessage::LogEntry(response))
            .await
            .unwrap();

        // Send stats update every 25 requests
        if i % 25 == 24 {
            let stats = ProxyStats {
                proxy_id: proxy_id.clone(),
                total_requests: (i + 1) as u64,
                successful_requests: (i + 1) as u64,
                failed_requests: 0,
                active_connections: 1,
                uptime: Duration::from_secs((i + 1) / 10),
                bytes_transferred: (i + 1) * 128,
            };
            proxy_client
                .send(IpcMessage::StatsUpdate(stats))
                .await
                .unwrap();
        }
    }

    // Process all messages
    let expected_log_messages = num_requests * 2; // request + response
    let expected_stats_messages = 4; // every 25 requests (100/25 = 4)
    let total_expected = expected_log_messages + expected_stats_messages;

    let mut processed = 0;
    let start_time = std::time::Instant::now();
    let test_timeout = Duration::from_secs(30); // Overall test timeout

    while processed < total_expected && start_time.elapsed() < test_timeout {
        match tokio::time::timeout(Duration::from_millis(100), connection.receive_message()).await {
            Ok(Ok(Some(envelope))) => {
                let event = match envelope.message {
                    IpcMessage::LogEntry(entry) => AppEvent::NewLogEntry(entry),
                    IpcMessage::StatsUpdate(stats) => AppEvent::StatsUpdate(stats),
                    _ => continue,
                };
                app.handle_event(event);
                processed += 1;
            }
            _ => {
                // Timeout or error - continue with small delay
                sleep(Duration::from_millis(10)).await;
            }
        }
    }

    // Verify high throughput was handled correctly
    assert_eq!(app.logs.len(), expected_log_messages as usize);

    // Check log size limit enforcement (should be capped at 10,000)
    assert!(app.logs.len() <= 10000);

    // Verify final stats
    let proxy = &app.proxies[&proxy_id];
    assert_eq!(proxy.stats.total_requests, num_requests as u64);
    assert_eq!(proxy.stats.successful_requests, num_requests as u64);
    assert_eq!(proxy.stats.failed_requests, 0);

    // Test that navigation still works with many logs
    app.scroll_to_top();
    assert_eq!(app.selected_index, 0);

    app.scroll_to_bottom();
    assert_eq!(app.selected_index, app.get_filtered_logs().len() - 1);

    // Test search functionality with high volume
    app.enter_search_mode();

    // Search for "tools/list" which should be in many messages
    for c in "tools".chars() {
        app.search_input_char(c);
    }

    let search_results = app.get_search_filtered_logs();
    assert!(!search_results.is_empty());

    // Results should contain "tools"
    for log in &search_results {
        assert!(log.message.to_lowercase().contains("tools"));
    }

    app.exit_search_mode();
}
