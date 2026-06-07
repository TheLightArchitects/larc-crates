//! Retrieval result types and convex-combination signal weights.
//!
//! These types are used by [`HelixBackend`] and are gated behind the `helix`
//! feature together with it.
//!
//! [`HelixBackend`]: crate::HelixBackend

use crate::Step;
use serde::{Deserialize, Serialize};

/// A single retrieval result with relevance score.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub struct RetrievalResult {
    /// The retrieved step.
    pub step: Step,
    /// Combined convex-combination relevance score — higher is more relevant.
    pub score: f64,
}

impl RetrievalResult {
    /// Create a new retrieval result.
    pub fn new(step: Step, score: f64) -> Self {
        Self { step, score }
    }

    /// Step ID.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.step.id
    }

    /// Relevance score.
    #[must_use]
    pub fn score(&self) -> f64 {
        self.score
    }
}

/// Adaptive retrieval mode — auto-selected by [`select_mode`].
///
/// [`select_mode`]: crate::select_mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum RetrievalMode {
    /// < 25 steps — BM25 weight 0.65, semantic 0.25, graph 0.07, structural 0.03.
    KeywordDominated,
    /// 25–99 steps — BM25 0.25, semantic 0.35, graph 0.30, structural 0.10.
    Balanced,
    /// ≥ 100 steps — BM25 0.15, semantic 0.30, graph 0.45, structural 0.10.
    GraphWeighted,
}

/// Signal weights for 4-signal convex combination fusion.
///
/// Weights should sum to 1.0. Use the named constructors for the three
/// adaptive presets, or [`SignalWeights::new`] for custom weights.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SignalWeights {
    /// BM25 fulltext keyword signal.
    pub bm25: f64,
    /// Semantic embedding similarity signal.
    pub semantic: f64,
    /// Graph traversal signal.
    pub graph: f64,
    /// Structural Node2Vec signal.
    pub structural: f64,
}

impl Default for SignalWeights {
    fn default() -> Self {
        Self::balanced()
    }
}

impl SignalWeights {
    /// Create custom weights for experimentation.
    pub fn new(bm25: f64, semantic: f64, graph: f64, structural: f64) -> Self {
        Self {
            bm25,
            semantic,
            graph,
            structural,
        }
    }

    /// Keyword-dominated preset (`RetrievalMode::KeywordDominated`).
    #[must_use]
    pub fn keyword_dominated() -> Self {
        Self {
            bm25: 0.65,
            semantic: 0.25,
            graph: 0.07,
            structural: 0.03,
        }
    }

    /// Balanced preset (`RetrievalMode::Balanced`).
    #[must_use]
    pub fn balanced() -> Self {
        Self {
            bm25: 0.25,
            semantic: 0.35,
            graph: 0.30,
            structural: 0.10,
        }
    }

    /// Graph-weighted preset (`RetrievalMode::GraphWeighted`).
    #[must_use]
    pub fn graph_weighted() -> Self {
        Self {
            bm25: 0.15,
            semantic: 0.30,
            graph: 0.45,
            structural: 0.10,
        }
    }

    /// Return the preset weights for a given [`RetrievalMode`].
    #[must_use]
    pub fn for_mode(mode: RetrievalMode) -> Self {
        match mode {
            RetrievalMode::KeywordDominated => Self::keyword_dominated(),
            RetrievalMode::Balanced => Self::balanced(),
            RetrievalMode::GraphWeighted => Self::graph_weighted(),
        }
    }
}
