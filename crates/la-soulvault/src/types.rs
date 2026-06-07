//! Core types for the SOUL knowledge graph.

use crate::HelixOrderingMode;
use serde::{Deserialize, Serialize};

/// A container node — sibling root, sub-helix, or domain collection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub struct Helix {
    /// Unique identifier for this helix node.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Traversal ordering mode — controls how steps are ranked during retrieval.
    pub ordering: HelixOrderingMode,
    /// Scope tier — governs write semantics (`"platform"`, `"user"`, `"project"`, `"shared"`).
    ///
    /// `None` defaults to `"user"` scope in implementations that enforce tier semantics.
    pub scope_tier: Option<String>,
    /// Arbitrary JSON metadata for caller-defined attributes.
    pub metadata: serde_json::Value,
}

impl Helix {
    /// Create a new helix node.
    pub fn new(
        id: String,
        name: String,
        ordering: HelixOrderingMode,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            id,
            name,
            ordering,
            scope_tier: None,
            metadata,
        }
    }
}

/// A content atom — entry, chapter, log line, document chunk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub struct Step {
    /// Unique identifier for this step.
    pub id: String,
    /// Parent helix that owns this step.
    pub helix_id: String,
    /// Text content of the step.
    pub content: String,
    /// Optional short title summarising the step content.
    pub title: Option<String>,
    /// Optional ISO-8601 date string associated with the event or creation time.
    pub step_date: Option<String>,
    /// Optional ISO-8601 expiry timestamp — after this instant the step may be
    /// pruned by the consolidation pipeline.
    pub expires: Option<String>,
    /// Optional ordinal position within the helix — used for sequence-ordered helixes.
    pub step_index: Option<i64>,
    /// Arbitrary JSON metadata for caller-defined attributes.
    pub metadata: serde_json::Value,
}

impl Step {
    /// Create a new step with required fields. Optional fields default to `None`.
    pub fn new(id: String, helix_id: String, content: String, metadata: serde_json::Value) -> Self {
        Self {
            id,
            helix_id,
            content,
            title: None,
            step_date: None,
            expires: None,
            step_index: None,
            metadata,
        }
    }
}

/// A domain axis — emotion facet, literary genre, build phase type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub struct Strand {
    /// Unique identifier for this strand.
    pub id: String,
    /// Human-readable name for the domain axis.
    pub name: String,
    /// Parent helix that owns this strand.
    pub helix_id: String,
    /// Arbitrary JSON metadata for caller-defined attributes.
    pub metadata: serde_json::Value,
}

impl Strand {
    /// Create a new strand.
    pub fn new(id: String, name: String, helix_id: String, metadata: serde_json::Value) -> Self {
        Self {
            id,
            name,
            helix_id,
            metadata,
        }
    }
}

/// A directed edge between steps with type and weight.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub struct HelixLink {
    /// Step ID of the edge origin.
    pub source_id: String,
    /// Step ID of the edge destination.
    pub target_id: String,
    /// Semantic type label for the relationship.
    ///
    /// Canonical values used by the Light Architects SDK:
    /// - `"HAS_STEP"` — helix → step containment
    /// - `"REFERENCES"` — step cites another step
    /// - `"FOLLOWS"` — sequential ordering between steps
    /// - `"SHARED_EXPERIENCE"` — step participates in a convergence node
    /// - `"STRAND_MEMBER"` — step belongs to a domain strand
    ///
    /// Custom implementations may define their own labels. The string form is
    /// intentional — it allows third-party `HelixBackend` implementors to extend
    /// the relationship vocabulary without forking this crate.
    pub link_type: String,
    /// Edge weight in `[0.0, 1.0]` — higher is stronger.
    pub weight: f64,
}

impl HelixLink {
    /// Create a new helix link.
    pub fn new(source_id: String, target_id: String, link_type: String, weight: f64) -> Self {
        Self {
            source_id,
            target_id,
            link_type,
            weight,
        }
    }
}

/// An N-way convergence node with participant edges.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub struct SharedExperience {
    /// Unique identifier for this shared experience node.
    pub id: String,
    /// Human-readable name describing the convergence.
    pub name: String,
    /// IDs of the steps or entities that participate in this experience.
    pub participant_ids: Vec<String>,
    /// Arbitrary JSON metadata for caller-defined attributes.
    pub metadata: serde_json::Value,
}

impl SharedExperience {
    /// Create a new shared experience.
    pub fn new(
        id: String,
        name: String,
        participant_ids: Vec<String>,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            id,
            name,
            participant_ids,
            metadata,
        }
    }
}
