//! # larc-gateway
//!
//! Gateway interface — transport abstraction, MCP handler traits, and JSON-RPC
//! protocol types. Pure contracts; bring your own engine.
//!
//! ## Architecture
//!
//! ```text
//! larc-gateway (interface — how you plug in)
//! ├── [default]: Transport, SdkError, Config, SiblingId, JSON-RPC types
//! ├── feature "mcp": McpHandler, SiblingHandler, HandlerError, HandlerConfig
//! └── feature "ayin": ObservableTransport + re-exports larc-ayinspan
//! ```
//!
//! This crate defines the protocol surface — how you connect, how you dispatch,
//! and the shape of the messages on the wire. Concrete client wrappers and
//! transport implementations are left to consumers.
//!
//! ## Usage
//!
//! ```toml
//! [dependencies]
//! larc-gateway = "0.1"
//!
//! # With MCP handler traits
//! larc-gateway = { version = "0.1", features = ["mcp"] }
//!
//! # With AYIN observability re-exports
//! larc-gateway = { version = "0.1", features = ["ayin"] }
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

// Feature-gated: AYIN observability (re-exports `larc-ayinspan`)
#[cfg(feature = "ayin")]
mod observe;

#[cfg(feature = "ayin")]
pub use observe::ObservableTransport;

#[cfg(feature = "ayin")]
pub use larc_ayinspan;
