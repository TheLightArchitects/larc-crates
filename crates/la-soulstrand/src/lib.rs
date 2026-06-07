//! # la-soulstrand
//!
//! Knowledge graph type vocabulary and backend traits for the Light Architects
//! SOUL engine.
//!
//! ## Default (no features) — zero async overhead
//!
//! Core graph types and error enums. Suitable for data-model-only consumers.
//!
//! | Tier | Types | Async? |
//! |------|-------|--------|
//! | **Data** | [`Step`], [`Helix`], [`Strand`], [`HelixLink`], [`SharedExperience`] | No |
//! | **Model** | [`HelixOrderingMode`], [`Tier`], [`SoulstrandError`] | No |
//!
//! ## Features — three independent leaf traits + one orchestration layer
//!
//! Each leaf trait is independently gated so consumers can swap one component
//! without pulling in the rest.
//!
//! | Feature | Exports | Typical backend |
//! |---------|---------|-----------------|
//! | `embedding` | [`EmbeddingBackend`] | fastembed, OpenAI, Ollama |
//! | `graph` | [`GraphBackend`] | Neo4j, Memgraph, in-memory |
//! | `promotion` | [`PromotionBackend`] | custom consolidator pipeline |
//! | `helix` | [`HelixBackend`], [`SoulClient`], retrieval types | implies all three above |
//! | `embedding-cache` | [`CachedEmbeddingProvider`] | wraps any `EmbeddingBackend` with moka TTL cache; implies `embedding` |
//!
//! The SDK enables `la-soulstrand/helix` and implements `HelixBackend` on
//! `HelixStore`. External users implement `HelixBackend` for their own backend.

// ── Always-available: data types + model types (zero async) ─────────────────

mod error;
mod ordering;
mod tier;
mod types;

pub use error::SoulstrandError;
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

// ── Feature `helix` (implies embedding + graph + promotion) ──────────────────

#[cfg(feature = "helix")]
mod client;
#[cfg(feature = "helix")]
mod helix;
#[cfg(feature = "helix")]
mod retrieval;

#[cfg(feature = "helix")]
pub use client::SoulClient;
#[cfg(feature = "helix")]
pub use helix::{HelixBackend, select_mode};
#[cfg(feature = "helix")]
pub use retrieval::{RetrievalMode, RetrievalResult, SignalWeights};
