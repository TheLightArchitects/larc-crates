//! Storage tier for helix steps.
//!
//! Tier determines the storage backend and retention policy for a step.
//! Meaningful even without any backend trait — data-model-only consumers
//! can describe hot/warm/cold without async.

use serde::{Deserialize, Serialize};

/// Storage tier — determines backend and retention policy.
///
/// | Tier | Backend | Retention |
/// |------|---------|-----------|
/// | Hot | In-memory / SQLite | Current session, instant recall |
/// | Warm | SQLite + FTS5 | Recent, sub-second recall |
/// | Cold | Neo4j / archive | Historical, graph-weighted recall |
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Tier {
    /// In-memory / SQLite — current session, instant recall.
    Hot,
    /// SQLite + FTS5 — recent, sub-second recall.
    Warm,
    /// Neo4j / archive — historical, graph-weighted recall.
    Cold,
}

impl Default for Tier {
    fn default() -> Self {
        Self::Warm
    }
}
