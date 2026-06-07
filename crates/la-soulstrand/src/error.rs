//! Errors for la-soulstrand.

/// Errors for the SOUL knowledge graph.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SoulstrandError {
    /// Connection to the backend failed.
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    /// A retrieval query failed.
    #[error("query failed: {0}")]
    QueryFailed(String),

    /// The requested resource was not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Configuration error.
    #[error("configuration error: {0}")]
    ConfigError(String),

    /// Invalid input — empty ID, negative limit, malformed query, etc.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// A backend operation failed with a message.
    ///
    /// Used by utility wrappers (e.g. `CachedEmbeddingProvider`) when an
    /// error cannot be mapped to a more specific variant.
    #[error("backend error: {0}")]
    Backend(String),
}
