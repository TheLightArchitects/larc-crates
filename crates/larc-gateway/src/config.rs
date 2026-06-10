//! SDK configuration types.

use serde::{Deserialize, Serialize};

/// Configuration for connecting to the Light Architects gateway.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Config {
    /// Gateway connection endpoint (e.g., "http://localhost:3742" or stdio path).
    pub endpoint: String,

    /// Request timeout in milliseconds.
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,

    /// Maximum number of retry attempts.
    #[serde(default = "default_retries")]
    pub max_retries: u32,

    /// Optional authentication token.
    #[serde(default, skip_serializing)]
    pub auth_token: Option<String>,
}

impl Config {
    /// Create a new config with an endpoint and defaults for all other fields.
    #[must_use]
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            timeout_ms: default_timeout(),
            max_retries: default_retries(),
            auth_token: None,
        }
    }
}

fn default_timeout() -> u64 {
    30_000
}

fn default_retries() -> u32 {
    3
}

impl Default for Config {
    fn default() -> Self {
        Self::new("http://localhost:3742")
    }
}
