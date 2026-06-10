//! Critic/Reviewer agent output types — structured review with calibration baseline.
//!
//! Closes SCRUM gaps G9 (Critic calibration protocol) and G11 (Reviewer findings
//! helix write path). These types define the Negative Bias Reviewer's structured
//! output contract and the finding lifecycle that feeds the SOUL knowledge graph.

use serde::{Deserialize, Serialize};

// ── Finding status ────────────────────────────────────────────────────────────

/// Lifecycle status of a single reviewer finding.
///
/// Determines whether the finding is routed to the SOUL helix vault.
/// Findings with recurrence ≥2 across pipeline runs and `status` of
/// `Actioned` or `Bypassed` are auto-promoted to helix entries at
/// significance ≥6.0 (SOUL Round 2 finding, SCRUM gleaming-marble).
///
/// `Waived` findings are preserved with a rationale to prevent
/// helix pollution from false positives — a bypassed finding without
/// rationale is indistinguishable from a real unresolved finding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FindingStatus {
    /// Finding is newly surfaced; no action taken yet.
    Pending,
    /// Fix was applied — the code no longer triggers this finding.
    Actioned,
    /// Finding was deliberately accepted; rationale recorded.
    Waived {
        /// Why this finding was accepted without a fix.
        rationale: String,
    },
    /// Finding was detected but correction loop did not address it.
    ///
    /// `Bypassed` findings with recurrence ≥2 are escalated to HITL
    /// (agents-playbook §HITL-7) and promoted to the helix as knowledge
    /// entries flagging the unresolved anti-pattern.
    Bypassed,
}

// ── Vulnerability taxonomy ────────────────────────────────────────────────────

/// Category of a reviewer-identified vulnerability or code quality issue.
///
/// The Critic agent evaluates code via a negative-bias rubric. Each `kind`
/// maps to one rubric dimension, enabling QUANTUM's calibration baseline to
/// compute per-category rejection rates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum VulnerabilityKind {
    /// Violation of a [`ConstraintManifest`] hard bound (e.g. `.unwrap()` found).
    ///
    /// [`ConstraintManifest`]: crate::pipeline::ConstraintManifest
    ConstraintViolation,
    /// Unclosed socket, un-dropped lock, or unreleased resource.
    ResourceLeak,
    /// Direct index access on a potentially empty collection without bounds check.
    NullOrOutOfBounds,
    /// Implicit integer overflow, underflow, or wrap in release mode.
    ArithmeticOverflow,
    /// Timing-dependent logic that may create a TOCTOU or race condition.
    RaceCondition,
    /// Category not covered by the standard rubric.
    Custom(String),
}

/// A single vulnerability or quality issue identified by the Critic agent.
///
/// Line-level attribution is load-bearing — prose-only findings are not
/// actionable (CORSO Round 1 finding, SCRUM gleaming-marble).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Vulnerability {
    /// Line number where the issue was identified (1-indexed).
    pub line: u32,
    /// Category of the finding.
    pub kind: VulnerabilityKind,
    /// Human-readable description of the specific issue.
    pub detail: String,
    /// Lifecycle status of this finding.
    ///
    /// Set by the pipeline orchestrator after the Coder processes the review.
    /// Only `Actioned` and `Waived` findings are eligible for SOUL helix routing.
    #[serde(default)]
    pub status: FindingStatus,
    /// Number of times this finding pattern appeared in prior pipeline runs.
    ///
    /// `0` = first occurrence (Critic just surfaced it). Set by the orchestrator
    /// after querying the SOUL helix for recurrence history. Findings with
    /// `recurrence >= 2 && status == FindingStatus::Bypassed` are auto-promoted
    /// to the helix at significance ≥6.0 (SOUL G11 finding, SCRUM gleaming-marble).
    #[serde(default)]
    pub recurrence: u32,
    /// UUID v4 string of the pipeline run that produced this finding.
    ///
    /// Links the finding to its originating run for cross-run recurrence
    /// tracking in the SOUL knowledge graph. `None` when the Critic creates
    /// the finding; set by the orchestrator before helix routing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pipeline_run_id: Option<String>,
}

impl Vulnerability {
    /// Create a new pending vulnerability finding.
    #[must_use]
    pub fn new(line: u32, kind: VulnerabilityKind, detail: String) -> Self {
        Self {
            line,
            kind,
            detail,
            status: FindingStatus::Pending,
            recurrence: 0,
            pipeline_run_id: None,
        }
    }
}

impl Default for FindingStatus {
    fn default() -> Self {
        Self::Pending
    }
}

// ── Critic calibration ────────────────────────────────────────────────────────

/// A single few-shot calibration exemplar for the Critic agent.
///
/// Per QUANTUM Round 2 finding: a Critic with no rejection baseline and no
/// exemplars is a single-source inference engine with unknown false-positive
/// and false-negative rates. At minimum three exemplars are required — one
/// that should be approved, one that should be rejected, one that should
/// escalate (QUANTUM Round 1, SCRUM gleaming-marble).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CriticExemplar {
    /// Short description of the code scenario being evaluated.
    pub input_summary: String,
    /// Expected `approved` verdict for this exemplar.
    pub expected_approved: bool,
    /// Expected findings for this exemplar (empty for approval cases).
    #[serde(default)]
    pub expected_vulnerabilities: Vec<Vulnerability>,
    /// Optional label for this exemplar (e.g. `"approval"`, `"rejection"`, `"escalation"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl CriticExemplar {
    /// Create a new calibration exemplar.
    #[must_use]
    pub fn new(input_summary: String, expected_approved: bool) -> Self {
        Self {
            input_summary,
            expected_approved,
            expected_vulnerabilities: Vec::new(),
            label: None,
        }
    }
}

/// Calibration configuration for the Critic agent.
///
/// Provides few-shot exemplars and a rejection baseline floor to ensure the
/// Critic can be audited for correctness. A Critic with `rejection_count_this_session`
/// below the baseline triggers a calibration warning surfaced in the build dashboard.
///
/// # Minimum requirements (QUANTUM finding)
///
/// - At least 3 exemplars: one approval, one rejection, one escalation
/// - `rejection_baseline_min` > 0.0 (a 0% floor is equivalent to no baseline)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CriticCalibration {
    /// Few-shot exemplars grounding the Critic's evaluation.
    ///
    /// Minimum 3. Order: approval first, then rejection, then escalation.
    pub exemplars: Vec<CriticExemplar>,
    /// Minimum expected rejection rate across the session (0.0–1.0).
    ///
    /// If `rejection_count_this_session / total_reviews < rejection_baseline_min`,
    /// the pipeline surfaces a calibration warning. A rate of 0.0 disables
    /// the check; values above 0.05 are recommended for non-trivial codebases.
    pub rejection_baseline_min: f32,
}

impl CriticCalibration {
    /// Create a calibration configuration.
    #[must_use]
    pub fn new(exemplars: Vec<CriticExemplar>, rejection_baseline_min: f32) -> Self {
        Self {
            exemplars,
            rejection_baseline_min,
        }
    }
}

// ── Critic review output ──────────────────────────────────────────────────────

/// Structured output of the Critic (Negative Bias Reviewer) agent.
///
/// The Critic reads code with the assumption that it is broken and applies a
/// fixed evaluation matrix (CORSO Round 1, SCRUM gleaming-marble):
///
/// 1. Does any finding violate a `ConstraintManifest` hard bound?
/// 2. Are there resource leaks (unclosed sockets, un-dropped locks)?
/// 3. Does the code handle empty, null, or out-of-bounds collection access?
///
/// The `rejection_count_this_session` and `total_reviews_this_session` fields
/// enable QUANTUM's calibration protocol — a Critic that never rejects is
/// indistinguishable from no Critic. The orchestrator populates both counters
/// and computes `calibration_alert` by comparing the rejection rate against
/// `CriticCalibration::rejection_baseline_min`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CriticReview {
    /// Whether the Critic approves this code for the next pipeline stage.
    ///
    /// `false` → findings returned to the Coder agent via the correction loop.
    /// `true` with non-empty `vulnerabilities` is valid — warnings may be
    /// recorded even on approval.
    pub approved: bool,
    /// Identified vulnerabilities and quality issues.
    ///
    /// Non-empty even on approval when `approved = true` with severity ≤ WARNING.
    #[serde(default)]
    pub vulnerabilities: Vec<Vulnerability>,
    /// Number of reviews rejected in this pipeline session.
    ///
    /// Incremented by the orchestrator. Combined with `total_reviews_this_session`
    /// to compute the rejection rate for calibration checking.
    #[serde(default)]
    pub rejection_count_this_session: u32,
    /// Total number of code reviews evaluated in this pipeline session.
    ///
    /// Maintained by the orchestrator and injected into each `CriticReview`.
    /// Denominator for the rejection rate: `rejection_count / total_reviews`.
    #[serde(default)]
    pub total_reviews_this_session: u32,
    /// Calibration warning, if the Critic's rejection rate is below baseline.
    ///
    /// `None` = calibration satisfied (or `CriticCalibration` not wired).
    /// `Some(msg)` = the rejection rate fell below
    /// `CriticCalibration::rejection_baseline_min`; the pipeline continues but
    /// the operator is notified via the build dashboard. A Critic that never
    /// rejects is indistinguishable from no Critic (QUANTUM Round 2 finding).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calibration_alert: Option<String>,
}

impl CriticReview {
    /// Create an approval with no findings.
    #[must_use]
    pub fn approved() -> Self {
        Self {
            approved: true,
            vulnerabilities: Vec::new(),
            rejection_count_this_session: 0,
            total_reviews_this_session: 0,
            calibration_alert: None,
        }
    }

    /// Create a rejection with identified vulnerabilities.
    #[must_use]
    pub fn rejected(vulnerabilities: Vec<Vulnerability>) -> Self {
        Self {
            approved: false,
            vulnerabilities,
            rejection_count_this_session: 0,
            total_reviews_this_session: 0,
            calibration_alert: None,
        }
    }

    /// Evaluate calibration and set `calibration_alert` if the rejection rate
    /// is below `CriticCalibration::rejection_baseline_min`.
    ///
    /// Called by the orchestrator after incrementing session counters. Skipped
    /// when `total_reviews_this_session == 0` or `rejection_baseline_min <= 0.0`.
    /// Returns `true` if calibration passes, `false` if an alert was set.
    pub fn check_calibration(&mut self, calibration: &CriticCalibration) -> bool {
        if self.total_reviews_this_session == 0 || calibration.rejection_baseline_min <= 0.0 {
            return true;
        }
        let rate =
            self.rejection_count_this_session as f32 / self.total_reviews_this_session as f32;
        if rate < calibration.rejection_baseline_min {
            self.calibration_alert = Some(format!(
                "Critic rejection rate {:.1}% is below baseline {:.1}% \
                 ({}/{} reviews rejected). Critic may not be functioning correctly.",
                rate * 100.0,
                calibration.rejection_baseline_min * 100.0,
                self.rejection_count_this_session,
                self.total_reviews_this_session,
            ));
            false
        } else {
            self.calibration_alert = None;
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn critic_review_approval_roundtrip() {
        let review = CriticReview::approved();
        let json = serde_json::to_string(&review).unwrap();
        let back: CriticReview = serde_json::from_str(&json).unwrap();
        assert!(back.approved);
        assert!(back.vulnerabilities.is_empty());
        assert_eq!(back.rejection_count_this_session, 0);
    }

    #[test]
    fn critic_review_rejection_has_finding() {
        let v = Vulnerability::new(
            24,
            VulnerabilityKind::NullOrOutOfBounds,
            "Direct `[0]` index access without `.is_empty()` guard".into(),
        );
        let review = CriticReview::rejected(vec![v]);
        assert!(!review.approved);
        assert_eq!(review.vulnerabilities.len(), 1);
        assert_eq!(review.vulnerabilities[0].line, 24);
    }

    #[test]
    fn finding_status_default_is_pending() {
        let v = Vulnerability::new(1, VulnerabilityKind::ConstraintViolation, "unwrap".into());
        let json = serde_json::to_string(&v).unwrap();
        assert!(
            json.contains("\"pending\""),
            "default status should serialize as pending"
        );
    }

    #[test]
    fn finding_status_waived_carries_rationale() {
        let mut v = Vulnerability::new(5, VulnerabilityKind::ResourceLeak, "socket".into());
        v.status = FindingStatus::Waived {
            rationale: "test-only code path".into(),
        };
        let json = serde_json::to_string(&v).unwrap();
        let back: Vulnerability = serde_json::from_str(&json).unwrap();
        matches!(&back.status, FindingStatus::Waived { rationale } if rationale == "test-only code path");
    }

    #[test]
    fn calibration_stores_three_exemplars() {
        let cal = CriticCalibration::new(
            vec![
                CriticExemplar::new("safe code, no issues".into(), true),
                CriticExemplar::new("direct vec[0] without guard".into(), false),
                CriticExemplar::new("unclosed socket on error path".into(), false),
            ],
            0.10,
        );
        assert_eq!(cal.exemplars.len(), 3);
        assert!((cal.rejection_baseline_min - 0.10).abs() < f32::EPSILON);
    }
}
