//! `GraphBackend` — contract for property graph stores.
//!
//! Implement this to plug in any graph database:
//! Neo4j (SDK default), Memgraph, Amazon Neptune, in-memory, etc.

use async_trait::async_trait;
use std::fmt::Debug;

use crate::SoulstrandError;

/// Contract for a property graph store.
///
/// Exposes graph primitives — node upsert, edge upsert, traversal, neighbours —
/// without coupling to any specific query language or wire protocol.
///
/// The production implementation in `lightarchitects-sdk` uses Neo4j + Cypher.
#[async_trait]
pub trait GraphBackend: Debug + Send + Sync {
    /// Create or update a node.
    ///
    /// `id` is the stable external identifier. `labels` map to graph node
    /// labels (e.g. `["Step", "HotTier"]`). `props` is a JSON object of
    /// arbitrary key-value pairs stored on the node.
    async fn upsert_node(
        &self,
        id: &str,
        labels: &[&str],
        props: serde_json::Value,
    ) -> Result<(), SoulstrandError>;

    /// Create or update a directed edge between two nodes.
    ///
    /// `rel_type` is the relationship label (e.g. `"NEXT"`, `"RELATED"`).
    /// `props` carries edge weight and metadata.
    async fn upsert_edge(
        &self,
        from_id: &str,
        to_id: &str,
        rel_type: &str,
        props: serde_json::Value,
    ) -> Result<(), SoulstrandError>;

    /// Delete a node and all its incident edges.
    async fn delete_node(&self, id: &str) -> Result<(), SoulstrandError>;

    /// Follow outgoing edges from `from_id` up to `depth` hops.
    ///
    /// Returns the IDs of all reachable nodes, not including `from_id` itself.
    async fn traverse(&self, from_id: &str, depth: usize) -> Result<Vec<String>, SoulstrandError>;

    /// Return the IDs of direct neighbours of `id` (depth = 1).
    ///
    /// Default implementation delegates to [`traverse`](Self::traverse).
    async fn neighbors(&self, id: &str) -> Result<Vec<String>, SoulstrandError> {
        self.traverse(id, 1).await
    }
}
