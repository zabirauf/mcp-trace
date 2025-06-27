use mcp_common::*;
use tempfile::tempdir;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_basic_ipc_communication() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("basic.sock")
        .to_string_lossy()
        .to_string();

    // Start IPC server
    let server = IpcServer::bind(&socket_path).await.unwrap();

    // Create client and send message
    let proxy_id = ProxyId::new();
    let log_entry = LogEntry::new(
        LogLevel::Request,
        "Test request".to_string(),
        proxy_id.clone(),
    );
    let test_message = IpcMessage::LogEntry(log_entry.clone());

    let client_task = tokio::spawn(async move {
        let mut client = IpcClient::connect(&socket_path).await.unwrap();
        client.send(test_message).await.unwrap();
    });

    // Server receives message
    let mut connection = server.accept().await.unwrap();
    let received_envelope = connection.receive_message().await.unwrap().unwrap();

    // Verify message content
    match received_envelope.message {
        IpcMessage::LogEntry(entry) => {
            assert_eq!(entry.message, log_entry.message);
            assert_eq!(entry.proxy_id, proxy_id);
        }
        _ => panic!("Expected LogEntry message"),
    }

    client_task.await.unwrap();
}

#[tokio::test]
async fn test_multiple_messages_sequence() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("sequence.sock")
        .to_string_lossy()
        .to_string();

    let server = IpcServer::bind(&socket_path).await.unwrap();

    let proxy_id = ProxyId::new();
    let num_messages = 10;

    let client_task = tokio::spawn(async move {
        let mut client = IpcClient::connect(&socket_path).await.unwrap();

        for i in 0..num_messages {
            let log_entry =
                LogEntry::new(LogLevel::Info, format!("Message {}", i), proxy_id.clone());
            client.send(IpcMessage::LogEntry(log_entry)).await.unwrap();
        }
    });

    // Server receives all messages
    let mut connection = server.accept().await.unwrap();
    let mut received_messages = Vec::new();

    for _ in 0..num_messages {
        if let Some(envelope) = connection.receive_message().await.unwrap() {
            received_messages.push(envelope);
        }
    }

    client_task.await.unwrap();

    // Verify all messages received in order
    assert_eq!(received_messages.len(), num_messages);

    for (i, envelope) in received_messages.iter().enumerate() {
        match &envelope.message {
            IpcMessage::LogEntry(entry) => {
                assert_eq!(entry.message, format!("Message {}", i));
                // Note: Can't compare proxy_id here as it was moved into async closure
            }
            _ => panic!("Expected LogEntry message"),
        }
    }
}

#[tokio::test]
async fn test_proxy_stats_updates() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("stats.sock")
        .to_string_lossy()
        .to_string();

    let server = IpcServer::bind(&socket_path).await.unwrap();
    let proxy_id = ProxyId::new();

    let client_task = tokio::spawn(async move {
        let mut client = IpcClient::connect(&socket_path).await.unwrap();

        // Send stats updates
        for i in 1..=5 {
            let stats = ProxyStats {
                proxy_id: proxy_id.clone(),
                total_requests: i * 10,
                successful_requests: i * 9,
                failed_requests: i,
                active_connections: 1,
                uptime: Duration::from_secs(i * 60),
                bytes_transferred: i * 1024,
            };

            client.send(IpcMessage::StatsUpdate(stats)).await.unwrap();
            sleep(Duration::from_millis(10)).await;
        }
    });

    let mut connection = server.accept().await.unwrap();
    let mut stats_updates = Vec::new();

    for _ in 0..5 {
        if let Some(envelope) = connection.receive_message().await.unwrap() {
            match envelope.message {
                IpcMessage::StatsUpdate(stats) => {
                    stats_updates.push(stats);
                }
                _ => panic!("Expected StatsUpdate message"),
            }
        }
    }

    client_task.await.unwrap();

    // Verify stats progression
    assert_eq!(stats_updates.len(), 5);

    for (i, stats) in stats_updates.iter().enumerate() {
        let expected_requests = (i + 1) as u64 * 10;
        assert_eq!(stats.total_requests, expected_requests);
        // Note: Can't compare proxy_id here as it was moved into async closure
    }
}

#[tokio::test]
async fn test_concurrent_clients() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("concurrent.sock")
        .to_string_lossy()
        .to_string();

    let server = IpcServer::bind(&socket_path).await.unwrap();

    let num_clients = 3;
    let messages_per_client = 5;

    // Start multiple clients
    let mut client_tasks = Vec::new();

    for client_id in 0..num_clients {
        let socket_path_clone = socket_path.clone();
        let task = tokio::spawn(async move {
            let mut client = IpcClient::connect(&socket_path_clone).await.unwrap();
            let proxy_id = ProxyId::new();

            for msg_id in 0..messages_per_client {
                let log_entry = LogEntry::new(
                    LogLevel::Info,
                    format!("Client {} Message {}", client_id, msg_id),
                    proxy_id.clone(),
                );
                client.send(IpcMessage::LogEntry(log_entry)).await.unwrap();
                sleep(Duration::from_millis(5)).await;
            }

            proxy_id
        });
        client_tasks.push(task);
    }

    // Accept all connections and collect messages
    let mut all_connections = Vec::new();
    for _ in 0..num_clients {
        let connection = server.accept().await.unwrap();
        all_connections.push(connection);
    }

    // Collect all messages
    let mut total_received = 0;
    let expected_total = num_clients * messages_per_client;

    while total_received < expected_total {
        for connection in &mut all_connections {
            match tokio::time::timeout(Duration::from_millis(50), connection.receive_message())
                .await
            {
                Ok(Ok(Some(_envelope))) => {
                    total_received += 1;
                }
                _ => {} // Timeout or error, continue
            }
        }
        sleep(Duration::from_millis(1)).await;
    }

    // Wait for all clients to finish
    for task in client_tasks {
        task.await.unwrap();
    }

    assert_eq!(total_received, expected_total);
}

#[tokio::test]
async fn test_connection_recovery() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("recovery.sock")
        .to_string_lossy()
        .to_string();

    let server = IpcServer::bind(&socket_path).await.unwrap();
    let proxy_id = ProxyId::new();

    // First connection - send some messages then disconnect
    {
        let mut client = IpcClient::connect(&socket_path).await.unwrap();

        for i in 0..3 {
            let log_entry = LogEntry::new(
                LogLevel::Info,
                format!("First connection message {}", i),
                proxy_id.clone(),
            );
            client.send(IpcMessage::LogEntry(log_entry)).await.unwrap();
        }

        // Client drops here
    }

    // Process first connection
    let mut connection1 = server.accept().await.unwrap();
    let mut first_messages = 0;

    loop {
        match connection1.receive_message().await {
            Ok(Some(_envelope)) => {
                first_messages += 1;
            }
            Ok(None) | Err(_) => break,
        }
    }

    assert_eq!(first_messages, 3);

    // Second connection
    {
        let mut client = IpcClient::connect(&socket_path).await.unwrap();

        for i in 0..3 {
            let log_entry = LogEntry::new(
                LogLevel::Info,
                format!("Second connection message {}", i),
                proxy_id.clone(),
            );
            client.send(IpcMessage::LogEntry(log_entry)).await.unwrap();
        }
    }

    // Process second connection
    let mut connection2 = server.accept().await.unwrap();
    let mut second_messages = 0;

    for _ in 0..3 {
        if let Some(_envelope) = connection2.receive_message().await.unwrap() {
            second_messages += 1;
        }
    }

    assert_eq!(second_messages, 3);
}

#[tokio::test]
async fn test_message_types() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("types.sock")
        .to_string_lossy()
        .to_string();

    let server = IpcServer::bind(&socket_path).await.unwrap();
    let proxy_id = ProxyId::new();

    let client_task = tokio::spawn(async move {
        let mut client = IpcClient::connect(&socket_path).await.unwrap();

        // Send all types of messages
        let proxy_info = ProxyInfo {
            id: proxy_id.clone(),
            name: "Test Proxy".to_string(),
            listen_address: "127.0.0.1:8080".to_string(),
            target_command: vec!["python".to_string(), "server.py".to_string()],
            status: ProxyStatus::Running,
            stats: ProxyStats::default(),
        };

        client
            .send(IpcMessage::ProxyStarted(proxy_info))
            .await
            .unwrap();

        let log_entry = LogEntry::new(
            LogLevel::Request,
            "Test request".to_string(),
            proxy_id.clone(),
        );
        client.send(IpcMessage::LogEntry(log_entry)).await.unwrap();

        let stats = ProxyStats {
            proxy_id: proxy_id.clone(),
            total_requests: 1,
            successful_requests: 1,
            failed_requests: 0,
            active_connections: 1,
            uptime: Duration::from_secs(60),
            bytes_transferred: 256,
        };
        client.send(IpcMessage::StatsUpdate(stats)).await.unwrap();

        client
            .send(IpcMessage::ProxyStopped(proxy_id))
            .await
            .unwrap();
    });

    let mut connection = server.accept().await.unwrap();
    let mut message_types = Vec::new();

    for _ in 0..4 {
        if let Some(envelope) = connection.receive_message().await.unwrap() {
            match envelope.message {
                IpcMessage::ProxyStarted(_) => message_types.push("ProxyStarted"),
                IpcMessage::ProxyStopped(_) => message_types.push("ProxyStopped"),
                IpcMessage::LogEntry(_) => message_types.push("LogEntry"),
                IpcMessage::StatsUpdate(_) => message_types.push("StatsUpdate"),
                _ => message_types.push("Other"),
            }
        }
    }

    client_task.await.unwrap();

    assert_eq!(message_types.len(), 4);
    assert_eq!(message_types[0], "ProxyStarted");
    assert_eq!(message_types[1], "LogEntry");
    assert_eq!(message_types[2], "StatsUpdate");
    assert_eq!(message_types[3], "ProxyStopped");
}
