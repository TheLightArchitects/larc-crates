//! Contract, verdict, and decision types for quality gates.

use crate::GateDimension;
use serde::{Deserialize, Serialize};

/// File ownership and concurrency contract for a task.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TaskContract {
    /// Files this task is allowed to modify.
    pub file_ownership: Vec<String>,
    /// Whether this task can run concurrently with other tasks.
    pub concurrency_safe: bool,
}

impl TaskContract {
    /// Create a new task contract.
    pub fn new(file_ownership: Vec<String>, concurrency_safe: bool) -> Self {
        Self {
            file_ownership,
            concurrency_safe,
        }
    }
}

/// Gate verdict — the result of a quality gate review.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Verdict {
    /// Gate passed — work meets the quality standard.
    Approve {
        /// Optional approval message.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    /// Gate failed — work does not meet the quality standard.
    Reject {
        /// Which dimensions failed.
        dimensions: Vec<GateDimension>,
        /// Why the gate rejected.
        reason: String,
    },
    /// Gate needs more investigation before deciding.
    Defer {
        /// Which dimensions need investigation.
        dimensions: Vec<GateDimension>,
        /// Why the gate was deferred.
        reason: String,
    },
}

/// A score for a single quality dimension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DimensionScore {
    /// The dimension being scored.
    pub dimension: GateDimension,
    /// Numerical score (0.0–100.0).
    pub score: f64,
    /// Optional findings or notes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl DimensionScore {
    /// Create a new dimension score.
    pub fn new(dimension: GateDimension, score: f64, notes: Option<String>) -> Self {
        Self {
            dimension,
            score,
            notes,
        }
    }
}

/// A gate decision — the final outcome after all reviews.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Decision {
    /// The final verdict.
    pub verdict: Verdict,
    /// Scores per dimension.
    pub scores: Vec<DimensionScore>,
    /// Aggregate score across all dimensions.
    pub aggregate_score: f64,
}

impl Decision {
    /// Create a new gate decision.
    pub fn new(verdict: Verdict, scores: Vec<DimensionScore>, aggregate_score: f64) -> Self {
        Self {
            verdict,
            scores,
            aggregate_score,
        }
    }
}
