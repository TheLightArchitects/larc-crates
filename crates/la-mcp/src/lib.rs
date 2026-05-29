//! # la-mcp
//!
//! Domain-agnostic MCP server framework with multi-transport and squad-aware routing.
//!
//! Provides a trait-based architecture for building MCP servers that can run over
//! stdio (JSON-RPC), HTTP/SSE, or WebSocket transports. The `lightsquad` feature
//! gate adds operator-wins invariant, HITL decision bus, and squad-aware routing.
//!
//! ## Quick start
//!
//! ```ignore
//! use la_mcp::{McpServer, McpHandler, McpTransport, StdioTransport};
//!
//! struct MyHandler;
//!
//! impl McpHandler for MyHandler {
//!     fn handle_request(&self, method: &str, params: serde_json::Value)
//!         -> Result<serde_json::Value, la_mcp::McpError>
//!     {
//!         match method {
//!             "tools/list" => Ok(serde_json::json!({"tools": []})),
//!             _ => Err(la_mcp::McpError::MethodNotFound(method.into())),
//!         }
//!     }
//! }
//! ```

mod error;
mod handler;
mod transport;

pub use error::McpError;
pub use handler::McpHandler;
pub use transport::McpTransport;

// stdio transport (default)
#[cfg(feature = "stdio")]
mod stdio_transport;

#[cfg(feature = "stdio")]
pub use stdio_transport::StdioTransport;

// HTTP/SSE transport
#[cfg(feature = "http")]
mod http_transport;

#[cfg(feature = "http")]
pub use http_transport::HttpTransport;

// WebSocket transport
#[cfg(feature = "websocket")]
mod ws_transport;

#[cfg(feature = "websocket")]
pub use ws_transport::WebSocketTransport;

/// Core trait for MCP server identity.
pub trait McpServer: Send + Sync {
    /// Server name (reported in MCP handshake).
    fn name(&self) -> &str;

    /// Server version (reported in MCP handshake).
    fn version(&self) -> &str;

    /// The handler that processes MCP requests.
    fn handler(&self) -> &dyn McpHandler;
}
