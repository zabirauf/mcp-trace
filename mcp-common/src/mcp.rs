use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(JsonRpcNotification),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcMessage {
    pub fn parse(input: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(input)
    }

    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn get_method(&self) -> Option<&str> {
        match self {
            JsonRpcMessage::Request(req) => Some(&req.method),
            JsonRpcMessage::Notification(notif) => Some(&notif.method),
            JsonRpcMessage::Response(_) => None,
        }
    }

    pub fn get_id(&self) -> Option<&Value> {
        match self {
            JsonRpcMessage::Request(req) => Some(&req.id),
            JsonRpcMessage::Response(resp) => Some(&resp.id),
            JsonRpcMessage::Notification(_) => None,
        }
    }
}

// Common MCP method names
pub mod methods {
    pub const INITIALIZE: &str = "initialize";
    pub const INITIALIZED: &str = "initialized";
    pub const LIST_TOOLS: &str = "tools/list";
    pub const CALL_TOOL: &str = "tools/call";
    pub const LIST_RESOURCES: &str = "resources/list";
    pub const READ_RESOURCE: &str = "resources/read";
    pub const LIST_PROMPTS: &str = "prompts/list";
    pub const GET_PROMPT: &str = "prompts/get";
    pub const COMPLETION: &str = "completion/complete";
    pub const LOGGING: &str = "logging/setLevel";
    pub const ROOTS_LIST: &str = "roots/list";
    pub const SAMPLING: &str = "sampling/createMessage";
}
