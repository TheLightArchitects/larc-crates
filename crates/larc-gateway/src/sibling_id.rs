//! Sibling identity types.

use serde::{Deserialize, Serialize};

/// Canonical sibling identifiers.
///
/// Each variant names a sibling binary the gateway can dispatch to.
/// The identifier is the protocol-level handle; the binary itself is
/// supplied by the consumer at runtime.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SiblingId {
    /// CORSO — AppSec and code-quality orchestration.
    Corso,
    /// EVA — DevOps, CI/CD, and operational memory.
    Eva,
    /// SOUL — Knowledge graph and semantic retrieval.
    Soul,
    /// QUANTUM — Forensic investigation and risk analysis.
    Quantum,
    /// SERAPH — Red team and offensive security.
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
