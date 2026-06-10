//! Quality gate dimensions for the Structured Delivery Protocol.
//!
//! Ten canonical dimensions covering Architecture, Security, Quality, Canon,
//! Operations, Performance, Knowledge, Documentation, Testing, and Research.
//! Each dimension corresponds to a quality gate that can be reviewed
//! independently. Add custom dimensions via [`GateDimension::Custom`].

use serde::{Deserialize, Serialize};

/// Quality gate dimension.
///
/// The ten canonical dimensions [A+S+Q+C+O+P+K+D+T+R] plus
/// a `Custom` variant for domain-specific extensions.
///
/// | Dimension | Owner | Scope |
/// |-----------|-------|-------|
/// | Architecture | CORSO | System design, API contracts |
/// | Security | SERAPH | Threat surface, vulnerabilities |
/// | Quality | CORSO | Standards, clippy, fmt |
/// | Canon | LÆX | Standards compliance |
/// | Operations | EVA | Deploy pipeline, CI/CD |
/// | Performance | EVA | Latency, throughput |
/// | Knowledge | SOUL | Documentation, citations |
/// | Documentation | SOUL | Rustdoc, examples |
/// | Testing | CORSO | Test pyramid, coverage |
/// | Research | QUANTUM | Prior art, risk scoring |
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum GateDimension {
    /// [A] Architecture — system design, API contracts, complexity.
    Architecture,
    /// [S] Security — threat surface, vulnerabilities, supply chain.
    Security,
    /// [Q] Quality — standards, clippy, fmt, complexity limits.
    Quality,
    /// [C] Canon — standards compliance, cross-reference consistency.
    Canon,
    /// [O] Operations — deploy pipeline, CI/CD, rollback.
    Operations,
    /// [P] Performance — latency, throughput, resource bounds.
    Performance,
    /// [K] Knowledge — documentation, citations, prior decisions.
    Knowledge,
    /// [D] Documentation — rustdoc, examples, in-code docs.
    Documentation,
    /// [T] Testing — test pyramid, coverage ≥90%, regression.
    Testing,
    /// [R] Research — prior art, dependency audit, risk scoring.
    Research,
    /// Custom dimension for domain-specific quality gates.
    Custom(String),
}

impl GateDimension {
    /// Short code for this dimension (e.g., "A", "S", "Q").
    #[must_use]
    pub fn code(&self) -> &str {
        match self {
            Self::Architecture => "A",
            Self::Security => "S",
            Self::Quality => "Q",
            Self::Canon => "C",
            Self::Operations => "O",
            Self::Performance => "P",
            Self::Knowledge => "K",
            Self::Documentation => "D",
            Self::Testing => "T",
            Self::Research => "R",
            Self::Custom(_) => "?",
        }
    }
}
