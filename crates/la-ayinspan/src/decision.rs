//! Decision points recorded during traced operations.

use serde::{Deserialize, Serialize};

/// A decision made during a traced operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPoint {
    /// Human-readable name of the decision (e.g. "route_to_hero").
    pub name: String,
    /// Summary of the input that informed the decision.
    pub input: String,
    /// The decision that was made.
    pub decision: String,
    /// Confidence in the decision, in the range `[0.0, 1.0]`.
    pub confidence: Option<f64>,
    /// Time spent making this decision, in milliseconds.
    pub duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_decision_point() {
        let dp = DecisionPoint {
            name: "route".into(),
            input: "guard request".into(),
            decision: "delegate".into(),
            confidence: Some(0.95),
            duration_ms: 12,
        };
        let json = serde_json::to_string(&dp).expect("serialize");
        let back: DecisionPoint = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(dp.name, back.name);
        assert_eq!(dp.confidence, back.confidence);
    }
}
