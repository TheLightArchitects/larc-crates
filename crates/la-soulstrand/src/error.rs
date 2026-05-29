use thiserror::Error;

/// Errors for la-soulstrand.
#[derive(Error, Debug)]
pub enum SoulstrandError {
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    #[error("query failed: {0}")]
    QueryFailed(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("configuration error: {0}")]
    ConfigError(String),
}
