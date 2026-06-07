//! # la-soulvault
//!
//! Knowledge graph type vocabulary, backend traits, and optional production
//! vault backend for the Light Architects SOUL engine.
//!
//! ## Default (no features) — zero async overhead
//!
//! Core graph types and error enums. Suitable for data-model-only consumers.
//!
//! | Tier | Types | Async? |
//! |------|-------|--------|
//! | **Data** | [`Step`], [`Helix`], [`Strand`], [`HelixLink`], [`SharedExperience`] | No |
//! | **Model** | [`HelixOrderingMode`], [`Tier`], [`SoulvaultError`] | No |
//!
//! ## Features
//!
//! | Feature | Exports | Use case |
//! |---------|---------|----------|
//! | `embedding` | [`EmbeddingBackend`] | Implement your own embedding backend |
//! | `graph` | [`GraphBackend`] | Implement your own graph backend |
//! | `promotion` | [`PromotionBackend`] | Implement your own consolidation pipeline |
//! | `engine` | [`HelixBackend`], [`SoulClient`], retrieval types | Full orchestration layer — implies all three above |
//! | `embedding-cache` | [`CachedEmbeddingProvider`] | Moka TTL cache wrapping any `EmbeddingBackend` |
//! | `vault` | [`vault`] module — [`vault::connect`], [`vault::connect_from_env`] | Production Neo4j backend via the private `lightarchitects` SDK |
//!
//! ## Quick start
//!
//! **Build your own backend** (open source, `engine` feature):
//! ```toml
//! la-soulvault = { git = "...", features = ["engine"] }
//! ```
//!
//! **Use the production vault** (`vault` feature, requires SDK access):
//! ```toml
//! la-soulvault = { git = "...", features = ["vault"] }
//! lightarchitects = { path = "...", optional = false }
//! ```
//! ```rust,ignore
//! let client = la_soulvault::vault::connect_from_env().await?;
//! client.upsert_step(step).await?;
//! let results = client.retrieve_adaptive("helix-id", "my query", 10).await?;
//! ```

// ── Always-available: data types + model types (zero async) ─────────────────

mod error;
mod ordering;
mod tier;
mod types;

pub use error::SoulvaultError;
pub use ordering::HelixOrderingMode;
pub use tier::Tier;
pub use types::{Helix, HelixLink, SharedExperience, Step, Strand};

// ── Feature `embedding` ───────────────────────────────────────────────────────

#[cfg(feature = "embedding")]
mod embedding;
#[cfg(feature = "embedding")]
pub use embedding::EmbeddingBackend;

// ── Feature `graph` ───────────────────────────────────────────────────────────

#[cfg(feature = "graph")]
mod graph;
#[cfg(feature = "graph")]
pub use graph::GraphBackend;

// ── Feature `promotion` ──────────────────────────────────────────────────────

#[cfg(feature = "promotion")]
mod promotion;
#[cfg(feature = "promotion")]
pub use promotion::PromotionBackend;

// ── Feature `embedding-cache` (implies embedding) ────────────────────────────

#[cfg(feature = "embedding-cache")]
mod cached;
#[cfg(feature = "embedding-cache")]
pub use cached::CachedEmbeddingProvider;

// ── Feature `engine` (implies embedding + graph + promotion) ─────────────────

#[cfg(feature = "engine")]
mod client;
#[cfg(feature = "engine")]
mod helix;
#[cfg(feature = "engine")]
mod retrieval;

#[cfg(feature = "engine")]
pub use client::SoulClient;
#[cfg(feature = "engine")]
pub use helix::{HelixBackend, select_mode};
#[cfg(feature = "engine")]
pub use retrieval::{RetrievalMode, RetrievalResult, SignalWeights};

// ── Feature `vault` (implies engine + lightarchitects SDK) ───────────────────

#[cfg(feature = "vault")]
pub mod vault;
