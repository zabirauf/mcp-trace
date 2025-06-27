use crate::{LogEntry, ProxyId, ProxyInfo, ProxyStats};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcMessage {
    // Proxy -> Monitor messages
    ProxyStarted(ProxyInfo),
    ProxyStopped(ProxyId),
    LogEntry(LogEntry),
    StatsUpdate(ProxyStats),

    // Monitor -> Proxy messages
    GetStatus(ProxyId),
    GetLogs {
        proxy_id: ProxyId,
        limit: Option<usize>,
    },
    Shutdown(ProxyId),

    // Bidirectional messages
    Ping,
    Pong,

    // Error handling
    Error {
        message: String,
        proxy_id: Option<ProxyId>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcEnvelope {
    pub message: IpcMessage,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub correlation_id: Option<uuid::Uuid>,
}
