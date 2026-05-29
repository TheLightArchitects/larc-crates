use crate::{McpError, McpHandler, McpTransport};

/// stdio transport — JSON-RPC over stdin/stdout.
pub struct StdioTransport {
    _inner: (),
}

impl StdioTransport {
    pub fn new() -> Self {
        Self { _inner: () }
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl McpTransport for StdioTransport {
    fn start(&self, _handler: &dyn McpHandler) -> Result<(), McpError> {
        todo!("implement stdio transport")
    }

    fn shutdown(&self) -> Result<(), McpError> {
        todo!("implement stdio shutdown")
    }
}
