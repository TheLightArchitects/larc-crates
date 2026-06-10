//! Wire compatibility tests: SSE event types vs TypeScript-shaped JSON.
//!
//! These tests verify that the public event structs in `larc_lightsquad::events`
//! correctly round-trip against the JSON shapes the TypeScript frontend sends
//! over SSE. All six pre-publish migration debt items (D-1 through D-6) are
//! now resolved; these tests confirm the resolutions.

use larc_lightsquad::{
    ConductorTickEvent, DecisionEntryDto, EscalationEvent, FixAgentIterationEvent,
    MergeAgentStatusEvent,
};

// ── ConductorTick ─────────────────────────────────────────────────────────────

#[test]
fn conductor_tick_ts_fixture_round_trips() {
    let json = r#"{"type":"conductor_tick","build_id":"build-001","tick_seq":42,"queue_depth":3,"active_workers":2}"#;
    let event: ConductorTickEvent = serde_json::from_str(json).expect("conductor_tick round-trip");
    assert_eq!(event.build_id, "build-001");
    assert_eq!(event.tick_seq, 42);
    assert_eq!(event.queue_depth, 3);
    assert_eq!(event.active_workers, 2);
}

#[test]
fn conductor_tick_serialize_has_no_type_field() {
    let event = ConductorTickEvent::new("b1".to_owned(), 1, 0, 0);
    let json = serde_json::to_string(&event).unwrap();
    assert!(
        !json.contains(r#""type""#),
        "Rust serialization must not include envelope discriminant"
    );
}

// ── MergeAgentStatus ──────────────────────────────────────────────────────────

#[test]
fn merge_agent_status_with_sha_round_trips() {
    let json = r#"{"type":"merge_agent_status","build_id":"build-001","wave_index":0,"phase":"merged","commit_sha":"abc123def456"}"#;
    let event: MergeAgentStatusEvent =
        serde_json::from_str(json).expect("merge_agent_status with sha");
    assert_eq!(event.phase, "merged");
    assert_eq!(event.commit_sha.as_deref(), Some("abc123def456"));
}

#[test]
fn merge_agent_status_without_sha_round_trips() {
    let json =
        r#"{"type":"merge_agent_status","build_id":"build-001","wave_index":1,"phase":"started"}"#;
    let event: MergeAgentStatusEvent =
        serde_json::from_str(json).expect("merge_agent_status no sha");
    assert_eq!(event.commit_sha, None);
}

// ── FixAgentIteration ─────────────────────────────────────────────────────────

#[test]
fn fix_agent_iteration_ts_fixture_round_trips() {
    let json = r#"{"type":"fix_agent_iteration","build_id":"build-001","wave_index":0,"worker_slot":3,"iteration":2,"issue_summary":"clippy::unwrap_used in src/lib.rs:42"}"#;
    let event: FixAgentIterationEvent =
        serde_json::from_str(json).expect("fix_agent_iteration round-trip");
    assert_eq!(event.worker_slot, 3);
    assert_eq!(event.iteration, 2);
    assert!(event.issue_summary.contains("clippy"));
}

// ── EscalationEvent (D-1 + D-2 resolved) ─────────────────────────────────────

#[test]
fn escalation_with_slot_and_canon_ref_round_trips() {
    let json = r#"{"type":"escalation","build_id":"build-001","wave_index":0,"worker_slot":4,"call_id":"550e8400-e29b-41d4-a716-446655440000","reason":"Gate [S] threshold exceeded","canon_ref":"canon://security-guardrails"}"#;
    let event: EscalationEvent =
        serde_json::from_str(json).expect("escalation with slot + canon_ref");
    assert_eq!(event.worker_slot, Some(4));
    assert_eq!(event.call_id, "550e8400-e29b-41d4-a716-446655440000");
    assert_eq!(
        event.canon_ref.as_deref(),
        Some("canon://security-guardrails")
    );
}

#[test]
fn escalation_without_slot_now_accepts_optional() {
    // D-1 RESOLVED: worker_slot is now Option<u8> with #[serde(default)].
    // TS sends `worker_slot?: number` — absent field deserializes to None.
    let json = r#"{"type":"escalation","build_id":"build-001","wave_index":0,"call_id":"550e8400-e29b-41d4-a716-446655440000","reason":"Gate [S] exceeded"}"#;
    let event: EscalationEvent =
        serde_json::from_str(json).expect("escalation without worker_slot now deserializes");
    assert_eq!(event.worker_slot, None);
    assert_eq!(event.reason, "Gate [S] exceeded");
}

#[test]
fn escalation_without_canon_ref_deserializes() {
    let json = r#"{"type":"escalation","build_id":"build-001","wave_index":0,"worker_slot":2,"call_id":"call-42","reason":"HITL approval required"}"#;
    let event: EscalationEvent = serde_json::from_str(json).unwrap();
    assert_eq!(event.canon_ref, None);
}

#[test]
fn escalation_serializes_with_skip_none() {
    let event = EscalationEvent::new("b1".to_owned(), 0, "call-42".to_owned(), "test".to_owned());
    let json = serde_json::to_string(&event).unwrap();
    assert!(
        !json.contains("worker_slot"),
        "None worker_slot should be skipped"
    );
    assert!(
        !json.contains("canon_ref"),
        "None canon_ref should be skipped"
    );
}

// ── DecisionEntryDto (D-3 resolved) ──────────────────────────────────────────

#[test]
fn decision_entry_dto_full_ts_fixture_round_trips() {
    let json = r#"{"line_n":0,"timestamp":"2026-05-29T10:00:00Z","level":"L2","decision":"APPROVED: all Canon checks pass","canon_ref":"canon://builders-cookbook §48","hmac_ok":true}"#;
    let entry: DecisionEntryDto =
        serde_json::from_str(json).expect("decision_entry_dto full round-trip");
    assert_eq!(entry.line_n, 0);
    assert_eq!(entry.level, "L2");
    assert_eq!(entry.decision, "APPROVED: all Canon checks pass");
    assert_eq!(
        entry.canon_ref.as_deref(),
        Some("canon://builders-cookbook §48")
    );
    assert_eq!(entry.hmac_ok, Some(true));
}

#[test]
fn decision_entry_dto_minimal_ts_fixture_round_trips() {
    let json = r#"{"line_n":1,"timestamp":"2026-05-29T10:01:00Z","level":"L1","decision":"BLOCKED: missing citation"}"#;
    let entry: DecisionEntryDto =
        serde_json::from_str(json).expect("decision_entry_dto minimal round-trip");
    assert_eq!(entry.line_n, 1);
    assert_eq!(entry.level, "L1");
    assert_eq!(entry.canon_ref, None);
    assert_eq!(entry.hmac_ok, None);
}

// ── ContextTier conversion (D-4 resolved) ─────────────────────────────────────

#[test]
fn context_tier_string_to_u8_round_trips() {
    use larc_lightsquad::ContextTier;

    assert_eq!(ContextTier::tier_from_string("T1"), Some(0));
    assert_eq!(ContextTier::tier_from_string("T2"), Some(1));
    assert_eq!(ContextTier::tier_from_string("T3"), Some(2));
    assert_eq!(ContextTier::tier_from_string("T0"), None);
    assert_eq!(ContextTier::tier_from_string("unknown"), None);

    assert_eq!(ContextTier::tier_to_string(0), Some("T1"));
    assert_eq!(ContextTier::tier_to_string(1), Some("T2"));
    assert_eq!(ContextTier::tier_to_string(2), Some("T3"));
    assert_eq!(ContextTier::tier_to_string(3), None);
}

// ── GateDimension ─────────────────────────────────────────────────────────────

#[test]
fn gate_dimension_custom_variant_serializes() {
    use larc_lightsquad::GateDimension;

    let custom = GateDimension::Custom("bespoke".to_owned());
    let json = serde_json::to_string(&custom).unwrap();
    assert_eq!(json, r#"{"custom":"bespoke"}"#);
    let back: GateDimension = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, GateDimension::Custom(s) if s == "bespoke"));
}
