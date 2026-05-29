use crate::{McpError, McpHandler, McpTransport};

/// WebSocket transport — MCP over persistent WebSocket connections.
pub struct WebSocketTransport {
    _inner: (),
}

impl WebSocketTransport {
    pub fn new() -> Self {
        Self { _inner: () }
    }
}

impl Default for WebSocketTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl McpTransport for WebSocketTransport {
    fn start(&self, _handler: &dyn McpHandler) -> Result<(), McpError> {
        todo!("implement WebSocket transport")
    }

    fn shutdown(&self) -> Result<(), McpError> {
        todo!("implement WebSocket shutdown")
    }
}
