//! Trace span and builder.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::actor::Actor;
use crate::decision::DecisionPoint;
use crate::error::TraceError;
use crate::outcome::TraceOutcome;
use crate::strand::StrandActivation;

/// A complete trace record for a single unit of work.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TraceSpan {
    /// Unique identifier for this span.
    pub id: Uuid,
    /// Parent span ID for nested traces.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<Uuid>,
    /// Session ID for cross-actor correlation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// 0-based turn index within the session.
    ///
    /// Each user→assistant exchange increments this counter. All spans
    /// belonging to the same turn share the same `turn_index`. Absent on
    /// legacy spans written before this field was introduced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_index: Option<u32>,
    /// Which actor produced this trace.
    ///
    /// Serializes as `"actor"` in new spans. Accepts `"sibling"` on
    /// deserialization for backward compatibility with existing JSON files.
    #[serde(alias = "sibling")]
    pub actor: Actor,
    /// The action being traced (e.g. "guard", "speak", "helix_query").
    pub action: String,
    /// When the operation started.
    pub timestamp: DateTime<Utc>,
    /// Total wall-clock duration in milliseconds.
    pub duration_ms: u64,
    /// Decisions made during the operation.
    #[serde(default)]
    pub decision_points: Vec<DecisionPoint>,
    /// Strand activations during the operation.
    #[serde(default)]
    pub strand_activations: Vec<StrandActivation>,
    /// Final outcome.
    pub outcome: TraceOutcome,
    /// Arbitrary metadata (tool params, intermediate results, etc.).
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl TraceSpan {
    /// Backward-compatible accessor: returns the actor (formerly `sibling`).
    #[must_use]
    pub fn sibling(&self) -> &Actor {
        &self.actor
    }
}

/// Builder for constructing a [`TraceSpan`] incrementally.
///
/// # Example
///
/// ```rust
/// use larc_ayinspan::{TraceContext, Actor, TraceOutcome};
///
/// let span = TraceContext::new(Actor::corso(), "guard")
///     .session_id("sess-123")
///     .outcome(TraceOutcome::Continue)
///     .finish();
/// assert!(span.is_ok());
/// ```
pub struct TraceContext {
    id: Uuid,
    parent_id: Option<Uuid>,
    session_id: Option<String>,
    turn_index: Option<u32>,
    actor: Actor,
    action: String,
    start: DateTime<Utc>,
    decision_points: Vec<DecisionPoint>,
    strand_activations: Vec<StrandActivation>,
    outcome: Option<TraceOutcome>,
    metadata: serde_json::Value,
}

impl TraceContext {
    /// Start building a new trace span.
    #[must_use]
    pub fn new(actor: Actor, action: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            parent_id: None,
            session_id: None,
            turn_index: None,
            actor,
            action: action.to_owned(),
            start: Utc::now(),
            decision_points: Vec::new(),
            strand_activations: Vec::new(),
            outcome: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Override the auto-generated span UUID.
    ///
    /// Useful when the caller pre-allocates an ID to set up parent references
    /// before the span is finished (e.g. pipeline orchestration).
    #[must_use]
    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    /// Return the UUID this span will be assigned when [`finish`](Self::finish) is called.
    #[must_use]
    pub fn span_id(&self) -> Uuid {
        self.id
    }

    /// Set a parent span for nesting.
    #[must_use]
    pub fn parent(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Set the session ID for cross-actor correlation.
    #[must_use]
    pub fn session_id(mut self, id: &str) -> Self {
        self.session_id = Some(id.to_owned());
        self
    }

    /// Set the 0-based turn index within the session.
    #[must_use]
    pub fn turn_index(mut self, index: u32) -> Self {
        self.turn_index = Some(index);
        self
    }

    /// Record a decision point.
    ///
    /// # Errors
    ///
    /// Returns [`TraceError::ConfidenceOutOfRange`] if `confidence` is
    /// outside `[0.0, 1.0]`.
    pub fn decision(
        mut self,
        name: &str,
        input: &str,
        decision: &str,
        confidence: Option<f64>,
        duration_ms: u64,
    ) -> Result<Self, TraceError> {
        if let Some(c) = confidence
            && !(0.0..=1.0).contains(&c)
        {
            return Err(TraceError::ConfidenceOutOfRange { value: c });
        }
        self.decision_points.push(DecisionPoint {
            name: name.to_owned(),
            input: input.to_owned(),
            decision: decision.to_owned(),
            confidence,
            duration_ms,
        });
        Ok(self)
    }

    /// Record a strand activation.
    ///
    /// # Errors
    ///
    /// Returns [`TraceError::WeightOutOfRange`] if `weight` is outside
    /// `[0.0, 1.0]`.
    pub fn strand(mut self, strand: &str, weight: f64) -> Result<Self, TraceError> {
        if !(0.0..=1.0).contains(&weight) {
            return Err(TraceError::WeightOutOfRange { value: weight });
        }
        self.strand_activations.push(StrandActivation {
            strand: strand.to_owned(),
            weight,
        });
        Ok(self)
    }

    /// Set the outcome.
    #[must_use]
    pub fn outcome(mut self, outcome: TraceOutcome) -> Self {
        self.outcome = Some(outcome);
        self
    }

    /// Attach arbitrary metadata.
    #[must_use]
    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Attach LASDLC semantic-convention metadata.
    ///
    /// Filters keys to the `lasdlc.*` namespace per
    /// [`crate::semconv::lasdlc::is_lasdlc_key`]. Non-conforming keys
    /// are dropped.
    #[must_use]
    pub fn lasdlc_metadata(mut self, attrs: serde_json::Value) -> Self {
        let filtered = if let serde_json::Value::Object(map) = attrs {
            let valid: serde_json::Map<String, serde_json::Value> = map
                .into_iter()
                .filter(|(k, _)| crate::semconv::lasdlc::is_lasdlc_key(k))
                .collect();
            serde_json::Value::Object(valid)
        } else {
            serde_json::Value::Null
        };
        self.metadata = filtered;
        self
    }

    /// Consume the builder and produce a [`TraceSpan`].
    ///
    /// # Errors
    ///
    /// Returns [`TraceError::MissingField`] if `outcome` was not set.
    pub fn finish(self) -> Result<TraceSpan, TraceError> {
        let outcome = self.outcome.ok_or_else(|| TraceError::MissingField {
            field: "outcome".to_owned(),
        })?;

        let now = Utc::now();
        let delta = now.signed_duration_since(self.start);
        // Clock regression produces 0 rather than a bogus positive value.
        let duration_ms = if delta.num_milliseconds() < 0 {
            0
        } else {
            delta.num_milliseconds() as u64
        };

        Ok(TraceSpan {
            id: self.id,
            parent_id: self.parent_id,
            session_id: self.session_id,
            turn_index: self.turn_index,
            actor: self.actor,
            action: self.action,
            timestamp: self.start,
            duration_ms,
            decision_points: self.decision_points,
            strand_activations: self.strand_activations,
            outcome,
            metadata: self.metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_full_span() {
        let span = TraceSpan {
            id: Uuid::new_v4(),
            parent_id: Some(Uuid::new_v4()),
            session_id: Some("sess-abc".into()),
            turn_index: Some(2),
            actor: Actor::eva(),
            action: "speak".into(),
            timestamp: Utc::now(),
            duration_ms: 42,
            decision_points: vec![DecisionPoint {
                name: "voice_select".into(),
                input: "converse".into(),
                decision: "use eva voice".into(),
                confidence: Some(0.99),
                duration_ms: 3,
            }],
            strand_activations: vec![StrandActivation {
                strand: "empathy".into(),
                weight: 0.9,
            }],
            outcome: TraceOutcome::Continue,
            metadata: serde_json::json!({"tool": "speak"}),
        };

        let json = serde_json::to_string_pretty(&span).expect("serialize");
        let back: TraceSpan = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(span.id, back.id);
        assert_eq!(span.parent_id, back.parent_id);
        assert_eq!(span.session_id, back.session_id);
        assert_eq!(span.actor, back.actor);
        assert_eq!(span.action, back.action);
        assert_eq!(span.duration_ms, back.duration_ms);
        assert_eq!(span.outcome, back.outcome);
    }

    #[test]
    fn backward_compat_sibling_json() {
        let old_json = r#"{
            "id": "00000000-0000-0000-0000-000000000001",
            "parent_id": null,
            "session_id": null,
            "sibling": "eva",
            "action": "speak",
            "timestamp": "2026-01-01T00:00:00Z",
            "duration_ms": 10,
            "decision_points": [],
            "strand_activations": [],
            "outcome": {"type": "Continue"},
            "metadata": null
        }"#;
        let span: TraceSpan = serde_json::from_str(old_json).expect("deserialize old format");
        assert_eq!(span.actor, Actor::eva());
    }

    #[test]
    fn context_builder_happy_path() {
        let span = TraceContext::new(Actor::corso(), "guard")
            .session_id("sess-1")
            .outcome(TraceOutcome::Block)
            .metadata(serde_json::json!({"severity": "critical"}))
            .finish();

        assert!(span.is_ok());
        let span = span.expect("just checked");
        assert_eq!(span.actor, Actor::corso());
        assert_eq!(span.action, "guard");
        assert_eq!(span.outcome, TraceOutcome::Block);
        assert!(span.session_id.is_some());
    }

    #[test]
    fn context_builder_missing_outcome() {
        let result = TraceContext::new(Actor::soul(), "query").finish();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("outcome"));
    }

    #[test]
    fn context_builder_with_decisions_and_strands() {
        let span = TraceContext::new(Actor::quantum(), "probe")
            .decision("source_select", "multi-source", "perplexity", Some(0.7), 5)
            .expect("valid confidence")
            .strand("methodical", 0.85)
            .expect("valid weight")
            .outcome(TraceOutcome::Continue)
            .finish()
            .expect("valid span");

        assert_eq!(span.decision_points.len(), 1);
        assert_eq!(span.strand_activations.len(), 1);
    }

    #[test]
    fn confidence_out_of_range() {
        let result =
            TraceContext::new(Actor::eva(), "speak").decision("test", "in", "out", Some(1.5), 1);
        assert!(result.is_err());
    }

    #[test]
    fn weight_out_of_range() {
        let result = TraceContext::new(Actor::eva(), "speak").strand("test", -0.1);
        assert!(result.is_err());
    }

    #[test]
    fn turn_index_roundtrip() {
        let span = TraceContext::new(Actor::claude(), "user.message")
            .session_id("sess-xyz")
            .turn_index(3)
            .outcome(TraceOutcome::Continue)
            .finish()
            .expect("valid span");

        assert_eq!(span.turn_index, Some(3));

        // Serialize and verify field present in JSON.
        let json = serde_json::to_string(&span).expect("serialize");
        assert!(json.contains("\"turn_index\":3"));

        let back: TraceSpan = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.turn_index, Some(3));
    }

    #[test]
    fn turn_index_absent_on_legacy_spans() {
        // Old JSON without turn_index must deserialize with None.
        let old_json = r#"{
            "id": "00000000-0000-0000-0000-000000000002",
            "parent_id": null,
            "session_id": "sess-old",
            "actor": "claude",
            "action": "user.message",
            "timestamp": "2026-01-01T00:00:00Z",
            "duration_ms": 0,
            "decision_points": [],
            "strand_activations": [],
            "outcome": {"type": "Continue"},
            "metadata": null
        }"#;
        let span: TraceSpan = serde_json::from_str(old_json).expect("deserialize legacy");
        assert_eq!(span.turn_index, None);
    }

    #[test]
    fn span_hierarchy_impl() {
        use crate::hierarchy::SpanHierarchy;
        use uuid::Uuid;

        let parent_id = Uuid::new_v4();
        let span = TraceContext::new(Actor::new("copilot"), "tool.Bash")
            .session_id("sess-h")
            .turn_index(1)
            .parent(parent_id)
            .outcome(TraceOutcome::Continue)
            .finish()
            .expect("valid span");

        assert_eq!(span.session_id(), Some("sess-h"));
        assert_eq!(span.turn_index(), Some(1));
        assert_eq!(span.parent_id(), Some(parent_id));
        assert!(!span.is_turn_root());

        // A root span (no parent) reports is_turn_root = true.
        let root = TraceContext::new(Actor::new("user"), "user.message")
            .session_id("sess-h")
            .turn_index(1)
            .outcome(TraceOutcome::Continue)
            .finish()
            .expect("valid root");
        assert!(root.is_turn_root());
    }

    #[test]
    fn lasdlc_metadata_filters_keys() {
        let span = TraceContext::new(Actor::corso(), "guard")
            .lasdlc_metadata(serde_json::json!({
                "lasdlc.hook.name": "PreToolUse",
                "lasdlc.blocked": false,
                "non_lasdlc_key": "dropped"
            }))
            .outcome(TraceOutcome::Continue)
            .finish()
            .expect("valid span");

        let obj = span.metadata.as_object().expect("should be object");
        assert!(obj.contains_key("lasdlc.hook.name"));
        assert!(obj.contains_key("lasdlc.blocked"));
        assert!(!obj.contains_key("non_lasdlc_key"));
    }
}
