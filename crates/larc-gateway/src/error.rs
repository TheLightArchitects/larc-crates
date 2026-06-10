//! Errors for `larc-gateway`.

/// Errors for the Light Architects gateway interface.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SdkError {
    /// Connection to the MCP transport failed.
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    /// A request timed out.
    #[error("request timed out after {0}ms")]
    Timeout(u64),

    /// The requested resource was not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// A JSON-RPC protocol error.
    #[error("JSON-RPC error: code={code}, message={message}")]
    JsonRpc { code: i64, message: String },

    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// The sibling is unavailable or not running.
    #[error("sibling unavailable: {0}")]
    SiblingUnavailable(String),

    /// A serialization/deserialization error.
    #[error("serialization error: {0}")]
    Serialization(String),
}
