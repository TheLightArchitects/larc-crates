//! Transport trait for MCP communication.

use crate::SdkError;
use async_trait::async_trait;

/// Transport layer for MCP JSON-RPC messages.
///
/// Implement this trait to provide a custom transport (stdio, HTTP, WebSocket).
/// The SDK provides built-in implementations for standard transports.
///
/// This trait is object-safe via `async-trait`. Use `Box<dyn Transport>`
/// for dynamic dispatch.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a JSON-RPC request and receive a response.
    async fn request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, SdkError>;

    /// Send a JSON-RPC notification (no response expected).
    async fn notify(&self, method: &str, params: serde_json::Value) -> Result<(), SdkError>;

    /// Close the transport connection.
    async fn close(&self) -> Result<(), SdkError>;

    /// Check if the transport is connected.
    fn is_connected(&self) -> bool;
}
