use thiserror::Error;

/// Errors for la-mcp.
#[derive(Error, Debug)]
pub enum McpError {
    #[error("method not found: {0}")]
    MethodNotFound(String),

    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("transport error: {0}")]
    TransportError(String),

    #[error("handler error: {0}")]
    HandlerError(String),
}
