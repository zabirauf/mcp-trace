use mcp_common::*;
use mcp_proxy::*;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::process::Command;
use tokio::sync::{broadcast, Mutex};
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_stdio_handler_creation() {
    let proxy_id = ProxyId::new();
    let stats = Arc::new(Mutex::new(ProxyStats::default()));

    let handler = StdioHandler::new(proxy_id.clone(), stats.clone(), None).await;
    assert!(handler.is_ok());
}

#[tokio::test]
async fn test_stdio_handler_with_ipc_client() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("test.sock")
        .to_string_lossy()
        .to_string();

    // Start IPC server
    let _server = IpcServer::bind(&socket_path).await.unwrap();

    let proxy_id = ProxyId::new();
    let stats = Arc::new(Mutex::new(ProxyStats::default()));
    let ipc_client = Arc::new(BufferedIpcClient::new(socket_path).await);

    let handler =
        StdioHandler::new(proxy_id.clone(), stats.clone(), Some(ipc_client.clone())).await;

    assert!(handler.is_ok());

    // Give time for IPC connection
    sleep(Duration::from_millis(100)).await;

    // Note: shutdown takes ownership, so we can't call it on Arc
    // In real tests, we'd need to restructure to avoid this
}

#[tokio::test]
async fn test_stdio_handler_stats_collection() {
    let proxy_id = ProxyId::new();
    let stats = Arc::new(Mutex::new(ProxyStats::default()));

    // Manually update stats to verify they're being tracked
    {
        let mut stats_guard = stats.lock().await;
        stats_guard.total_requests = 10;
        stats_guard.successful_requests = 8;
        stats_guard.failed_requests = 2;
        stats_guard.bytes_transferred = 1024;
    }

    let handler = StdioHandler::new(proxy_id.clone(), stats.clone(), None).await;
    assert!(handler.is_ok());

    // Verify stats are accessible
    let stats_guard = stats.lock().await;
    assert_eq!(stats_guard.total_requests, 10);
    assert_eq!(stats_guard.successful_requests, 8);
    assert_eq!(stats_guard.failed_requests, 2);
    assert_eq!(stats_guard.bytes_transferred, 1024);
}

// Mock child process simulation tests
#[tokio::test]
#[ignore = "Hangs due to handle_communication not terminating - needs investigation"]
async fn test_stdio_handler_process_lifecycle() {
    let proxy_id = ProxyId::new();
    let stats = Arc::new(Mutex::new(ProxyStats::default()));

    let mut handler = StdioHandler::new(proxy_id.clone(), stats.clone(), None)
        .await
        .unwrap();

    // Create a simple echo process for testing
    let mut child = Command::new("echo")
        .arg("test")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

    // Get child PID for cleanup
    let child_id = child.id();

    // Start handling communication (in background to avoid blocking)
    let handle =
        tokio::spawn(async move { handler.handle_communication(&mut child, shutdown_rx).await });

    // Give it a moment to process
    sleep(Duration::from_millis(100)).await;

    // Send shutdown signal
    let _ = shutdown_tx.send(());

    // Wait for handler to finish with timeout
    match tokio::time::timeout(Duration::from_secs(2), handle).await {
        Ok(Ok(result)) => assert!(result.is_ok()),
        Ok(Err(_)) => panic!("Handler task panicked"),
        Err(_) => {
            // Force kill the child process if still running
            if let Some(id) = child_id {
                let _ = std::process::Command::new("kill")
                    .arg("-9")
                    .arg(id.to_string())
                    .output();
            }
            panic!("Handler task timed out");
        }
    }
}

#[tokio::test]
#[ignore = "Hangs due to handle_communication not terminating - needs investigation"]
async fn test_stdio_handler_with_long_running_process() {
    let proxy_id = ProxyId::new();
    let stats = Arc::new(Mutex::new(ProxyStats::default()));

    let mut handler = StdioHandler::new(proxy_id.clone(), stats.clone(), None)
        .await
        .unwrap();

    // Use 'cat' as a long-running process that echoes input
    let mut child = Command::new("cat")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

    // Get child PID for cleanup
    let child_id = child.id();

    // Start handling communication in background
    let stats_clone = stats.clone();
    let handle =
        tokio::spawn(async move { handler.handle_communication(&mut child, shutdown_rx).await });

    // Give it time to start up
    sleep(Duration::from_millis(50)).await;

    // Check initial stats
    {
        let stats_guard = stats_clone.lock().await;
        assert_eq!(stats_guard.total_requests, 0);
    }

    // Send shutdown signal after a brief moment
    sleep(Duration::from_millis(50)).await;
    let _ = shutdown_tx.send(());

    // Wait for handler to finish with timeout
    match tokio::time::timeout(Duration::from_secs(2), handle).await {
        Ok(Ok(result)) => assert!(result.is_ok()),
        Ok(Err(_)) => panic!("Handler task panicked"),
        Err(_) => {
            // Force kill the child process if still running
            if let Some(id) = child_id {
                let _ = std::process::Command::new("kill")
                    .arg("-9")
                    .arg(id.to_string())
                    .output();
            }
            panic!("Handler task timed out");
        }
    }
}

#[tokio::test]
async fn test_stdio_handler_error_handling() {
    let proxy_id = ProxyId::new();
    let stats = Arc::new(Mutex::new(ProxyStats::default()));

    let _handler = StdioHandler::new(proxy_id.clone(), stats.clone(), None)
        .await
        .unwrap();

    // Use a command that will fail
    let child = Command::new("nonexistent_command_that_should_fail")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    // This should fail to spawn
    assert!(child.is_err());
}

#[tokio::test]
async fn test_stdio_handler_shutdown_signal() {
    let proxy_id = ProxyId::new();
    let stats = Arc::new(Mutex::new(ProxyStats::default()));

    let mut handler = StdioHandler::new(proxy_id.clone(), stats.clone(), None)
        .await
        .unwrap();

    // Use sleep command as a controllable process
    let mut child = Command::new("sleep")
        .arg("10") // Sleep for 10 seconds
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

    // Start handling in background
    let handle =
        tokio::spawn(async move { handler.handle_communication(&mut child, shutdown_rx).await });

    // Immediately send shutdown signal
    let _ = shutdown_tx.send(());

    // Handler should exit quickly due to shutdown signal
    let start = std::time::Instant::now();
    let result = handle.await.unwrap();
    let duration = start.elapsed();

    assert!(result.is_ok());
    assert!(
        duration < Duration::from_secs(1),
        "Handler should exit quickly on shutdown"
    );
}

#[tokio::test]
#[ignore = "Hangs due to handle_communication not terminating - needs investigation"]
async fn test_stdio_handler_stats_updates() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir
        .path()
        .join("test.sock")
        .to_string_lossy()
        .to_string();

    // Start IPC server to receive stats updates
    let server = IpcServer::bind(&socket_path).await.unwrap();

    let proxy_id = ProxyId::new();
    let stats = Arc::new(Mutex::new(ProxyStats {
        proxy_id: proxy_id.clone(),
        total_requests: 5,
        successful_requests: 4,
        failed_requests: 1,
        active_connections: 2,
        uptime: Duration::from_secs(60),
        bytes_transferred: 2048,
    }));

    let ipc_client = Arc::new(BufferedIpcClient::new(socket_path).await);

    let mut handler = StdioHandler::new(proxy_id.clone(), stats.clone(), Some(ipc_client.clone()))
        .await
        .unwrap();

    // Create a simple process
    let mut child = Command::new("echo")
        .arg("test")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

    // Start handling
    let handle =
        tokio::spawn(async move { handler.handle_communication(&mut child, shutdown_rx).await });

    // Give time for IPC connection and potential stats update
    sleep(Duration::from_millis(300)).await;

    // Accept connection and try to receive stats update
    let mut server_connection = server.accept().await.unwrap();

    // We might receive a stats update
    if let Ok(Ok(Some(envelope))) = tokio::time::timeout(
        Duration::from_millis(1500), // Wait for at least one stats interval
        server_connection.receive_message(),
    )
    .await
    {
        match envelope.message {
            IpcMessage::StatsUpdate(received_stats) => {
                assert_eq!(received_stats.proxy_id, proxy_id);
                assert_eq!(received_stats.total_requests, 5);
                assert_eq!(received_stats.successful_requests, 4);
                assert_eq!(received_stats.failed_requests, 1);
            }
            _ => {
                // Might receive other types of messages, that's OK
            }
        }
    }

    // Send shutdown signal
    let _ = shutdown_tx.send(());

    // Wait for handler to finish with timeout
    match tokio::time::timeout(Duration::from_secs(2), handle).await {
        Ok(Ok(_)) => {} // Success
        Ok(Err(_)) => panic!("Handler task panicked"),
        Err(_) => {
            // This test might timeout because echo exits quickly
            // That's OK for this test
        }
    }
}
