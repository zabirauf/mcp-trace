use mcp_common::*;
use serde_json;

#[test]
fn test_parse_valid_json_rpc_request() {
    let json =
        r#"{"jsonrpc": "2.0", "method": "test_method", "params": {"key": "value"}, "id": "123"}"#;

    let parsed: Result<serde_json::Value, _> = serde_json::from_str(json);
    assert!(parsed.is_ok());

    let value = parsed.unwrap();
    assert_eq!(value["jsonrpc"], "2.0");
    assert_eq!(value["method"], "test_method");
    assert_eq!(value["id"], "123");
    assert_eq!(value["params"]["key"], "value");
}

#[test]
fn test_parse_valid_json_rpc_response() {
    let json = r#"{"jsonrpc": "2.0", "result": {"success": true}, "id": "123"}"#;

    let parsed: Result<serde_json::Value, _> = serde_json::from_str(json);
    assert!(parsed.is_ok());

    let value = parsed.unwrap();
    assert_eq!(value["jsonrpc"], "2.0");
    assert_eq!(value["result"]["success"], true);
    assert_eq!(value["id"], "123");
}

#[test]
fn test_parse_json_rpc_error_response() {
    let json = r#"{"jsonrpc": "2.0", "error": {"code": -32601, "message": "Method not found"}, "id": "123"}"#;

    let parsed: Result<serde_json::Value, _> = serde_json::from_str(json);
    assert!(parsed.is_ok());

    let value = parsed.unwrap();
    assert_eq!(value["jsonrpc"], "2.0");
    assert_eq!(value["error"]["code"], -32601);
    assert_eq!(value["error"]["message"], "Method not found");
    assert_eq!(value["id"], "123");
}

#[test]
fn test_parse_notification_without_id() {
    let json = r#"{"jsonrpc": "2.0", "method": "notification", "params": {}}"#;

    let parsed: Result<serde_json::Value, _> = serde_json::from_str(json);
    assert!(parsed.is_ok());

    let value = parsed.unwrap();
    assert_eq!(value["jsonrpc"], "2.0");
    assert_eq!(value["method"], "notification");
    assert!(value["id"].is_null());
}

#[test]
fn test_parse_malformed_json() {
    let malformed_jsons = vec![
        r#"{"jsonrpc": "2.0", "method": "test", "id": 123"#, // Missing closing brace
        r#"{"jsonrpc": "2.0" "method": "test", "id": 123}"#, // Missing comma
        r#"{"jsonrpc": "2.0", "method": test", "id": 123}"#, // Unquoted method value
        r#"not json at all"#,                                // Not JSON
        r#""#,                                               // Empty string
    ];

    for json in malformed_jsons {
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(json);
        assert!(parsed.is_err(), "Should fail parsing: {}", json);
    }
}

#[test]
fn test_create_mcp_request() {
    let request = MCPRequest {
        id: "test-123".to_string(),
        method: "initialize".to_string(),
        params: Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })),
    };

    let json = serde_json::to_string(&request).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["id"], "test-123");
    assert_eq!(parsed["method"], "initialize");
    assert_eq!(parsed["params"]["clientInfo"]["name"], "test-client");
}

#[test]
fn test_create_mcp_response_success() {
    let response = MCPResponse {
        id: "test-123".to_string(),
        result: Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "test-server",
                "version": "1.0.0"
            }
        })),
        error: None,
    };

    let json = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["id"], "test-123");
    assert_eq!(parsed["result"]["serverInfo"]["name"], "test-server");
    assert!(parsed["error"].is_null());
}

#[test]
fn test_create_mcp_response_error() {
    let response = MCPResponse {
        id: "test-123".to_string(),
        result: None,
        error: Some(MCPError {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: Some(serde_json::json!({
                "details": "Missing required field"
            })),
        }),
    };

    let json = serde_json::to_string(&response).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["id"], "test-123");
    assert!(parsed["result"].is_null());
    assert_eq!(parsed["error"]["code"], -32600);
    assert_eq!(parsed["error"]["message"], "Invalid Request");
    assert_eq!(parsed["error"]["data"]["details"], "Missing required field");
}

#[test]
fn test_standard_json_rpc_error_codes() {
    let error_codes = vec![
        (-32700, "Parse error"),
        (-32600, "Invalid Request"),
        (-32601, "Method not found"),
        (-32602, "Invalid params"),
        (-32603, "Internal error"),
    ];

    for (code, message) in error_codes {
        let error = MCPError {
            code,
            message: message.to_string(),
            data: None,
        };

        let json = serde_json::to_string(&error).unwrap();
        let parsed: MCPError = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.code, code);
        assert_eq!(parsed.message, message);
    }
}

#[test]
fn test_round_trip_complex_params() {
    let complex_params = serde_json::json!({
        "tools": [
            {
                "name": "file_reader",
                "description": "Read files from filesystem",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to file"
                        }
                    },
                    "required": ["path"]
                }
            }
        ],
        "resources": [],
        "prompts": []
    });

    let request = MCPRequest {
        id: "complex-123".to_string(),
        method: "tools/list".to_string(),
        params: Some(complex_params.clone()),
    };

    // Serialize and deserialize
    let json = serde_json::to_string(&request).unwrap();
    let deserialized: MCPRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(request.id, deserialized.id);
    assert_eq!(request.method, deserialized.method);
    assert_eq!(request.params, deserialized.params);

    // Verify complex structure is preserved
    let params = deserialized.params.unwrap();
    assert_eq!(params["tools"][0]["name"], "file_reader");
    assert_eq!(
        params["tools"][0]["inputSchema"]["properties"]["path"]["type"],
        "string"
    );
}

#[test]
fn test_unicode_handling() {
    let unicode_message = "Test with unicode: 你好世界 🌍 émojis";

    let request = MCPRequest {
        id: "unicode-test".to_string(),
        method: unicode_message.to_string(),
        params: Some(serde_json::json!({
            "text": unicode_message,
            "emoji": "🚀",
            "chinese": "测试"
        })),
    };

    let json = serde_json::to_string(&request).unwrap();
    let deserialized: MCPRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(request.method, deserialized.method);
    assert_eq!(deserialized.params.unwrap()["text"], unicode_message);
}

#[test]
fn test_large_payload_handling() {
    // Create a large payload to test memory and performance
    let large_text = "x".repeat(10000); // 10KB string
    let mut large_array = Vec::new();
    for i in 0..1000 {
        large_array.push(serde_json::json!({
            "id": i,
            "data": format!("item_{}", i),
            "large_field": large_text.clone()
        }));
    }

    let request = MCPRequest {
        id: "large-payload".to_string(),
        method: "bulk_operation".to_string(),
        params: Some(serde_json::json!({
            "items": large_array
        })),
    };

    // Should be able to serialize and deserialize large payloads
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.len() > 10_000_000); // Should be > 10MB

    let deserialized: MCPRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(request.id, deserialized.id);
    assert_eq!(request.method, deserialized.method);

    let params = deserialized.params.unwrap();
    assert_eq!(params["items"].as_array().unwrap().len(), 1000);
}
