use chrono::Utc;
use mcp_common::*;
use serde_json;

#[test]
fn test_proxy_id_creation() {
    let id1 = ProxyId::new();
    let id2 = ProxyId::new();

    // Each ProxyId should be unique
    assert_ne!(id1, id2);

    // Should be valid UUIDs
    assert!(id1.0.get_version().is_some());
    assert!(id2.0.get_version().is_some());
}

#[test]
fn test_proxy_id_default() {
    let id1 = ProxyId::default();
    let id2 = ProxyId::default();

    // Each default ProxyId should be unique
    assert_ne!(id1, id2);
}

#[test]
fn test_proxy_id_serialization() {
    let id = ProxyId::new();

    // Should serialize and deserialize correctly
    let serialized = serde_json::to_string(&id).unwrap();
    let deserialized: ProxyId = serde_json::from_str(&serialized).unwrap();

    assert_eq!(id, deserialized);
}

#[test]
fn test_log_level_serialization() {
    let levels = vec![
        LogLevel::Debug,
        LogLevel::Info,
        LogLevel::Warning,
        LogLevel::Error,
        LogLevel::Request,
        LogLevel::Response,
    ];

    for level in levels {
        let serialized = serde_json::to_string(&level).unwrap();
        let deserialized: LogLevel = serde_json::from_str(&serialized).unwrap();
        assert_eq!(format!("{:?}", level), format!("{:?}", deserialized));
    }
}

#[test]
fn test_log_entry_creation() {
    let proxy_id = ProxyId::new();
    let message = "Test message".to_string();
    let level = LogLevel::Info;

    let entry = LogEntry::new(level.clone(), message.clone(), proxy_id.clone());

    assert_eq!(entry.level, level);
    assert_eq!(entry.message, message);
    assert_eq!(entry.proxy_id, proxy_id);
    assert!(entry.request_id.is_none());
    assert!(entry.metadata.is_none());

    // Should have valid UUID and timestamp
    assert!(entry.id.get_version().is_some());
    assert!(entry.timestamp <= Utc::now());
}

#[test]
fn test_log_entry_with_request_id() {
    let proxy_id = ProxyId::new();
    let request_id = "test-request-123".to_string();

    let entry = LogEntry::new(LogLevel::Request, "Test".to_string(), proxy_id)
        .with_request_id(request_id.clone());

    assert_eq!(entry.request_id, Some(request_id));
}

#[test]
fn test_log_entry_with_metadata() {
    let proxy_id = ProxyId::new();
    let metadata = serde_json::json!({
        "method": "test_method",
        "params": {"key": "value"}
    });

    let entry = LogEntry::new(LogLevel::Request, "Test".to_string(), proxy_id)
        .with_metadata(metadata.clone());

    assert_eq!(entry.metadata, Some(metadata));
}

#[test]
fn test_log_entry_chaining() {
    let proxy_id = ProxyId::new();
    let request_id = "test-request-123".to_string();
    let metadata = serde_json::json!({"test": "data"});

    let entry = LogEntry::new(LogLevel::Request, "Test".to_string(), proxy_id)
        .with_request_id(request_id.clone())
        .with_metadata(metadata.clone());

    assert_eq!(entry.request_id, Some(request_id));
    assert_eq!(entry.metadata, Some(metadata));
}

#[test]
fn test_proxy_stats_default() {
    let stats = ProxyStats::default();

    assert_eq!(stats.total_requests, 0);
    assert_eq!(stats.successful_requests, 0);
    assert_eq!(stats.failed_requests, 0);
    assert_eq!(stats.active_connections, 0);
    assert_eq!(stats.bytes_transferred, 0);
    assert_eq!(stats.uptime.as_secs(), 0);

    // Should have a valid proxy ID
    assert!(stats.proxy_id.0.get_version().is_some());
}

#[test]
fn test_proxy_stats_serialization() {
    let stats = ProxyStats {
        proxy_id: ProxyId::new(),
        total_requests: 100,
        successful_requests: 95,
        failed_requests: 5,
        active_connections: 3,
        uptime: std::time::Duration::from_secs(3600),
        bytes_transferred: 1024000,
    };

    let serialized = serde_json::to_string(&stats).unwrap();
    let deserialized: ProxyStats = serde_json::from_str(&serialized).unwrap();

    assert_eq!(stats.proxy_id, deserialized.proxy_id);
    assert_eq!(stats.total_requests, deserialized.total_requests);
    assert_eq!(stats.successful_requests, deserialized.successful_requests);
    assert_eq!(stats.failed_requests, deserialized.failed_requests);
    assert_eq!(stats.active_connections, deserialized.active_connections);
    assert_eq!(stats.bytes_transferred, deserialized.bytes_transferred);
    // Note: Duration serialization might have slight differences, so we check within reasonable bounds
    assert!(deserialized.uptime.as_secs() >= 3599 && deserialized.uptime.as_secs() <= 3601);
}

#[test]
fn test_proxy_status_variants() {
    let statuses = vec![
        ProxyStatus::Starting,
        ProxyStatus::Running,
        ProxyStatus::Stopped,
        ProxyStatus::Error("Test error".to_string()),
    ];

    for status in statuses {
        let serialized = serde_json::to_string(&status).unwrap();
        let deserialized: ProxyStatus = serde_json::from_str(&serialized).unwrap();

        match (&status, &deserialized) {
            (ProxyStatus::Starting, ProxyStatus::Starting) => {}
            (ProxyStatus::Running, ProxyStatus::Running) => {}
            (ProxyStatus::Stopped, ProxyStatus::Stopped) => {}
            (ProxyStatus::Error(msg1), ProxyStatus::Error(msg2)) => assert_eq!(msg1, msg2),
            _ => panic!("Status mismatch: {:?} != {:?}", status, deserialized),
        }
    }
}

#[test]
fn test_mcp_request_serialization() {
    let request = MCPRequest {
        id: "test-123".to_string(),
        method: "test_method".to_string(),
        params: Some(serde_json::json!({"key": "value"})),
    };

    let serialized = serde_json::to_string(&request).unwrap();
    let deserialized: MCPRequest = serde_json::from_str(&serialized).unwrap();

    assert_eq!(request.id, deserialized.id);
    assert_eq!(request.method, deserialized.method);
    assert_eq!(request.params, deserialized.params);
}

#[test]
fn test_mcp_response_with_result() {
    let response = MCPResponse {
        id: "test-123".to_string(),
        result: Some(serde_json::json!({"success": true})),
        error: None,
    };

    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: MCPResponse = serde_json::from_str(&serialized).unwrap();

    assert_eq!(response.id, deserialized.id);
    assert_eq!(response.result, deserialized.result);
    assert!(deserialized.error.is_none());
}

#[test]
fn test_mcp_response_with_error() {
    let error = MCPError {
        code: -32601,
        message: "Method not found".to_string(),
        data: Some(serde_json::json!({"additional": "info"})),
    };

    let response = MCPResponse {
        id: "test-123".to_string(),
        result: None,
        error: Some(error.clone()),
    };

    let serialized = serde_json::to_string(&response).unwrap();
    let deserialized: MCPResponse = serde_json::from_str(&serialized).unwrap();

    assert_eq!(response.id, deserialized.id);
    assert!(deserialized.result.is_none());

    let deserialized_error = deserialized.error.unwrap();
    assert_eq!(error.code, deserialized_error.code);
    assert_eq!(error.message, deserialized_error.message);
    assert_eq!(error.data, deserialized_error.data);
}

#[test]
fn test_proxy_info_complete() {
    let proxy_id = ProxyId::new();
    let stats = ProxyStats {
        proxy_id: proxy_id.clone(),
        total_requests: 50,
        successful_requests: 48,
        failed_requests: 2,
        active_connections: 1,
        uptime: std::time::Duration::from_secs(1800),
        bytes_transferred: 512000,
    };

    let info = ProxyInfo {
        id: proxy_id.clone(),
        name: "Test Proxy".to_string(),
        listen_address: "127.0.0.1:8080".to_string(),
        target_command: vec!["python".to_string(), "server.py".to_string()],
        status: ProxyStatus::Running,
        stats: stats.clone(),
    };

    let serialized = serde_json::to_string(&info).unwrap();
    let deserialized: ProxyInfo = serde_json::from_str(&serialized).unwrap();

    assert_eq!(info.id, deserialized.id);
    assert_eq!(info.name, deserialized.name);
    assert_eq!(info.listen_address, deserialized.listen_address);
    assert_eq!(info.target_command, deserialized.target_command);
    assert_eq!(info.stats.total_requests, deserialized.stats.total_requests);
}
