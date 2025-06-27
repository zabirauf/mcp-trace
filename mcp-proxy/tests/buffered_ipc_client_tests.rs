use mcp_common::*;
use mcp_proxy::BufferedIpcClient;
use tempfile::tempdir;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_buffered_client_creation() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("test.sock")
        .to_string_lossy()
        .to_string();

    let client = BufferedIpcClient::new(socket_path).await;

    // Should be able to create client even when server doesn't exist yet
    // (it will buffer messages until connection is established)

    // Test sending a message (should be buffered)
    let proxy_id = ProxyId::new();
    let log_entry = LogEntry::new(LogLevel::Info, "Test message".to_string(), proxy_id);
    let message = IpcMessage::LogEntry(log_entry);

    let result = client.send(message).await;
    assert!(result.is_ok());

    client.shutdown().await;
}

#[tokio::test]
async fn test_buffered_client_with_server() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("test.sock")
        .to_string_lossy()
        .to_string();

    // Start server first
    let server = IpcServer::bind(&socket_path).await.unwrap();

    // Create client
    let client = BufferedIpcClient::new(socket_path.clone()).await;

    // Give client time to connect
    sleep(Duration::from_millis(200)).await;

    // Send a message
    let proxy_id = ProxyId::new();
    let log_entry = LogEntry::new(LogLevel::Request, "Test request".to_string(), proxy_id);
    let message = IpcMessage::LogEntry(log_entry.clone());

    client.send(message).await.unwrap();

    // Accept connection and receive message
    let mut server_connection = server.accept().await.unwrap();
    let received_envelope = server_connection.receive_message().await.unwrap().unwrap();

    match received_envelope.message {
        IpcMessage::LogEntry(entry) => {
            assert_eq!(entry.message, log_entry.message);
            assert_eq!(entry.proxy_id, log_entry.proxy_id);
        }
        _ => panic!("Expected LogEntry message"),
    }

    client.shutdown().await;
}

#[tokio::test]
async fn test_buffered_client_reconnection() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("test.sock")
        .to_string_lossy()
        .to_string();

    // Create client without server (will buffer messages)
    let client = BufferedIpcClient::new(socket_path.clone()).await;

    // Send messages while server is down (should be buffered)
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

    for message in &messages {
        client.send(message.clone()).await.unwrap();
    }

    // Start server (client should reconnect and flush buffered messages)
    let server = IpcServer::bind(&socket_path).await.unwrap();

    // Give client time to reconnect and flush
    sleep(Duration::from_millis(500)).await;

    // Accept connection and receive all buffered messages
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

    client.shutdown().await;
}

#[tokio::test]
async fn test_buffered_client_multiple_messages() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("test.sock")
        .to_string_lossy()
        .to_string();

    let server = IpcServer::bind(&socket_path).await.unwrap();
    let client = BufferedIpcClient::new(socket_path.clone()).await;

    // Give client time to connect
    sleep(Duration::from_millis(200)).await;

    let proxy_id = ProxyId::new();
    let num_messages = 100;

    // Send many messages rapidly
    for i in 0..num_messages {
        let message = IpcMessage::LogEntry(LogEntry::new(
            LogLevel::Info,
            format!("Message {}", i),
            proxy_id.clone(),
        ));
        client.send(message).await.unwrap();
    }

    // Accept connection and receive all messages
    let mut server_connection = server.accept().await.unwrap();
    let mut received_count = 0;

    while received_count < num_messages {
        if let Some(envelope) = server_connection.receive_message().await.unwrap() {
            match envelope.message {
                IpcMessage::LogEntry(entry) => {
                    assert!(entry.message.starts_with("Message "));
                    assert_eq!(entry.proxy_id, proxy_id);
                    received_count += 1;
                }
                _ => panic!("Expected LogEntry message"),
            }
        }
    }

    assert_eq!(received_count, num_messages);
    client.shutdown().await;
}

#[tokio::test]
async fn test_buffered_client_connection_failure_recovery() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("test.sock")
        .to_string_lossy()
        .to_string();

    // Start server
    let server = IpcServer::bind(&socket_path).await.unwrap();
    let client = BufferedIpcClient::new(socket_path.clone()).await;

    // Give client time to connect
    sleep(Duration::from_millis(200)).await;

    let proxy_id = ProxyId::new();

    // Send a message successfully
    let message1 = IpcMessage::LogEntry(LogEntry::new(
        LogLevel::Info,
        "Before disconnect".to_string(),
        proxy_id.clone(),
    ));
    client.send(message1).await.unwrap();

    // Accept and verify first message
    let mut server_connection = server.accept().await.unwrap();
    let envelope = server_connection.receive_message().await.unwrap().unwrap();
    match envelope.message {
        IpcMessage::LogEntry(entry) => {
            assert_eq!(entry.message, "Before disconnect");
        }
        _ => panic!("Expected LogEntry message"),
    }

    // Simulate server disconnect by dropping server and connection
    drop(server_connection);
    drop(server);

    // Send messages while server is down (should be buffered)
    let message2 = IpcMessage::LogEntry(LogEntry::new(
        LogLevel::Warning,
        "During disconnect".to_string(),
        proxy_id.clone(),
    ));
    client.send(message2).await.unwrap();

    // Restart server
    let server = IpcServer::bind(&socket_path).await.unwrap();

    // Give client time to reconnect
    sleep(Duration::from_millis(500)).await;

    // Send another message after reconnection
    let message3 = IpcMessage::LogEntry(LogEntry::new(
        LogLevel::Error,
        "After reconnect".to_string(),
        proxy_id.clone(),
    ));
    client.send(message3).await.unwrap();

    // Accept reconnection and verify messages
    let mut server_connection = server.accept().await.unwrap();

    // Should receive the buffered message first
    let envelope = server_connection.receive_message().await.unwrap().unwrap();
    match envelope.message {
        IpcMessage::LogEntry(entry) => {
            assert_eq!(entry.message, "During disconnect");
        }
        _ => panic!("Expected LogEntry message"),
    }

    // Then the new message
    let envelope = server_connection.receive_message().await.unwrap().unwrap();
    match envelope.message {
        IpcMessage::LogEntry(entry) => {
            assert_eq!(entry.message, "After reconnect");
        }
        _ => panic!("Expected LogEntry message"),
    }

    client.shutdown().await;
}
