use crate::McpError;

/// Transport trait for MCP servers.
///
/// Implement this to add a new transport (e.g., custom HTTP middleware).
pub trait McpTransport: Send + Sync {
    /// Start the transport, routing requests to the handler.
    fn start(&self, handler: &dyn crate::McpHandler) -> Result<(), McpError>;

    /// Gracefully shut down the transport.
    fn shutdown(&self) -> Result<(), McpError>;
}
