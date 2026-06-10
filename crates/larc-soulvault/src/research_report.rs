//! Research report — structured output of the Researcher agent pipeline stage.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A code snippet verified against a canonical source.
///
/// Carries a checksum of the source that produced the snippet, enabling
/// downstream agents to detect tampering or source drift. Part of the
/// [`ResearchReport`] provenance chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct VerifiedSnippet {
    /// The verified code snippet.
    pub code: String,
    /// SHA-256 hex digest of the source page or crates.io manifest that
    /// produced this snippet. Allows downstream agents to verify provenance.
    ///
    /// Snippets without a checksum MUST be marked `[UNVERIFIED_SOURCE]`
    /// by the assembler (Security Guardrails §3.4.1 extension).
    pub source_checksum: String,
    /// Programming language of the snippet (e.g. `"rust"`, `"toml"`).
    pub language: String,
}

impl VerifiedSnippet {
    /// Create a new verified snippet.
    #[must_use]
    pub fn new(code: String, source_checksum: String, language: String) -> Self {
        Self {
            code,
            source_checksum,
            language,
        }
    }
}

/// Structured output of the Researcher agent — first stage of the supervised
/// pipeline.
///
/// Formalises the untyped research report described in LASDLC §1.8 into a
/// machine-readable artifact. Prose reports degrade silently under token
/// pressure; typed fields do not (Cookbook §66, QUANTUM Round 2 finding).
///
/// # Knowledge graph placement
///
/// Placed in `larc-soulvault` because `ResearchReport` is a knowledge
/// retrieval artifact. Helix schema: `type: research_report`, under
/// `helix/shared/research/{pipeline_run_id}.md`.
///
/// # Provenance chain
///
/// Each [`VerifiedSnippet`] carries a `source_checksum`. Snippets without
/// a checksum must be labelled `[UNVERIFIED_SOURCE]` by the assembler —
/// an unverified report reaching the Coder agent is an injection surface
/// (Security Guardrails §3.4.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ResearchReport {
    /// UUID v4 string identifying the pipeline run that produced this report.
    pub pipeline_run_id: String,
    /// Canonical source queried (crates.io URL, docs.rs page, etc.).
    pub source_uri: String,
    /// Exact locked crate versions relevant to this research context
    /// (`crate_name` → `"x.y.z"`).
    ///
    /// Should match or be a subset of the pipeline's
    /// `EnvironmentManifest::strict_lockfile` (from `larc-lightsquad`).
    #[serde(default)]
    pub version_constraints: HashMap<String, String>,
    /// Code snippets verified against the canonical source.
    #[serde(default)]
    pub verified_snippets: Vec<VerifiedSnippet>,
    /// Hypotheses considered and rejected during research.
    ///
    /// Preserves the Researcher's negative findings so the Coder agent
    /// does not re-explore eliminated paths.
    #[serde(default)]
    pub rejected_hypotheses: Vec<String>,
    /// Approximate token count consumed producing this report.
    pub token_budget_used: u32,
}

impl ResearchReport {
    /// Create a new research report with the minimum required fields.
    #[must_use]
    pub fn new(pipeline_run_id: String, source_uri: String, token_budget_used: u32) -> Self {
        Self {
            pipeline_run_id,
            source_uri,
            version_constraints: HashMap::new(),
            verified_snippets: Vec::new(),
            rejected_hypotheses: Vec::new(),
            token_budget_used,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn research_report_roundtrip_json() {
        let mut report = ResearchReport::new(
            "550e8400-e29b-41d4-a716-446655440000".into(),
            "https://docs.rs/sqlx/0.7.4/sqlx/".into(),
            1_200,
        );
        report
            .version_constraints
            .insert("sqlx".into(), "0.7.4".into());
        report.verified_snippets.push(VerifiedSnippet::new(
            "query!(\"SELECT id FROM users\")".into(),
            "abc123def456".into(),
            "rust".into(),
        ));
        report
            .rejected_hypotheses
            .push("sqlx::query_as requires Debug impl".into());

        let json = serde_json::to_string(&report).unwrap();
        let back: ResearchReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.token_budget_used, 1_200);
        assert_eq!(
            back.version_constraints.get("sqlx").map(String::as_str),
            Some("0.7.4")
        );
        assert_eq!(back.verified_snippets.len(), 1);
        assert_eq!(back.rejected_hypotheses.len(), 1);
    }
}
