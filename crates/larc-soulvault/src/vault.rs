//! Production vault backend via the private `lightarchitects` SDK.
//!
//! Enabled with `features = ["vault"]`.
//!
//! # Usage
//!
//! ```rust,ignore
//! // From explicit config:
//! let client = larc_soulvault::vault::connect(VaultConfig::new(
//!     "bolt://localhost:7687", "neo4j", "password",
//! )).await?;
//!
//! // From environment variables (SOUL_VAULT_URI / SOUL_VAULT_USER / SOUL_VAULT_PASSWORD):
//! let client = larc_soulvault::vault::connect_from_env().await?;
//!
//! // Use the SoulClient:
//! client.upsert_step(step).await?;
//! let hits = client.retrieve_adaptive("helix-id", "query text", 10).await?;
//! ```

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;

use lightarchitects::helix::{
    HelixDb, HelixNeo4j, HelixStore,
    embedding::{create_embedding_provider, EmbeddingConfig, EmbeddingProvider},
    search::SearchOptions,
    soul_search::hybrid::{HybridRetriever, HybridRetrieverConfig, SignalWeights as SdkWeights},
    types as sdk_types,
};

use crate::{
    EmbeddingBackend, GraphBackend, Helix, HelixBackend, HelixLink, HelixOrderingMode,
    PromotionBackend, RetrievalResult, SignalWeights, SoulvaultError, Step, Tier,
};

// ============================================================================
// VaultConfig
// ============================================================================

/// Connection configuration for a SOUL vault.
///
/// Create via [`VaultConfig::new`] or [`VaultConfig::from_env`].
#[derive(Debug, Clone)]
pub struct VaultConfig {
    /// Neo4j Bolt URI, e.g. `"bolt://localhost:7687"`.
    pub uri: String,
    /// Neo4j username.
    pub user: String,
    /// Neo4j password.
    pub password: String,
}

impl VaultConfig {
    /// Create a config from explicit values.
    pub fn new(
        uri: impl Into<String>,
        user: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            uri: uri.into(),
            user: user.into(),
            password: password.into(),
        }
    }

    /// Read config from environment variables.
    ///
    /// | Variable | Field |
    /// |---|---|
    /// | `SOUL_VAULT_URI` | `uri` |
    /// | `SOUL_VAULT_USER` | `user` |
    /// | `SOUL_VAULT_PASSWORD` | `password` |
    ///
    /// # Errors
    ///
    /// Returns [`SoulvaultError::ConfigError`] if any variable is unset.
    pub fn from_env() -> Result<Self, SoulvaultError> {
        let uri = std::env::var("SOUL_VAULT_URI")
            .map_err(|_| SoulvaultError::ConfigError("SOUL_VAULT_URI not set".into()))?;
        let user = std::env::var("SOUL_VAULT_USER")
            .map_err(|_| SoulvaultError::ConfigError("SOUL_VAULT_USER not set".into()))?;
        let password = std::env::var("SOUL_VAULT_PASSWORD")
            .map_err(|_| SoulvaultError::ConfigError("SOUL_VAULT_PASSWORD not set".into()))?;
        Ok(Self { uri, user, password })
    }
}

// ============================================================================
// SdkHelixAdapter
// ============================================================================

/// SDK-backed implementation of [`HelixBackend`].
///
/// Wraps `Arc<HelixNeo4j>` + an `EmbeddingProvider` and adapts the
/// private SDK API to the la-soulvault trait surface.
///
/// `GraphBackend` methods on this type delegate to the helix-level API
/// (link creation, traversal via backlinks). For raw Neo4j graph work,
/// use the helix-specific methods from `HelixBackend` directly.
pub struct SdkHelixAdapter {
    inner: Arc<HelixNeo4j>,
    embedder: Arc<dyn EmbeddingProvider>,
}

impl fmt::Debug for SdkHelixAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SdkHelixAdapter").finish_non_exhaustive()
    }
}

// ============================================================================
// EmbeddingBackend
// ============================================================================

#[async_trait]
impl EmbeddingBackend for SdkHelixAdapter {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, SoulvaultError> {
        let mut vecs = self
            .embedder
            .embed(&[text])
            .await
            .map_err(|e| SoulvaultError::Backend(e.to_string()))?;
        vecs.pop()
            .ok_or_else(|| SoulvaultError::Backend("embedder returned empty batch".into()))
    }

    fn dimensions(&self) -> usize {
        self.embedder.dimensions()
    }
}

// ============================================================================
// GraphBackend
// ============================================================================

#[async_trait]
impl GraphBackend for SdkHelixAdapter {
    /// Stub — use `upsert_step` / `upsert_helix` from [`HelixBackend`] instead.
    async fn upsert_node(
        &self,
        _id: &str,
        _labels: &[&str],
        _props: serde_json::Value,
    ) -> Result<(), SoulvaultError> {
        Ok(())
    }

    /// Creates a typed edge between two steps.
    async fn upsert_edge(
        &self,
        from_id: &str,
        to_id: &str,
        rel_type: &str,
        props: serde_json::Value,
    ) -> Result<(), SoulvaultError> {
        let weight = props
            .get("weight")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(1.0);
        let link = crate::HelixLink::new(
            from_id.to_owned(),
            to_id.to_owned(),
            rel_type.to_owned(),
            weight,
        );
        self.link_steps(link).await
    }

    /// Detaches and deletes a step node by ID.
    async fn delete_node(&self, id: &str) -> Result<(), SoulvaultError> {
        self.delete_step(id).await
    }

    /// Returns IDs of steps reachable from `from_id` via backlinks (depth ignored).
    async fn traverse(&self, from_id: &str, _depth: usize) -> Result<Vec<String>, SoulvaultError> {
        let steps = self
            .inner
            .find_backlinks(from_id)
            .await
            .map_err(|e| SoulvaultError::QueryFailed(e.to_string()))?;
        Ok(steps.into_iter().map(|s| s.id).collect())
    }
}

// ============================================================================
// PromotionBackend
// ============================================================================

#[async_trait]
impl PromotionBackend for SdkHelixAdapter {
    /// Returns the steps unchanged — SDK promotion runs via the consolidation daemon.
    async fn promote(&self, steps: Vec<Step>) -> Result<Vec<Step>, SoulvaultError> {
        Ok(steps)
    }

    /// No-op — SDK tiering runs via the SOUL consolidation pipeline.
    async fn tier_step(&self, _id: &str, _tier: Tier) -> Result<(), SoulvaultError> {
        Ok(())
    }
}

// ============================================================================
// HelixBackend
// ============================================================================

#[async_trait]
impl HelixBackend for SdkHelixAdapter {
    async fn upsert_step(&self, step: Step) -> Result<(), SoulvaultError> {
        let sdk_step = to_sdk_step(step);
        self.inner
            .upsert_step(&sdk_step)
            .await
            .map(|_| ())
            .map_err(|e| SoulvaultError::Backend(e.to_string()))
    }

    async fn delete_step(&self, id: &str) -> Result<(), SoulvaultError> {
        let mut params = BTreeMap::new();
        params.insert("id".into(), serde_json::json!(id));
        self.inner
            .execute_cypher_with_params(
                "MATCH (s:Step {id: $id}) DETACH DELETE s",
                params,
            )
            .await
            .map(|_| ())
            .map_err(|e| SoulvaultError::Backend(e.to_string()))
    }

    async fn upsert_helix(&self, helix: Helix) -> Result<(), SoulvaultError> {
        let sdk_helix = to_sdk_helix(helix);
        self.inner
            .upsert_helix(&sdk_helix)
            .await
            .map(|_| ())
            .map_err(|e| SoulvaultError::Backend(e.to_string()))
    }

    async fn link_steps(&self, link: HelixLink) -> Result<(), SoulvaultError> {
        let sdk_link = to_sdk_link(link);
        self.inner
            .create_link(&sdk_link)
            .await
            .map(|_| ())
            .map_err(|e| SoulvaultError::Backend(e.to_string()))
    }

    async fn get_step(&self, id: &str) -> Result<Option<Step>, SoulvaultError> {
        let steps = self
            .inner
            .get_steps_by_ids(&[id.to_owned()])
            .await
            .map_err(|e| SoulvaultError::QueryFailed(e.to_string()))?;
        Ok(steps.into_iter().next().map(from_sdk_step))
    }

    async fn step_count(&self, helix_id: &str) -> Result<usize, SoulvaultError> {
        let mut params = BTreeMap::new();
        params.insert("helix_id".into(), serde_json::json!(helix_id));
        let records = self
            .inner
            .execute_cypher_with_params(
                "MATCH (s:Step {helix_id: $helix_id}) RETURN count(s) AS count",
                params,
            )
            .await
            .map_err(|e| SoulvaultError::QueryFailed(e.to_string()))?;
        Ok(records
            .first()
            .and_then(|r| r.fields.get("count"))
            .and_then(serde_json::Value::as_u64)
            .map(|n| n as usize)
            .unwrap_or(0))
    }

    async fn retrieve(
        &self,
        query: &str,
        k: usize,
        weights: &SignalWeights,
    ) -> Result<Vec<RetrievalResult>, SoulvaultError> {
        let retriever = HybridRetriever::new(
            Arc::clone(&self.embedder),
            Arc::clone(&self.embedder),
        );
        let config = HybridRetrieverConfig {
            top_k: k as u32,
            mode_override: Some(to_sdk_mode(weights)),
            ..HybridRetrieverConfig::default()
        };
        let opts = SearchOptions::default();
        let sdk_result = retriever
            .search(&*self.inner, query, &opts, &config)
            .await
            .map_err(|e| SoulvaultError::QueryFailed(e.to_string()))?;

        // Hydrate step IDs → full Step objects
        let step_ids: Vec<String> = sdk_result.results.iter().map(|r| r.step_id.clone()).collect();
        if step_ids.is_empty() {
            return Ok(Vec::new());
        }
        let sdk_steps = self
            .inner
            .get_steps_by_ids(&step_ids)
            .await
            .map_err(|e| SoulvaultError::QueryFailed(e.to_string()))?;

        // Build a score lookup and zip with hydrated steps
        let scores: std::collections::HashMap<&str, f64> = sdk_result
            .results
            .iter()
            .map(|r| (r.step_id.as_str(), r.score))
            .collect();

        Ok(sdk_steps
            .into_iter()
            .map(|s| {
                let score = scores.get(s.id.as_str()).copied().unwrap_or(0.0);
                RetrievalResult::new(from_sdk_step(s), score)
            })
            .collect())
    }
}

// ============================================================================
// Type mapping helpers
// ============================================================================

fn to_sdk_step(s: Step) -> sdk_types::Step {
    sdk_types::Step {
        id: s.id,
        helix_id: s.helix_id,
        title: s.title,
        content: s.content,
        significance: 0.0,
        step_date: s
            .step_date
            .as_deref()
            .and_then(|d| d.parse::<chrono::NaiveDate>().ok()),
        step_index: s.step_index,
        community_id: None,
        expires: s.expires.as_deref().and_then(|e| {
            chrono::DateTime::parse_from_rfc3339(e)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        }),
        created_at: Utc::now(),
        metadata: s.metadata,
        vault_path: None,
        graph_embedding: None,
    }
}

fn from_sdk_step(s: sdk_types::Step) -> Step {
    Step {
        id: s.id,
        helix_id: s.helix_id,
        content: s.content,
        title: s.title,
        step_date: s.step_date.map(|d| d.to_string()),
        expires: s.expires.map(|e| e.to_rfc3339()),
        step_index: s.step_index,
        metadata: s.metadata,
    }
}

fn to_sdk_helix(h: Helix) -> sdk_types::Helix {
    let ordering_mode = match h.ordering {
        HelixOrderingMode::Temporal => sdk_types::HelixOrderingMode::Temporal,
        HelixOrderingMode::Indexed => sdk_types::HelixOrderingMode::Indexed,
        HelixOrderingMode::Custom => sdk_types::HelixOrderingMode::Custom,
    };
    let scope_tier = h
        .scope_tier
        .as_deref()
        .and_then(parse_scope_tier)
        .unwrap_or(sdk_types::ScopeTier::User);
    sdk_types::Helix {
        id: h.id,
        owner: "user".to_owned(),
        name: h.name,
        level: 0,
        ordering_mode,
        scope_tier,
        max_depth: None,
        created_at: Utc::now(),
    }
}

fn to_sdk_link(l: HelixLink) -> sdk_types::HelixLink {
    sdk_types::HelixLink {
        source_id: l.source_id,
        target_id: l.target_id,
        link_type: sdk_types::LinkType::Custom(l.link_type),
        strength: l.weight,
        raw_wikilink: None,
        metadata: serde_json::Value::Object(Default::default()),
    }
}

fn parse_scope_tier(s: &str) -> Option<sdk_types::ScopeTier> {
    match s {
        "platform" => Some(sdk_types::ScopeTier::Platform),
        "user" => Some(sdk_types::ScopeTier::User),
        "project" => Some(sdk_types::ScopeTier::Project),
        "shared" => Some(sdk_types::ScopeTier::Shared),
        _ => None,
    }
}

/// Map la-soulvault `SignalWeights` (f64 fractions) to an SDK `RetrievalMode`
/// for config-based mode selection.
fn to_sdk_mode(w: &SignalWeights) -> lightarchitects::helix::soul_search::hybrid::RetrievalMode {
    // Select mode by dominant signal
    if w.bm25 >= 0.5 {
        lightarchitects::helix::soul_search::hybrid::RetrievalMode::KeywordDominated
    } else if w.graph >= 0.4 {
        lightarchitects::helix::soul_search::hybrid::RetrievalMode::GraphWeighted
    } else {
        lightarchitects::helix::soul_search::hybrid::RetrievalMode::Balanced
    }
}

// ============================================================================
// Entry points
// ============================================================================

/// Connect to a SOUL vault using the given config.
///
/// Returns a [`SoulClient`](crate::SoulClient) — a type-erased [`HelixBackend`]
/// that hides all SDK internals.
///
/// # Errors
///
/// Returns [`SoulvaultError::ConnectionFailed`] if the Neo4j connection or
/// schema migration fails.
///
/// # Example
///
/// ```rust,ignore
/// let client = larc_soulvault::vault::connect(VaultConfig::new(
///     "bolt://localhost:7687", "neo4j", "s3cr3t",
/// )).await?;
/// ```
pub async fn connect(config: VaultConfig) -> Result<crate::SoulClient, SoulvaultError> {
    let store = HelixStore::connect(&config.uri, &config.user, &config.password)
        .await
        .map_err(|e| SoulvaultError::ConnectionFailed(e.to_string()))?;
    let inner = store.helix_db();
    let embedder: Arc<dyn EmbeddingProvider> =
        Arc::new(create_embedding_provider(EmbeddingConfig::default()));
    let adapter = SdkHelixAdapter { inner, embedder };
    Ok(crate::SoulClient::new(Box::new(adapter)))
}

/// Connect to a SOUL vault using environment variables.
///
/// Reads `SOUL_VAULT_URI`, `SOUL_VAULT_USER`, `SOUL_VAULT_PASSWORD`.
///
/// # Errors
///
/// Returns [`SoulvaultError::ConfigError`] if any variable is unset, or
/// [`SoulvaultError::ConnectionFailed`] on connection failure.
pub async fn connect_from_env() -> Result<crate::SoulClient, SoulvaultError> {
    connect(VaultConfig::from_env()?).await
}
