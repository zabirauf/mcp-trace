use mcp_common::*;
use tempfile::tempdir;

#[tokio::test]
async fn test_ipc_server_bind_and_accept() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("test.sock")
        .to_string_lossy()
        .to_string();

    let server = IpcServer::bind(&socket_path).await.unwrap();

    // Verify socket file was created
    assert!(std::path::Path::new(&socket_path).exists());

    // Test that we can create a client connection
    let client_task = tokio::spawn(async move {
        let _client = IpcConnection::connect(&socket_path).await.unwrap();
        // Keep connection alive briefly
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    });

    // Accept the connection
    let _connection = server.accept().await.unwrap();

    // Wait for client to finish
    client_task.await.unwrap();
}

#[tokio::test]
async fn test_ipc_server_bind_removes_existing_socket() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("test.sock")
        .to_string_lossy()
        .to_string();

    // Create first server
    let _server1 = IpcServer::bind(&socket_path).await.unwrap();
    assert!(std::path::Path::new(&socket_path).exists());

    // Create second server with same path - should succeed by removing existing socket
    let _server2 = IpcServer::bind(&socket_path).await.unwrap();
    assert!(std::path::Path::new(&socket_path).exists());
}

#[tokio::test]
async fn test_ipc_connection_send_and_receive() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("test.sock")
        .to_string_lossy()
        .to_string();

    let server = IpcServer::bind(&socket_path).await.unwrap();

    // Create test message
    let proxy_id = ProxyId::new();
    let log_entry = LogEntry::new(LogLevel::Request, "Test message".to_string(), proxy_id);
    let test_message = IpcMessage::LogEntry(log_entry.clone());

    // Client task
    let test_message_clone = test_message.clone();
    let socket_path_clone = socket_path.clone();
    let client_task = tokio::spawn(async move {
        let mut client = IpcConnection::connect(&socket_path_clone).await.unwrap();
        client.send_message(test_message_clone).await.unwrap();
    });

    // Server task
    let mut server_connection = server.accept().await.unwrap();
    let received_envelope = server_connection.receive_message().await.unwrap().unwrap();

    // Verify message content
    match received_envelope.message {
        IpcMessage::LogEntry(entry) => {
            assert_eq!(entry.message, log_entry.message);
            assert_eq!(entry.proxy_id, log_entry.proxy_id);
        }
        _ => panic!("Expected LogEntry message"),
    }

    // Verify envelope has correlation ID and timestamp
    assert!(received_envelope.correlation_id.is_some());
    assert!(received_envelope.timestamp <= chrono::Utc::now());

    client_task.await.unwrap();
}

#[tokio::test]
async fn test_ipc_client_wrapper() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("test.sock")
        .to_string_lossy()
        .to_string();

    let server = IpcServer::bind(&socket_path).await.unwrap();

    let proxy_id = ProxyId::new();
    let stats = ProxyStats {
        proxy_id: proxy_id.clone(),
        total_requests: 42,
        successful_requests: 40,
        failed_requests: 2,
        active_connections: 1,
        uptime: std::time::Duration::from_secs(300),
        bytes_transferred: 1024,
    };
    let test_message = IpcMessage::StatsUpdate(stats.clone());

    // Client task using IpcClient wrapper
    let test_message_clone = test_message.clone();
    let socket_path_clone = socket_path.clone();
    let client_task = tokio::spawn(async move {
        let mut client = IpcClient::connect(&socket_path_clone).await.unwrap();
        client.send(test_message_clone).await.unwrap();
    });

    // Server receives message
    let mut server_connection = server.accept().await.unwrap();
    let received_envelope = server_connection.receive_message().await.unwrap().unwrap();

    match received_envelope.message {
        IpcMessage::StatsUpdate(received_stats) => {
            assert_eq!(received_stats.proxy_id, stats.proxy_id);
            assert_eq!(received_stats.total_requests, stats.total_requests);
        }
        _ => panic!("Expected StatsUpdate message"),
    }

    client_task.await.unwrap();
}

#[tokio::test]
async fn test_multiple_messages_in_sequence() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("test.sock")
        .to_string_lossy()
        .to_string();

    let server = IpcServer::bind(&socket_path).await.unwrap();

    let proxy_id = ProxyId::new();
    let messages = vec![
        IpcMessage::LogEntry(LogEntry::new(
            LogLevel::Info,
            "Message 1".to_string(),
            proxy_id.clone(),
        )),
        IpcMessage::LogEntry(LogEntry::new(
            LogLevel::Warning,
            "Message 2".to_string(),
            proxy_id.clone(),
        )),
        IpcMessage::LogEntry(LogEntry::new(
            LogLevel::Error,
            "Message 3".to_string(),
            proxy_id.clone(),
        )),
    ];

    // Client sends multiple messages
    let messages_clone = messages.clone();
    let socket_path_clone = socket_path.clone();
    let client_task = tokio::spawn(async move {
        let mut client = IpcConnection::connect(&socket_path_clone).await.unwrap();
        for message in messages_clone {
            client.send_message(message).await.unwrap();
        }
    });

    // Server receives all messages
    let mut server_connection = server.accept().await.unwrap();
    for i in 0..messages.len() {
        let received_envelope = server_connection.receive_message().await.unwrap().unwrap();
        match (&messages[i], &received_envelope.message) {
            (IpcMessage::LogEntry(sent), IpcMessage::LogEntry(received)) => {
                assert_eq!(sent.message, received.message);
                assert_eq!(sent.proxy_id, received.proxy_id);
            }
            _ => panic!("Message type mismatch at index {}", i),
        }
    }

    client_task.await.unwrap();
}

#[tokio::test]
async fn test_connection_closed_handling() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("test.sock")
        .to_string_lossy()
        .to_string();

    let server = IpcServer::bind(&socket_path).await.unwrap();

    // Client connects and immediately disconnects
    let socket_path_clone = socket_path.clone();
    let client_task = tokio::spawn(async move {
        let _client = IpcConnection::connect(&socket_path_clone).await.unwrap();
        // Client drops connection immediately
    });

    let mut server_connection = server.accept().await.unwrap();
    client_task.await.unwrap();

    // Try to receive from closed connection
    let result = server_connection.receive_message().await.unwrap();
    assert!(result.is_none(), "Should return None for closed connection");
}

#[tokio::test]
async fn test_invalid_socket_path() {
    // Try to bind to an invalid path
    let result = IpcServer::bind("/invalid/path/that/does/not/exist/test.sock").await;
    assert!(result.is_err(), "Should fail to bind to invalid path");

    // Try to connect to non-existent socket
    let result = IpcConnection::connect("/non/existent/socket.sock").await;
    assert!(
        result.is_err(),
        "Should fail to connect to non-existent socket"
    );
}

#[tokio::test]
async fn test_large_message_transmission() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("test.sock")
        .to_string_lossy()
        .to_string();

    let server = IpcServer::bind(&socket_path).await.unwrap();

    // Create a large message (1MB)
    let large_text = "x".repeat(1_000_000);
    let proxy_id = ProxyId::new();
    let large_message = IpcMessage::LogEntry(LogEntry::new(
        LogLevel::Response,
        large_text.clone(),
        proxy_id,
    ));

    let large_message_clone = large_message.clone();
    let socket_path_clone = socket_path.clone();
    let client_task = tokio::spawn(async move {
        let mut client = IpcConnection::connect(&socket_path_clone).await.unwrap();
        client.send_message(large_message_clone).await.unwrap();
    });

    let mut server_connection = server.accept().await.unwrap();
    let received_envelope = server_connection.receive_message().await.unwrap().unwrap();

    match received_envelope.message {
        IpcMessage::LogEntry(entry) => {
            assert_eq!(entry.message, large_text);
            assert_eq!(entry.message.len(), 1_000_000);
        }
        _ => panic!("Expected LogEntry message"),
    }

    client_task.await.unwrap();
}

#[tokio::test]
async fn test_concurrent_clients() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("test.sock")
        .to_string_lossy()
        .to_string();

    let server = IpcServer::bind(&socket_path).await.unwrap();

    let num_clients = 5;
    let proxy_id = ProxyId::new();

    // Start multiple clients concurrently
    let mut client_tasks = Vec::new();
    for i in 0..num_clients {
        let socket_path_clone = socket_path.clone();
        let proxy_id_clone = proxy_id.clone();
        let task = tokio::spawn(async move {
            let mut client = IpcConnection::connect(&socket_path_clone).await.unwrap();
            let message = IpcMessage::LogEntry(LogEntry::new(
                LogLevel::Info,
                format!("Message from client {}", i),
                proxy_id_clone,
            ));
            client.send_message(message).await.unwrap();
        });
        client_tasks.push(task);
    }

    // Accept connections and receive messages
    let mut received_messages = Vec::new();
    for _ in 0..num_clients {
        let mut connection = server.accept().await.unwrap();
        let envelope = connection.receive_message().await.unwrap().unwrap();
        received_messages.push(envelope);
    }

    // Wait for all clients to complete
    for task in client_tasks {
        task.await.unwrap();
    }

    // Verify we received all messages
    assert_eq!(received_messages.len(), num_clients);

    // Verify all messages are unique
    let mut message_texts = std::collections::HashSet::new();
    for envelope in received_messages {
        if let IpcMessage::LogEntry(entry) = envelope.message {
            assert!(
                message_texts.insert(entry.message),
                "Duplicate message received"
            );
        }
    }
}
