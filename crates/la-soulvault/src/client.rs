//! `SoulClient` — type-erased wrapper over a `Box<dyn HelixBackend>`.
//!
//! Exposes all methods from [`HelixBackend`] and its three supertraits
//! ([`EmbeddingBackend`], [`GraphBackend`], [`PromotionBackend`]) without
//! requiring callers to carry the concrete backend type as a generic parameter.

use crate::{
    Helix, HelixBackend, HelixLink, RetrievalResult, SignalWeights, SoulvaultError, Step, Tier,
};

/// Owned, type-erased helix client.
///
/// ```rust,ignore
/// let client = SoulClient::new(MyBackend::new());
///
/// // write
/// client.upsert_step(step).await?;
///
/// // read — auto-selects fusion weights by corpus size
/// let results = client.retrieve_adaptive("claude", "session context", 5).await?;
///
/// // embedding
/// let vec = client.embed("query text").await?;
///
/// // graph
/// let neighbours = client.neighbors("step-42").await?;
///
/// // promotion
/// client.tier_step("step-42", Tier::Cold).await?;
/// ```
#[derive(Debug)]
#[non_exhaustive]
pub struct SoulClient {
    inner: Box<dyn HelixBackend>,
}

impl SoulClient {
    /// Wrap a concrete [`HelixBackend`] implementation.
    pub fn new(backend: impl HelixBackend + 'static) -> Self {
        Self {
            inner: Box::new(backend),
        }
    }

    // ── HelixBackend — write path ─────────────────────────────────────────────

    /// Create or update a step.
    pub async fn upsert_step(&self, step: Step) -> Result<(), SoulvaultError> {
        self.inner.upsert_step(step).await
    }

    /// Delete a step and its incident edges.
    pub async fn delete_step(&self, id: &str) -> Result<(), SoulvaultError> {
        self.inner.delete_step(id).await
    }

    /// Create or update a helix container.
    pub async fn upsert_helix(&self, helix: Helix) -> Result<(), SoulvaultError> {
        self.inner.upsert_helix(helix).await
    }

    /// Create a typed, weighted edge between two steps.
    pub async fn link_steps(&self, link: HelixLink) -> Result<(), SoulvaultError> {
        self.inner.link_steps(link).await
    }

    // ── HelixBackend — read path ──────────────────────────────────────────────

    /// Point lookup by step ID.
    pub async fn get_step(&self, id: &str) -> Result<Option<Step>, SoulvaultError> {
        self.inner.get_step(id).await
    }

    /// Number of steps in a helix.
    pub async fn step_count(&self, helix_id: &str) -> Result<usize, SoulvaultError> {
        self.inner.step_count(helix_id).await
    }

    /// 4-signal convex-combination ranked retrieval with explicit weights.
    pub async fn retrieve(
        &self,
        query: &str,
        k: usize,
        weights: &SignalWeights,
    ) -> Result<Vec<RetrievalResult>, SoulvaultError> {
        self.inner.retrieve(query, k, weights).await
    }

    /// 4-signal convex-combination ranked retrieval with auto-selected weights.
    pub async fn retrieve_adaptive(
        &self,
        helix_id: &str,
        query: &str,
        k: usize,
    ) -> Result<Vec<RetrievalResult>, SoulvaultError> {
        self.inner.retrieve_adaptive(helix_id, query, k).await
    }

    // ── EmbeddingBackend ──────────────────────────────────────────────────────

    /// Embed a single text string into a dense vector.
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, SoulvaultError> {
        self.inner.embed(text).await
    }

    /// Embed a batch of texts.
    pub async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, SoulvaultError> {
        self.inner.embed_batch(texts).await
    }

    /// Embedding vector dimensionality.
    pub fn dimensions(&self) -> usize {
        self.inner.dimensions()
    }

    // ── GraphBackend ──────────────────────────────────────────────────────────

    /// Create or update a graph node.
    pub async fn upsert_node(
        &self,
        id: &str,
        labels: &[&str],
        props: serde_json::Value,
    ) -> Result<(), SoulvaultError> {
        self.inner.upsert_node(id, labels, props).await
    }

    /// Create or update a directed graph edge.
    pub async fn upsert_edge(
        &self,
        from_id: &str,
        to_id: &str,
        rel_type: &str,
        props: serde_json::Value,
    ) -> Result<(), SoulvaultError> {
        self.inner
            .upsert_edge(from_id, to_id, rel_type, props)
            .await
    }

    /// Follow outgoing edges up to `depth` hops.
    pub async fn traverse(
        &self,
        from_id: &str,
        depth: usize,
    ) -> Result<Vec<String>, SoulvaultError> {
        self.inner.traverse(from_id, depth).await
    }

    /// Direct neighbours of a node (depth = 1).
    pub async fn neighbors(&self, id: &str) -> Result<Vec<String>, SoulvaultError> {
        self.inner.neighbors(id).await
    }

    // ── PromotionBackend ──────────────────────────────────────────────────────

    /// Promote a batch of raw steps into enriched helix entries.
    pub async fn promote(&self, steps: Vec<Step>) -> Result<Vec<Step>, SoulvaultError> {
        self.inner.promote(steps).await
    }

    /// Move a step to a different lifecycle tier.
    pub async fn tier_step(&self, id: &str, tier: Tier) -> Result<(), SoulvaultError> {
        self.inner.tier_step(id, tier).await
    }

    /// Deduplicate a batch of steps before promotion.
    pub async fn deduplicate(&self, steps: Vec<Step>) -> Result<Vec<Step>, SoulvaultError> {
        self.inner.deduplicate(steps).await
    }
}
