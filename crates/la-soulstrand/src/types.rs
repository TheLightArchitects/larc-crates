/// Core types for la-soulstrand.
///
/// These 5 primitives map directly to the Neo4j label schema:
/// - `Helix` → `:Helix` container nodes
/// - `Step` → `:Step` content atoms
/// - `Strand` → `:Strand` domain axes
/// - `HelixLink` → `[:LINKS_TO]` directed edges
/// - `SharedExperience` → `:SharedExperience` convergence nodes
use serde::{Deserialize, Serialize};

/// Ordering mode for steps within a helix.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HelixOrderingMode {
    /// Order by `step_date` ASC — journals, builds, consciousness entries.
    Temporal,
    /// Order by `step_index` ASC — bible chapters, plan phases.
    Indexed,
    /// Order by metadata key — application-specific.
    Custom,
}

impl Default for HelixOrderingMode {
    fn default() -> Self {
        Self::Temporal
    }
}

/// A container node — sibling root, sub-helix, or domain collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Helix {
    pub id: String,
    pub name: String,
    pub ordering: HelixOrderingMode,
    pub metadata: serde_json::Value,
}

/// A content atom — entry, chapter, log line, document chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub id: String,
    pub helix_id: String,
    pub content: String,
    pub step_date: Option<String>,
    pub step_index: Option<i64>,
    pub metadata: serde_json::Value,
}

/// A domain axis — emotion facet, literary genre, build phase type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strand {
    pub id: String,
    pub name: String,
    pub helix_id: String,
    pub metadata: serde_json::Value,
}

/// A directed edge between steps with type and weight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelixLink {
    pub source_id: String,
    pub target_id: String,
    pub link_type: String,
    pub weight: f64,
}

/// An N-way convergence node with participant edges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedExperience {
    pub id: String,
    pub name: String,
    pub participant_ids: Vec<String>,
    pub metadata: serde_json::Value,
}

/// A single retrieval result with score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    pub step: Step,
    pub score: f64,
}

impl RetrievalResult {
    pub fn id(&self) -> &str {
        &self.step.id
    }

    pub fn score(&self) -> f64 {
        self.score
    }
}
