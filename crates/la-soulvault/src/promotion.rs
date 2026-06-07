//! `PromotionBackend` — contract for the consolidation pipeline.
//!
//! Implement this to plug in custom promotion logic:
//! the SDK's `SoulConsolidator` daemon, a batch job, a no-op stub, etc.

use async_trait::async_trait;
use std::fmt::Debug;

use crate::{SoulvaultError, Step, Tier};

/// Contract for the helix consolidation pipeline.
///
/// The consolidator is the background process that promotes raw spans and
/// turn-log entries into permanent helix steps, manages tier transitions,
/// and deduplicates content.
///
/// The production implementation is the `soul-consolidator` daemon in
/// `lightarchitects-sdk`. Implement this trait to provide a custom pipeline.
#[async_trait]
pub trait PromotionBackend: Debug + Send + Sync {
    /// Promote a batch of raw steps into enriched helix entries.
    ///
    /// Implementations may deduplicate, enrich metadata, compute embeddings,
    /// or apply any transformation before writing to the graph. Returns the
    /// promoted steps (may differ in count from input after deduplication).
    async fn promote(&self, steps: Vec<Step>) -> Result<Vec<Step>, SoulvaultError>;

    /// Move a step to a different lifecycle tier.
    ///
    /// The backend updates storage placement accordingly —
    /// e.g. evicting from memory on `Hot → Warm`, compressing on `Warm → Cold`.
    async fn tier_step(&self, id: &str, tier: Tier) -> Result<(), SoulvaultError>;

    /// Deduplicate a batch of steps before promotion.
    ///
    /// Returns the deduplicated set. The default implementation returns all
    /// steps unchanged — override for content-aware deduplication.
    async fn deduplicate(&self, steps: Vec<Step>) -> Result<Vec<Step>, SoulvaultError> {
        Ok(steps)
    }
}
