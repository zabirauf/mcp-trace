use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ProxyId(pub Uuid);

impl ProxyId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ProxyId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Request,
    Response,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub proxy_id: ProxyId,
    pub request_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: String, proxy_id: ProxyId) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            level,
            message,
            proxy_id,
            request_id: None,
            metadata: None,
        }
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStats {
    pub proxy_id: ProxyId,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub active_connections: u32,
    pub uptime: std::time::Duration,
    pub bytes_transferred: u64,
}

impl Default for ProxyStats {
    fn default() -> Self {
        Self {
            proxy_id: ProxyId::new(),
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            active_connections: 0,
            uptime: std::time::Duration::from_secs(0),
            bytes_transferred: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyInfo {
    pub id: ProxyId,
    pub name: String,
    pub listen_address: String,
    pub target_command: Vec<String>,
    pub status: ProxyStatus,
    pub stats: ProxyStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyStatus {
    Starting,
    Running,
    Stopped,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPRequest {
    pub id: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResponse {
    pub id: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<MCPError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}
