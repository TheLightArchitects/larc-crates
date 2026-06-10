//! Trace outcome enumeration.

use serde::{Deserialize, Serialize};

/// The outcome of a traced operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "detail")]
#[non_exhaustive]
pub enum TraceOutcome {
    /// The operation completed and processing should continue.
    Continue,
    /// The operation was blocked (e.g., security gate).
    Block,
    /// The operation was intentionally skipped.
    Skip,
    /// The operation failed with an error message.
    ///
    /// Messages longer than 4096 characters are truncated on construction
    /// via [`error`](TraceOutcome::error). Direct variant construction
    /// bypasses this cap — prefer `TraceOutcome::error(msg)`.
    Error(String),
}

impl TraceOutcome {
    /// Maximum length for error messages.
    pub const MAX_ERROR_LEN: usize = 4096;

    /// Create an error outcome, truncating messages longer than 4096 characters.
    #[must_use]
    pub fn error(msg: impl Into<String>) -> Self {
        let mut s = msg.into();
        s.truncate(Self::MAX_ERROR_LEN);
        Self::Error(s)
    }
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

    #[test]
    fn error_constructor_truncates_long_messages() {
        let long_msg = "x".repeat(5000);
        let outcome = TraceOutcome::error(long_msg);
        assert_eq!(
            outcome.as_error().map(|s| s.len()),
            Some(TraceOutcome::MAX_ERROR_LEN)
        );
    }

    #[test]
    fn error_constructor_preserves_short_messages() {
        let outcome = TraceOutcome::error("short");
        assert_eq!(outcome.as_error(), Some("short"));
    }
}

#[cfg(test)]
impl TraceOutcome {
    /// Extract the error message, if this is an `Error` variant.
    fn as_error(&self) -> Option<&str> {
        match self {
            Self::Error(msg) => Some(msg),
            _ => None,
        }
    }
}
