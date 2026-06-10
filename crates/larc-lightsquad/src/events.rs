//! SSE event types — wire-format structs for the build event stream.
//!
//! These types match the JSON payloads that the webshell frontend consumes
//! over SSE. They are the **public API surface** for build event consumers.
//!
//! # Envelope strategy
//!
//! The TypeScript frontend sends a `"type"` discriminant inline in each event
//! (e.g. `"type": "conductor_tick"`). Rust structs here do **not** include a
//! `r#type` field. Serde silently ignores unknown fields by default, so the
//! `"type"` field passes through without error. This is intentional — the
//! envelope discriminant belongs to the SSE framing layer, not the payload.
//!
//! Do **not** add `#[serde(deny_unknown_fields)]` to these structs. That would
//! break deserialization of valid TS payloads.

use serde::{Deserialize, Serialize};

// ── ConductorTick ────────────────────────────────────────────────────────────

/// SSE payload: build heartbeat with queue depth and active workers.
///
/// Wire format: `{"type":"conductor_tick","build_id":"...","tick_seq":42,...}`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ConductorTickEvent {
    /// Build identifier (e.g. `"ironclaw-spine"`).
    pub build_id: String,
    /// Monotonically increasing tick sequence number.
    pub tick_seq: u64,
    /// Number of tasks waiting to be dispatched.
    pub queue_depth: u32,
    /// Number of worker slots currently active.
    pub active_workers: u8,
}

impl ConductorTickEvent {
    /// Create a new conductor tick event.
    #[must_use]
    pub fn new(build_id: String, tick_seq: u64, queue_depth: u32, active_workers: u8) -> Self {
        Self {
            build_id,
            tick_seq,
            queue_depth,
            active_workers,
        }
    }
}

// ── MergeAgentStatus ───────────────────────────────────────────────────────────

/// SSE payload: merge agent progress within a wave.
///
/// Wire format: `{"type":"merge_agent_status","build_id":"...","wave_index":0,...}`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MergeAgentStatusEvent {
    /// Build identifier.
    pub build_id: String,
    /// Zero-based wave index.
    pub wave_index: u32,
    /// Current merge phase (e.g. `"started"`, `"merged"`, `"conflict"`).
    pub phase: String,
    /// SHA of the merged commit, if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,
}

impl MergeAgentStatusEvent {
    /// Create a new merge agent status event.
    #[must_use]
    pub fn new(build_id: String, wave_index: u32, phase: String) -> Self {
        Self {
            build_id,
            wave_index,
            phase,
            commit_sha: None,
        }
    }
}

// ── FixAgentIteration ─────────────────────────────────────────────────────────

/// SSE payload: fix-agent iteration during gate remediation.
///
/// Wire format: `{"type":"fix_agent_iteration","build_id":"...","wave_index":0,...}`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FixAgentIterationEvent {
    /// Build identifier.
    pub build_id: String,
    /// Zero-based wave index.
    pub wave_index: u32,
    /// Worker slot number (1-indexed).
    pub worker_slot: u8,
    /// Iteration number (1-indexed, max = gate's `max_iterations`).
    pub iteration: u32,
    /// One-line summary of the issue being fixed.
    pub issue_summary: String,
    /// Structured evidence bundle for this correction iteration.
    ///
    /// `None` for legacy callers or when evidence assembly is not wired.
    /// `Some` carries the full `EvidenceBundle` — compiler failures,
    /// Reviewer vulnerabilities, AYIN span ID, and loop index.
    /// Frontend consumers use this to display structured correction context
    /// rather than relying on the plain-text `issue_summary` (Gap D closure).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<crate::evidence::EvidenceBundle>,
}

impl FixAgentIterationEvent {
    /// Create a new fix agent iteration event.
    #[must_use]
    pub fn new(
        build_id: String,
        wave_index: u32,
        worker_slot: u8,
        iteration: u32,
        issue_summary: String,
    ) -> Self {
        Self {
            build_id,
            wave_index,
            worker_slot,
            iteration,
            issue_summary,
            evidence: None,
        }
    }
}

// ── Escalation ─────────────────────────────────────────────────────────────────

/// SSE payload: build escalation — a gate or task failed and requires operator action.
///
/// Wire format:
/// ```json
/// {"type":"escalation","build_id":"...","wave_index":0,"worker_slot":4,
///  "call_id":"550e8400-...","reason":"Gate [S] threshold exceeded",
///  "canon_ref":"canon://security-guardrails"}
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EscalationEvent {
    /// Build identifier.
    pub build_id: String,
    /// Zero-based wave index where the escalation occurred.
    pub wave_index: u32,
    /// Worker slot that triggered the escalation (absent for gate-level escalations).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worker_slot: Option<u8>,
    /// Unique call identifier for the escalation.
    pub call_id: String,
    /// Human-readable reason for the escalation.
    pub reason: String,
    /// Canonical reference (e.g. `"canon://security-guardrails §48"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canon_ref: Option<String>,
}

impl EscalationEvent {
    /// Create a new escalation event.
    #[must_use]
    pub fn new(build_id: String, wave_index: u32, call_id: String, reason: String) -> Self {
        Self {
            build_id,
            wave_index,
            worker_slot: None,
            call_id,
            reason,
            canon_ref: None,
        }
    }
}

// ── DecisionEntry (API view DTO) ──────────────────────────────────────────────

/// A single entry in the build's HMAC-chained decision log, as returned by
/// `GET /api/builds/:id/decisions`.
///
/// This is the **public view** of a decision-log entry. Implementations that
/// maintain an HMAC-chained log keep the `prev_hash` and `entry_hash` crypto
/// fields private; only [`hmac_ok`] (verification result) is exposed.
///
/// [`hmac_ok`]: DecisionEntryDto::hmac_ok
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DecisionEntryDto {
    /// 0-based line number within the decision log.
    pub line_n: u64,
    /// ISO 8601 timestamp.
    pub timestamp: String,
    /// Decision level (`"L1"`, `"L2"`, `"L3"`, `"L4"`).
    pub level: String,
    /// Human-readable decision text (e.g. `"APPROVED: all Canon checks pass"`).
    pub decision: String,
    /// Canonical reference, if applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canon_ref: Option<String>,
    /// Whether the HMAC chain verified this entry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hmac_ok: Option<bool>,
}

impl DecisionEntryDto {
    /// Create a new decision entry DTO.
    #[must_use]
    pub fn new(line_n: u64, timestamp: String, level: String, decision: String) -> Self {
        Self {
            line_n,
            timestamp,
            level,
            decision,
            canon_ref: None,
            hmac_ok: None,
        }
    }
}

// ── AgentHeartbeat ────────────────────────────────────────────────────────────

/// SSE payload: liveness signal from an agent during long-running operations.
///
/// Typically emitted every 10 seconds per agent. An orchestrator can alert
/// when the gap between heartbeats exceeds a threshold (e.g. 15 seconds),
/// catching silent agent stalls without requiring human intervention.
///
/// Wire format: `{"type":"agent_heartbeat","build_id":"...","agent_id":"...","stage":"..."}`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AgentHeartbeatEvent {
    /// Build identifier.
    pub build_id: String,
    /// Agent identifier within the pool (e.g. `"researcher-0"`, `"tester-fix-1"`).
    pub agent_id: String,
    /// Current pipeline stage (e.g. `"research"`, `"code"`, `"review"`, `"test"`).
    pub stage: String,
    /// Current correction loop iteration, if the agent is in a fix cycle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loop_index: Option<u8>,
}

impl AgentHeartbeatEvent {
    /// Create a new heartbeat event.
    #[must_use]
    pub fn new(build_id: String, agent_id: String, stage: String) -> Self {
        Self {
            build_id,
            agent_id,
            stage,
            loop_index: None,
        }
    }
}

// ── IterationMetrics ──────────────────────────────────────────────────────────

/// SSE payload: per-iteration performance metrics from the Tester agent.
///
/// Emitted at the end of each correction loop iteration. Use to monitor
/// the performance gate — a significant wall-clock regression between
/// iterations may indicate a compile-cache miss or sandbox cold-start.
///
/// Wire format: `{"type":"iteration_metrics","build_id":"...","loop_index":0,...}`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IterationMetricsEvent {
    /// Build identifier.
    pub build_id: String,
    /// Zero-based wave index.
    pub wave_index: u32,
    /// Zero-indexed correction iteration (matches `SanitizedTrace::loop_index`).
    pub loop_index: u8,
    /// Time to compile in milliseconds (wall-clock from `cargo check` start).
    pub compile_ms: u64,
    /// Time to run tests in milliseconds (wall-clock from `cargo test` start).
    pub test_ms: u64,
    /// Whether the sccache layer served this compilation from cache.
    ///
    /// `false` on first iteration is expected; `false` on iterations 1–2
    /// indicates sccache misconfiguration or cache key collision.
    pub cache_hit: bool,
    /// Change in aggregate test pass rate relative to the previous iteration.
    ///
    /// `None` for iteration 0 (no prior baseline). Positive = improvement.
    /// Used by the orchestrator to detect convergence vs stall.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score_delta: Option<f32>,
}

impl IterationMetricsEvent {
    /// Create a new iteration metrics event.
    #[must_use]
    pub fn new(
        build_id: String,
        wave_index: u32,
        loop_index: u8,
        compile_ms: u64,
        test_ms: u64,
        cache_hit: bool,
    ) -> Self {
        Self {
            build_id,
            wave_index,
            loop_index,
            compile_ms,
            test_ms,
            cache_hit,
            score_delta: None,
        }
    }
}
