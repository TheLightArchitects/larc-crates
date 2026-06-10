//! # la-gateway
//!
//! Light Architects gateway interface — transport, MCP handler, and protocol types.
//!
//! This crate defines the **plug adapter**: how anything connects to the LA ecosystem.
//! The private `lightarchitects-sdk` provides the concrete implementations.
//!
//! ## Architecture
//!
//! ```text
//! la-gateway (interface — how you plug in)
//! ├── [default]: Transport, SdkError, Config, SiblingId, JSON-RPC types
//! ├── feature "mcp": McpHandler, SiblingHandler, HandlerError, HandlerConfig
//! └── feature "ayin": ObservableTransport + re-exports la-ayinspan
//! ```
//!
//! The typed sibling clients (SoulClient, CorsoClient, etc.) live in the SDK.
//! la-gateway only defines the protocol — how you connect, how you dispatch,
//! and the shape of the messages on the wire.
//!
//! ## Usage
//!
//! ```toml
//! [dependencies]
//! # Gateway connection types
//! la-gateway = { git = "https://github.com/TheLightArchitect/lightarchitects-sdk" }
//!
//! # With MCP handler traits
//! la-gateway = { git = "...", features = ["mcp"] }
//!
//! # Production implementation
//! lightarchitects = { git = "...", features = ["soul"] }
//! ```

mod config;
mod error;
mod json_rpc;
mod sibling_id;
mod transport;

pub use config::Config;
pub use error::SdkError;
pub use json_rpc::{JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
pub use sibling_id::SiblingId;
pub use transport::Transport;

// Feature-gated: MCP server handler traits
#[cfg(feature = "mcp")]
mod handler;

#[cfg(feature = "mcp")]
pub use handler::{HandlerConfig, HandlerError, McpHandler, SiblingHandler};

// Feature-gated: AYIN observability (re-exports la-ayinspan)
#[cfg(feature = "ayin")]
mod observe;

#[cfg(feature = "ayin")]
pub use observe::ObservableTransport;

#[cfg(feature = "ayin")]
pub use larc_ayinspan;
