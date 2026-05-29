use crate::McpError;
use serde_json::Value;

/// Handler trait for processing MCP requests.
///
/// Implement this to define your server's request routing.
pub trait McpHandler: Send + Sync {
    /// Handle an incoming MCP request.
    fn handle_request(&self, method: &str, params: Value) -> Result<Value, McpError>;
}
