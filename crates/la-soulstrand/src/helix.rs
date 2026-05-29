//! Neo4j + 4-signal RRF backend (feature gate `helix`).
//!
//! Combines BM25 keyword, semantic HNSW, structural Node2Vec,
//! and graph traversal signals via Reciprocal Rank Fusion.

use crate::{SoulstrandError, RetrievalResult};

/// Adaptive retrieval mode (auto-selected by step count).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrievalMode {
    /// < 25 steps: keyword-dominated (0.65 / 0.25 / 0.03 / 0.07)
    KeywordDominated,
    /// 25-99 steps: balanced (0.25 / 0.35 / 0.10 / 0.30)
    Balanced,
    /// >= 100 steps: graph-weighted (0.15 / 0.30 / 0.10 / 0.45)
    GraphWeighted,
}

/// Signal weights for 4-signal RRF.
#[derive(Debug, Clone)]
pub struct SignalWeights {
    pub bm25: f64,
    pub semantic: f64,
    pub structural: f64,
    pub graph: f64,
}

impl Default for SignalWeights {
    fn default() -> Self {
        Self::balanced()
    }
}

impl SignalWeights {
    pub fn keyword_dominated() -> Self {
        Self { bm25: 0.65, semantic: 0.25, structural: 0.03, graph: 0.07 }
    }

    pub fn balanced() -> Self {
        Self { bm25: 0.25, semantic: 0.35, structural: 0.10, graph: 0.30 }
    }

    pub fn graph_weighted() -> Self {
        Self { bm25: 0.15, semantic: 0.30, structural: 0.10, graph: 0.45 }
    }
}

/// Neo4j-backed knowledge graph storage with 4-signal RRF.
pub struct HelixBackend {
    _inner: (),
}

impl HelixBackend {
    /// Connect to a Neo4j database.
    pub async fn connect(_uri: &str, _user: &str, _password: &str) -> Result<Self, SoulstrandError> {
        todo!("implement Neo4j backend")
    }

    /// Retrieve steps using 4-signal RRF.
    pub async fn retrieve_hybrid(
        &self,
        _query: &str,
        _k: usize,
        _weights: &SignalWeights,
    ) -> Result<Vec<RetrievalResult>, SoulstrandError> {
        todo!("implement 4-signal RRF")
    }
}

/// Hybrid retriever that combines SQLite BM25 with Neo4j 4-signal RRF.
pub struct HybridRetriever {
    _inner: (),
}

impl HybridRetriever {
    /// Select retrieval mode based on step count.
    pub fn select_mode(step_count: usize) -> RetrievalMode {
        match step_count {
            0..=24 => RetrievalMode::KeywordDominated,
            25..=99 => RetrievalMode::Balanced,
            _ => RetrievalMode::GraphWeighted,
        }
    }
}