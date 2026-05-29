use crate::{McpError, McpHandler, McpTransport};

/// HTTP/SSE transport — MCP over HTTP with Server-Sent Events.
pub struct HttpTransport {
    _inner: (),
}

impl HttpTransport {
    pub fn new() -> Self {
        Self { _inner: () }
    }
}

impl Default for HttpTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl McpTransport for HttpTransport {
    fn start(&self, _handler: &dyn McpHandler) -> Result<(), McpError> {
        todo!("implement HTTP/SSE transport")
    }

    fn shutdown(&self) -> Result<(), McpError> {
        todo!("implement HTTP shutdown")
    }
}
