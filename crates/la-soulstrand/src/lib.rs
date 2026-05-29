//! # la-soulstrand
//!
//! Knowledge graph + retrieval library with 4-signal RRF.
//!
//! Tier 1 (default): SQLite + BM25/FTS5 — no Neo4j needed.
//! Tier 2 (feature `helix`): Neo4j graph backend with 4-signal RRF retrieval.
//!
//! ## Quick start
//!
//! ```no_run
//! use la_soulstrand::SoulClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = SoulClient::builder().connect().await?;
//! let results = client.retrieve("what is consciousness?", 5).await?;
//! for result in &results {
//!     println!("{}: {:.3}", result.id(), result.score());
//! }
//! # Ok(())
//! # }
//! ```

// Core types — always available
mod types;
mod error;
mod client;
mod builder;

pub use types::*;
pub use error::*;
pub use client::SoulClient;
pub use builder::SoulClientBuilder;

// Tier 1: SQLite backend (default)
#[cfg(feature = "sqlite")]
mod sqlite;

#[cfg(feature = "sqlite")]
pub use sqlite::SqliteBackend;

// Tier 2: Neo4j + 4-signal RRF
#[cfg(feature = "helix")]
mod helix;

#[cfg(feature = "helix")]
pub use helix::{
    HelixBackend,
    HybridRetriever,
    SignalWeights,
};