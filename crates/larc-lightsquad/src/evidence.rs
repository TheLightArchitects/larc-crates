//! Correction-loop evidence bundle — Rust companion to §57.2b `evidence-bundle.json`.

use crate::critic::Vulnerability;
use crate::pipeline::CompilerDiagnostic;
use serde::{Deserialize, Serialize};

/// Structured evidence artifact for a fix-agent correction iteration.
///
/// Rust companion to the §57.2b `evidence-bundle.json` canonical schema.
/// Replaces the ad-hoc "cleaned stack trace" string with a structured,
/// allowlist-parsed, AYIN-observable record.
///
/// The TypeScript counterpart lives at
/// `lightarchitects-webshell-ui/e2e/lib/artifacts.ts`.
///
/// # §57.2b field mapping
///
/// | §57.2b field | This field | Notes |
/// |---|---|---|
/// | `test_name` | `test_name` | exact |
/// | `assertion_text` | `assertion_text` | exact |
/// | last 10 SSE frames | `last_sse_frames` | capped at 10 entries |
/// | artifact paths | `artifact_paths` | exact |
/// | AYIN span summary | `ayin_span_id` | UUID string; fetch detail via `:3742/api/spans/{id}` |
/// | *(added)* | `cargo_test_failures` | allowlist-parsed cargo test errors (§3.4.1) |
/// | *(added)* | `loop_index` | 0-indexed iteration for Coder aggression calibration |
/// | *(added)* | `critic_vulnerabilities` | Reviewer→Coder findings (Gap A closure) |
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EvidenceBundle {
    /// Name of the failing test.
    pub test_name: String,
    /// Text of the assertion that failed.
    pub assertion_text: String,
    /// Zero-indexed correction iteration (matches [`SanitizedTrace::loop_index`]).
    ///
    /// [`SanitizedTrace::loop_index`]: crate::pipeline::SanitizedTrace::loop_index
    pub loop_index: u8,
    /// Allowlist-parsed compiler and test-runner failures.
    ///
    /// Replaces the bespoke "cleaned stack trace" string. Each entry was
    /// produced by a structural allowlist parser (Security Guardrails §3.4.1).
    #[serde(default)]
    pub cargo_test_failures: Vec<CompilerDiagnostic>,
    /// AYIN span ID for this correction iteration (UUID v4 string).
    ///
    /// `None` when AYIN is not wired into the pipeline. When `Some`, the
    /// consumer may fetch the full span summary via `GET :3742/api/spans/{id}`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ayin_span_id: Option<String>,
    /// Paths to supporting artifacts (test logs, snapshots, generated files).
    #[serde(default)]
    pub artifact_paths: Vec<String>,
    /// Last ≤10 SSE frames from the test session (per §57.2b).
    #[serde(default)]
    pub last_sse_frames: Vec<String>,
    /// Reviewer-identified vulnerabilities for this correction iteration.
    ///
    /// Populated when the Critic rejected in the same loop iteration.
    /// Empty when the correction was triggered by compiler/test failures only.
    /// Closes the Reviewer→Coder feedback path — vulnerabilities and compiler
    /// failures share a single correction envelope (Gap A, SCRUM gleaming-marble).
    #[serde(default)]
    pub critic_vulnerabilities: Vec<Vulnerability>,
}

impl EvidenceBundle {
    /// Create a minimal evidence bundle with the required fields.
    #[must_use]
    pub fn new(test_name: String, assertion_text: String, loop_index: u8) -> Self {
        Self {
            test_name,
            assertion_text,
            loop_index,
            cargo_test_failures: Vec::new(),
            ayin_span_id: None,
            artifact_paths: Vec::new(),
            last_sse_frames: Vec::new(),
            critic_vulnerabilities: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::CompilerDiagnostic;

    #[test]
    fn evidence_bundle_roundtrip_json() {
        let mut bundle = EvidenceBundle::new(
            "test_fetch_user".into(),
            "assert_eq!(result.id, 42)".into(),
            0,
        );
        bundle
            .cargo_test_failures
            .push(CompilerDiagnostic::with_error_code(
                "src/db.rs".into(),
                18,
                "E0277".into(),
                "the trait `Send` is not implemented".into(),
            ));
        bundle.ayin_span_id = Some("550e8400-e29b-41d4-a716-446655440000".into());

        let json = serde_json::to_string(&bundle).unwrap();
        let back: EvidenceBundle = serde_json::from_str(&json).unwrap();
        assert_eq!(back.loop_index, 0);
        assert_eq!(back.cargo_test_failures.len(), 1);
        assert_eq!(
            back.ayin_span_id.as_deref(),
            Some("550e8400-e29b-41d4-a716-446655440000")
        );
    }

    #[test]
    fn evidence_bundle_ayin_span_omitted_when_none() {
        let bundle = EvidenceBundle::new("t".into(), "a".into(), 2);
        let json = serde_json::to_string(&bundle).unwrap();
        assert!(
            !json.contains("ayin_span_id"),
            "None field should be omitted"
        );
    }
}
