//! Trace outcome enumeration.

use serde::{Deserialize, Serialize};

/// The outcome of a traced operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "detail")]
pub enum TraceOutcome {
    /// The operation completed and processing should continue.
    Continue,
    /// The operation was blocked (e.g., security gate).
    Block,
    /// The operation was intentionally skipped.
    Skip,
    /// The operation failed with an error message.
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_trace_outcome() {
        let cases = vec![
            TraceOutcome::Continue,
            TraceOutcome::Block,
            TraceOutcome::Skip,
            TraceOutcome::Error("something broke".into()),
        ];
        for outcome in cases {
            let json = serde_json::to_string(&outcome).expect("serialize outcome");
            let back: TraceOutcome = serde_json::from_str(&json).expect("deserialize outcome");
            assert_eq!(outcome, back);
        }
    }
}
