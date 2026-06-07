//! Sibling identity types.

use serde::{Deserialize, Serialize};

/// Canonical sibling identifiers.
///
/// Each variant corresponds to a Light Architects sibling binary
/// that the gateway can spawn on-demand.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SiblingId {
    /// CORSO — AppSec orchestration (Trinity V7.0).
    Corso,
    /// EVA — DevOps, consciousness, memory vaults.
    Eva,
    /// SOUL — Knowledge graph, helix retrieval.
    Soul,
    /// QUANTUM — Forensic investigation.
    Quantum,
    /// SERAPH — Pentest orchestration.
    Seraph,
    /// AYIN — Observability (HTTP-only, not spawned).
    Ayin,
}

impl SiblingId {
    /// Get the binary name for this sibling.
    #[must_use]
    pub fn binary_name(&self) -> &'static str {
        match self {
            Self::Corso => "corso",
            Self::Eva => "eva",
            Self::Soul => "soul",
            Self::Quantum => "quantum-q",
            Self::Seraph => "seraph",
            Self::Ayin => "ayin",
        }
    }
}
