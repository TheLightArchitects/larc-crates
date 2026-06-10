//! Correction-loop evidence bundle.

use crate::critic::Vulnerability;
use crate::pipeline::CompilerDiagnostic;
use serde::{Deserialize, Serialize};

/// Structured evidence artifact for a fix-agent correction iteration.
///
/// A correction-loop record carrying the failing test name, the assertion that
/// failed, allowlist-parsed compiler/test diagnostics, reviewer findings, and
/// pointers to supporting artifacts. Replaces ad-hoc "stack trace" strings with
/// a typed, deterministic input for the next correction step.
///
/// # Fields
///
/// | Field | Purpose |
/// |---|---|
/// | `test_name` | Name of the failing test. |
/// | `assertion_text` | Text of the assertion that failed. |
/// | `loop_index` | 0-indexed correction iteration (for aggression calibration). |
/// | `cargo_test_failures` | Allowlist-parsed compiler/test errors. |
/// | `ayin_span_id` | Trace span UUID for correlated observability. |
/// | `critic_vulnerabilities` | Reviewer-identified findings for this iteration. |
/// | `artifact_paths` | Paths to supporting artifacts (logs, snapshots). |
/// | `last_sse_frames` | Last ≤10 SSE frames from the test session. |
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
    /// Each entry is the output of a structural allowlist parser (no raw
    /// strings, no ANSI escapes, no role-override tokens). See
    /// [`CompilerDiagnostic`] for the field shape.
    #[serde(default)]
    pub cargo_test_failures: Vec<CompilerDiagnostic>,
    /// Trace span ID for this correction iteration (UUID v4 string).
    ///
    /// `None` when no tracing backend is wired into the pipeline. When `Some`,
    /// the consumer may correlate this iteration to the originating trace.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ayin_span_id: Option<String>,
    /// Paths to supporting artifacts (test logs, snapshots, generated files).
    #[serde(default)]
    pub artifact_paths: Vec<String>,
    /// Last ≤10 SSE frames from the test session.
    #[serde(default)]
    pub last_sse_frames: Vec<String>,
    /// Reviewer-identified vulnerabilities for this correction iteration.
    ///
    /// Populated when the Critic rejected in the same loop iteration.
    /// Empty when the correction was triggered by compiler/test failures only.
    /// Carrying vulnerabilities and compiler failures in a single envelope
    /// lets the Coder treat both classes of feedback uniformly.
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
