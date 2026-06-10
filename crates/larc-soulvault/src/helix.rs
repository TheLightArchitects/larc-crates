//! `HelixBackend` — orchestration layer over the three sub-traits.

use async_trait::async_trait;

use crate::{
    EmbeddingBackend, GraphBackend, Helix, HelixLink, PromotionBackend, RetrievalMode,
    RetrievalResult, SignalWeights, SoulvaultError, Step,
};

/// Select the adaptive retrieval mode based on corpus size.
///
/// | Step count | Mode |
/// |---|---|
/// | < 25 | [`KeywordDominated`](RetrievalMode::KeywordDominated) |
/// | 25–99 | [`Balanced`](RetrievalMode::Balanced) |
/// | ≥ 100 | [`GraphWeighted`](RetrievalMode::GraphWeighted) |
#[must_use]
pub fn select_mode(step_count: usize) -> RetrievalMode {
    match step_count {
        0..=24 => RetrievalMode::KeywordDominated,
        25..=99 => RetrievalMode::Balanced,
        _ => RetrievalMode::GraphWeighted,
    }
}

/// Orchestration layer over [`EmbeddingBackend`], [`GraphBackend`], and
/// [`PromotionBackend`].
///
/// `HelixBackend` adds the helix-specific write path (upsert/delete steps and
/// helix container nodes, link steps) and the 4-signal convex-combination ranked retrieval
/// that fuses the embedding and graph signals. Graph traversal, raw embedding,
/// and promotion are inherited from the three supertraits.
///
/// # Implementing
///
/// Implement all three supertraits on your struct, then implement the five
/// helix-specific methods below. The `retrieve_adaptive` default
/// implementation comes for free.
///
/// ```rust,ignore
/// use larc_soulvault::{
///     EmbeddingBackend, GraphBackend, HelixBackend, PromotionBackend,
///     Helix, HelixLink, RetrievalResult, SignalWeights, SoulvaultError, Step, Tier,
/// };
///
/// struct MyBackend;
///
/// #[async_trait::async_trait]
/// impl EmbeddingBackend for MyBackend {
///     async fn embed(&self, text: &str) -> Result<Vec<f32>, SoulvaultError> { todo!() }
///     fn dimensions(&self) -> usize { 768 }
/// }
///
/// #[async_trait::async_trait]
/// impl GraphBackend for MyBackend {
///     async fn upsert_node(&self, id: &str, labels: &[&str], props: serde_json::Value)
///         -> Result<(), SoulvaultError> { todo!() }
///     async fn upsert_edge(&self, from: &str, to: &str, rel: &str, props: serde_json::Value)
///         -> Result<(), SoulvaultError> { todo!() }
///     async fn delete_node(&self, id: &str) -> Result<(), SoulvaultError> { todo!() }
///     async fn traverse(&self, from: &str, depth: usize) -> Result<Vec<String>, SoulvaultError> { todo!() }
/// }
///
/// #[async_trait::async_trait]
/// impl PromotionBackend for MyBackend {
///     async fn promote(&self, steps: Vec<Step>) -> Result<Vec<Step>, SoulvaultError> { todo!() }
///     async fn tier_step(&self, id: &str, tier: Tier) -> Result<(), SoulvaultError> { todo!() }
/// }
///
/// #[async_trait::async_trait]
/// impl HelixBackend for MyBackend {
///     async fn upsert_step(&self, step: Step) -> Result<(), SoulvaultError> { todo!() }
///     async fn delete_step(&self, id: &str) -> Result<(), SoulvaultError> { todo!() }
///     async fn upsert_helix(&self, helix: Helix) -> Result<(), SoulvaultError> { todo!() }
///     async fn link_steps(&self, link: HelixLink) -> Result<(), SoulvaultError> { todo!() }
///     async fn get_step(&self, id: &str) -> Result<Option<Step>, SoulvaultError> { todo!() }
///     async fn step_count(&self, helix_id: &str) -> Result<usize, SoulvaultError> { todo!() }
///     async fn retrieve(&self, query: &str, k: usize, weights: &SignalWeights)
///         -> Result<Vec<RetrievalResult>, SoulvaultError> { todo!() }
/// }
/// ```
#[async_trait]
pub trait HelixBackend: EmbeddingBackend + GraphBackend + PromotionBackend {
    // ── Write path ────────────────────────────────────────────────────────────

    /// Create or update a helix step (content atom).
    async fn upsert_step(&self, step: Step) -> Result<(), SoulvaultError>;

    /// Delete a step and its incident graph edges.
    async fn delete_step(&self, id: &str) -> Result<(), SoulvaultError>;

    /// Create or update a helix container node.
    async fn upsert_helix(&self, helix: Helix) -> Result<(), SoulvaultError>;

    /// Create a typed, weighted edge between two steps.
    async fn link_steps(&self, link: HelixLink) -> Result<(), SoulvaultError>;

    // ── Read path ─────────────────────────────────────────────────────────────

    /// Point lookup — retrieve a single step by ID.
    async fn get_step(&self, id: &str) -> Result<Option<Step>, SoulvaultError>;

    /// Count steps in a helix — drives [`select_mode`] and adaptive retrieval.
    async fn step_count(&self, helix_id: &str) -> Result<usize, SoulvaultError>;

    /// 4-signal convex-combination ranked retrieval — return the top-`k` steps for `query`.
    ///
    /// Implementations fuse BM25, semantic embedding, graph traversal, and
    /// Node2Vec signals using `weights`. Use [`SignalWeights::for_mode`] with
    /// [`select_mode`] to auto-select appropriate weights.
    async fn retrieve(
        &self,
        query: &str,
        k: usize,
        weights: &SignalWeights,
    ) -> Result<Vec<RetrievalResult>, SoulvaultError>;

    // ── Default helpers ───────────────────────────────────────────────────────

    /// Ranked retrieval with auto-selected signal weights.
    ///
    /// Calls [`step_count`](Self::step_count) → [`select_mode`] →
    /// [`SignalWeights::for_mode`] → [`retrieve`](Self::retrieve).
    async fn retrieve_adaptive(
        &self,
        helix_id: &str,
        query: &str,
        k: usize,
    ) -> Result<Vec<RetrievalResult>, SoulvaultError> {
        let count = self.step_count(helix_id).await?;
        let weights = SignalWeights::for_mode(select_mode(count));
        self.retrieve(query, k, &weights).await
    }
}
