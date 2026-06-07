//! JSON-RPC types for MCP communication.

use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 protocol version constant.
pub const JSONRPC_VERSION: &str = "2.0";

/// A JSON-RPC 2.0 request.
///
/// Note: `id` is `u64` (numeric only) as MCP uses numeric request IDs.
/// String and null IDs from the full JSON-RPC 2.0 spec are not supported.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct JsonRpcRequest {
    /// JSON-RPC version — must be `"2.0"`.
    pub jsonrpc: String,
    /// Request ID — numeric per MCP convention.
    pub id: u64,
    /// Method name.
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl JsonRpcRequest {
    /// Create a new request with the correct JSON-RPC version.
    #[must_use]
    pub fn new(id: u64, method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            id,
            method: method.into(),
            params,
        }
    }
}

/// A JSON-RPC 2.0 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Create a successful response.
    #[must_use]
    pub fn ok(id: u64, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response.
    #[must_use]
    pub fn err(id: u64, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// A JSON-RPC 2.0 error.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    /// Create a new JSON-RPC error.
    #[must_use]
    pub fn new(code: i64, message: impl Into<String>, data: Option<serde_json::Value>) -> Self {
        Self {
            code,
            message: message.into(),
            data,
        }
    }
}

/// A JSON-RPC 2.0 notification (no response expected).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl JsonRpcNotification {
    /// Create a new notification.
    #[must_use]
    pub fn new(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            method: method.into(),
            params,
        }
    }
}
