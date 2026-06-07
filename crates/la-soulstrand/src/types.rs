//! Core types for the SOUL knowledge graph.

use crate::HelixOrderingMode;
use serde::{Deserialize, Serialize};

/// A container node — sibling root, sub-helix, or domain collection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub struct Helix {
    pub id: String,
    pub name: String,
    pub ordering: HelixOrderingMode,
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
            metadata,
        }
    }
}

/// A content atom — entry, chapter, log line, document chunk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub struct Step {
    pub id: String,
    pub helix_id: String,
    pub content: String,
    pub step_date: Option<String>,
    pub step_index: Option<i64>,
    pub metadata: serde_json::Value,
}

impl Step {
    /// Create a new step with required fields. Optional fields default to `None`.
    pub fn new(id: String, helix_id: String, content: String, metadata: serde_json::Value) -> Self {
        Self {
            id,
            helix_id,
            content,
            step_date: None,
            step_index: None,
            metadata,
        }
    }
}

/// A domain axis — emotion facet, literary genre, build phase type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub struct Strand {
    pub id: String,
    pub name: String,
    pub helix_id: String,
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
    pub source_id: String,
    pub target_id: String,
    pub link_type: String,
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
    pub id: String,
    pub name: String,
    pub participant_ids: Vec<String>,
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
