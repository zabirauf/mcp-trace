use chrono::Utc;
use mcp_common::*;
use serde_json;

#[test]
fn test_ipc_message_log_entry_serialization() {
    let proxy_id = ProxyId::new();
    let log_entry = LogEntry::new(LogLevel::Request, "Test log message".to_string(), proxy_id);

    let message = IpcMessage::LogEntry(log_entry.clone());

    let serialized = serde_json::to_string(&message).unwrap();
    let deserialized: IpcMessage = serde_json::from_str(&serialized).unwrap();

    match deserialized {
        IpcMessage::LogEntry(entry) => {
            assert_eq!(entry.message, log_entry.message);
            assert_eq!(entry.proxy_id, log_entry.proxy_id);
            assert_eq!(
                format!("{:?}", entry.level),
                format!("{:?}", log_entry.level)
            );
        }
        _ => panic!("Expected LogEntry message"),
    }
}

#[test]
fn test_ipc_message_stats_update_serialization() {
    let proxy_id = ProxyId::new();
    let stats = ProxyStats {
        proxy_id: proxy_id.clone(),
        total_requests: 100,
        successful_requests: 95,
        failed_requests: 5,
        active_connections: 2,
        uptime: std::time::Duration::from_secs(3600),
        bytes_transferred: 1024000,
    };

    let message = IpcMessage::StatsUpdate(stats.clone());

    let serialized = serde_json::to_string(&message).unwrap();
    let deserialized: IpcMessage = serde_json::from_str(&serialized).unwrap();

    match deserialized {
        IpcMessage::StatsUpdate(deserialized_stats) => {
            assert_eq!(deserialized_stats.proxy_id, stats.proxy_id);
            assert_eq!(deserialized_stats.total_requests, stats.total_requests);
            assert_eq!(
                deserialized_stats.successful_requests,
                stats.successful_requests
            );
            assert_eq!(deserialized_stats.failed_requests, stats.failed_requests);
            assert_eq!(
                deserialized_stats.active_connections,
                stats.active_connections
            );
            assert_eq!(
                deserialized_stats.bytes_transferred,
                stats.bytes_transferred
            );
        }
        _ => panic!("Expected StatsUpdate message"),
    }
}

#[test]
fn test_ipc_message_proxy_connected_serialization() {
    let proxy_id = ProxyId::new();
    let proxy_info = ProxyInfo {
        id: proxy_id.clone(),
        name: "Test Proxy".to_string(),
        listen_address: "127.0.0.1:8080".to_string(),
        target_command: vec!["python".to_string(), "server.py".to_string()],
        status: ProxyStatus::Running,
        stats: ProxyStats::default(),
    };

    let message = IpcMessage::ProxyStarted(proxy_info.clone());

    let serialized = serde_json::to_string(&message).unwrap();
    let deserialized: IpcMessage = serde_json::from_str(&serialized).unwrap();

    match deserialized {
        IpcMessage::ProxyStarted(info) => {
            assert_eq!(info.id, proxy_info.id);
            assert_eq!(info.name, proxy_info.name);
            assert_eq!(info.listen_address, proxy_info.listen_address);
            assert_eq!(info.target_command, proxy_info.target_command);
        }
        _ => panic!("Expected ProxyStarted message"),
    }
}

#[test]
fn test_ipc_message_proxy_disconnected_serialization() {
    let proxy_id = ProxyId::new();
    let message = IpcMessage::ProxyStopped(proxy_id.clone());

    let serialized = serde_json::to_string(&message).unwrap();
    let deserialized: IpcMessage = serde_json::from_str(&serialized).unwrap();

    match deserialized {
        IpcMessage::ProxyStopped(id) => {
            assert_eq!(id, proxy_id);
        }
        _ => panic!("Expected ProxyStopped message"),
    }
}

#[test]
fn test_ipc_envelope_creation() {
    let proxy_id = ProxyId::new();
    let log_entry = LogEntry::new(LogLevel::Info, "Test message".to_string(), proxy_id);
    let message = IpcMessage::LogEntry(log_entry);

    let envelope = IpcEnvelope {
        message: message.clone(),
        timestamp: Utc::now(),
        correlation_id: Some(uuid::Uuid::new_v4()),
    };

    assert!(envelope.correlation_id.is_some());
    assert!(envelope.timestamp <= Utc::now());

    // Test serialization
    let serialized = serde_json::to_string(&envelope).unwrap();
    let deserialized: IpcEnvelope = serde_json::from_str(&serialized).unwrap();

    assert_eq!(envelope.correlation_id, deserialized.correlation_id);
    // Timestamps might have slight differences due to serialization precision
    let time_diff = (envelope.timestamp - deserialized.timestamp)
        .num_milliseconds()
        .abs();
    assert!(time_diff < 1000); // Less than 1 second difference
}

#[test]
fn test_ipc_envelope_without_correlation_id() {
    let proxy_id = ProxyId::new();
    let log_entry = LogEntry::new(LogLevel::Debug, "Debug message".to_string(), proxy_id);
    let message = IpcMessage::LogEntry(log_entry);

    let envelope = IpcEnvelope {
        message,
        timestamp: Utc::now(),
        correlation_id: None,
    };

    assert!(envelope.correlation_id.is_none());

    // Should still serialize/deserialize correctly
    let serialized = serde_json::to_string(&envelope).unwrap();
    let deserialized: IpcEnvelope = serde_json::from_str(&serialized).unwrap();

    assert!(deserialized.correlation_id.is_none());
}

#[test]
fn test_all_ipc_message_variants() {
    let proxy_id = ProxyId::new();

    let messages = vec![
        IpcMessage::LogEntry(LogEntry::new(
            LogLevel::Request,
            "Request message".to_string(),
            proxy_id.clone(),
        )),
        IpcMessage::StatsUpdate(ProxyStats {
            proxy_id: proxy_id.clone(),
            total_requests: 50,
            successful_requests: 48,
            failed_requests: 2,
            active_connections: 1,
            uptime: std::time::Duration::from_secs(1800),
            bytes_transferred: 256000,
        }),
        IpcMessage::ProxyStarted(ProxyInfo {
            id: proxy_id.clone(),
            name: "Test Proxy".to_string(),
            listen_address: "localhost:9000".to_string(),
            target_command: vec!["node".to_string(), "server.js".to_string()],
            status: ProxyStatus::Starting,
            stats: ProxyStats::default(),
        }),
        IpcMessage::ProxyStopped(proxy_id.clone()),
    ];

    for (i, message) in messages.iter().enumerate() {
        let serialized = serde_json::to_string(message).unwrap();
        let deserialized: IpcMessage = serde_json::from_str(&serialized).unwrap();

        // Verify the message type is preserved
        match (message, &deserialized) {
            (IpcMessage::LogEntry(_), IpcMessage::LogEntry(_)) => {}
            (IpcMessage::StatsUpdate(_), IpcMessage::StatsUpdate(_)) => {}
            (IpcMessage::ProxyStarted(_), IpcMessage::ProxyStarted(_)) => {}
            (IpcMessage::ProxyStopped(_), IpcMessage::ProxyStopped(_)) => {}
            _ => panic!("Message type mismatch at index {}", i),
        }
    }
}

#[test]
fn test_message_size_limits() {
    let proxy_id = ProxyId::new();

    // Test with a very large log message
    let large_message = "x".repeat(100000); // 100KB message
    let log_entry = LogEntry::new(LogLevel::Response, large_message.clone(), proxy_id);
    let message = IpcMessage::LogEntry(log_entry);

    // Should be able to serialize large messages
    let serialized = serde_json::to_string(&message).unwrap();
    assert!(serialized.len() > 100000);

    // Should be able to deserialize large messages
    let deserialized: IpcMessage = serde_json::from_str(&serialized).unwrap();
    match deserialized {
        IpcMessage::LogEntry(entry) => {
            assert_eq!(entry.message, large_message);
        }
        _ => panic!("Expected LogEntry message"),
    }
}

#[test]
fn test_message_with_special_characters() {
    let proxy_id = ProxyId::new();
    let special_message = "Test\nwith\nnewlines\tand\ttabs\"and quotes\"and\\backslashes";

    let log_entry = LogEntry::new(LogLevel::Error, special_message.to_string(), proxy_id);
    let message = IpcMessage::LogEntry(log_entry);

    let serialized = serde_json::to_string(&message).unwrap();
    let deserialized: IpcMessage = serde_json::from_str(&serialized).unwrap();

    match deserialized {
        IpcMessage::LogEntry(entry) => {
            assert_eq!(entry.message, special_message);
        }
        _ => panic!("Expected LogEntry message"),
    }
}

#[test]
fn test_envelope_ordering() {
    let proxy_id = ProxyId::new();
    let mut envelopes = Vec::new();

    // Create multiple envelopes with slight time differences
    for i in 0..5 {
        let log_entry = LogEntry::new(LogLevel::Info, format!("Message {}", i), proxy_id.clone());
        let envelope = IpcEnvelope {
            message: IpcMessage::LogEntry(log_entry),
            timestamp: Utc::now() + chrono::Duration::milliseconds(i),
            correlation_id: Some(uuid::Uuid::new_v4()),
        };
        envelopes.push(envelope);

        // Small delay to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(1));
    }

    // Verify timestamps are in order
    for i in 1..envelopes.len() {
        assert!(envelopes[i].timestamp >= envelopes[i - 1].timestamp);
    }

    // All should have unique correlation IDs
    let mut correlation_ids = std::collections::HashSet::new();
    for envelope in &envelopes {
        let id = envelope.correlation_id.unwrap();
        assert!(correlation_ids.insert(id), "Duplicate correlation ID found");
    }
}
