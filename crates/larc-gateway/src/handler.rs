//! MCP server handler traits.

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;

/// Handler for MCP tool requests.
///
/// Implement this trait to handle incoming MCP tool calls. A server loop
/// dispatches incoming JSON-RPC envelopes to registered handlers; this trait
/// defines the contract the loop expects.
///
/// This trait is not object-safe due to async-fn-in-trait return types.
/// Use [`SiblingHandler`] when you need dynamic dispatch.
#[async_trait]
pub trait McpHandler: Send + Sync + 'static {
    /// Handle a raw JSON-RPC request.
    ///
    /// Returns `Vec<Value>`:
    /// - Empty: suppress reply (notification).
    /// - Single: normal JSON-RPC response.
    /// - Multiple: send notifications, then respond with last item.
    async fn handle(&self, raw: serde_json::Value) -> Vec<serde_json::Value>;
}

/// Object-safe handler trait for dynamic dispatch by name.
///
/// Implement this trait to define a sibling handler that the gateway
/// can route requests to by name.
#[async_trait]
pub trait SiblingHandler: Send + Sync {
    /// Handler name (e.g., "soul", "corso", "eva").
    fn name(&self) -> &'static str;

    /// Call a specific action on this handler.
    async fn call(
        &self,
        action: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, HandlerError>;

    /// List of action names this handler supports.
    fn actions(&self) -> &[&'static str];

    /// Initialize the handler with configuration.
    async fn initialize(&self, _config: &HandlerConfig) -> Result<(), HandlerError> {
        Ok(())
    }

    /// Shut down the handler gracefully.
    async fn shutdown(&self) -> Result<(), HandlerError> {
        Ok(())
    }
}

/// Errors from handler dispatch.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum HandlerError {
    #[error("unknown action '{action}' on handler '{handler}'")]
    UnknownAction { handler: String, action: String },
    #[error("invalid params for '{action}' on '{handler}': {message}")]
    InvalidParams {
        handler: String,
        action: String,
        message: String,
    },
    #[error("handler '{handler}' not initialized: {message}")]
    NotInitialized { handler: String, message: String },
    #[error("service error on '{handler}/{action}': {message}")]
    ServiceError {
        handler: String,
        action: String,
        message: String,
    },
    #[error("internal error on '{handler}/{action}': {message}")]
    Internal {
        handler: String,
        action: String,
        message: String,
    },
}

/// Configuration for handler initialization.
#[derive(Clone)]
#[non_exhaustive]
pub struct HandlerConfig {
    pub api_keys: HashMap<String, String>,
    pub vault_path: PathBuf,
    pub helix_path: PathBuf,
    pub home_dir: PathBuf,
}

impl std::fmt::Debug for HandlerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HandlerConfig")
            .field(
                "api_keys",
                &format!("{} keys REDACTED", self.api_keys.len()),
            )
            .field("vault_path", &self.vault_path)
            .field("helix_path", &self.helix_path)
            .field("home_dir", &self.home_dir)
            .finish()
    }
}
