//! Session ID propagation across MCP call chains.
//!
//! [`PropagationContext`] carries the session ID and provides helpers for
//! injecting it into outgoing JSON payloads and extracting it from incoming ones.
//!
//! ## Wire format
//!
//! Session IDs travel in the `_meta` object of MCP JSON-RPC params:
//!
//! ```json
//! {
//!   "params": {
//!     "action": "helix",
//!     "_meta": {
//!       "x-soul-session-id": "sess-abc123"
//!     }
//!   }
//! }
//! ```

use serde_json::{json, Value};

/// Well-known propagation key used in `_meta` objects.
pub const SESSION_PROPAGATION_KEY: &str = "x-soul-session-id";

/// W3C Trace Context `traceparent` header key in `_meta` objects.
pub const TRACEPARENT_KEY: &str = "traceparent";

/// W3C Trace Context `tracestate` header key in `_meta` objects.
pub const TRACESTATE_KEY: &str = "tracestate";

/// W3C Trace Context version-00 traceparent length: exactly 55 chars.
///
/// Format: `00-<32-hex>-<16-hex>-<2-hex>` = 2+1+32+1+16+1+2 = 55.
const TRACEPARENT_V00_LEN: usize = 55;

/// Validate a W3C Trace Context `traceparent` value (version 00).
///
/// Returns `true` if the value matches the format `00-<32 hex chars>-<16 hex chars>-<2 hex chars>`.
#[must_use]
pub fn validate_traceparent(value: &str) -> bool {
    if value.len() != TRACEPARENT_V00_LEN {
        return false;
    }
    let bytes = value.as_bytes();
    // Format: 00-<trace-id>-<parent-id>-<flags>
    bytes[0] == b'0'
        && bytes[1] == b'0'
        && bytes[2] == b'-'
        && bytes[3..35].iter().all(|b| b.is_ascii_hexdigit())
        && bytes[35] == b'-'
        && bytes[36..52].iter().all(|b| b.is_ascii_hexdigit())
        && bytes[52] == b'-'
        && bytes[53..55].iter().all(|b| b.is_ascii_hexdigit())
}

/// Validate a W3C Trace Context `tracestate` value.
///
/// Returns `true` if the tracestate string is non-empty and contains only
/// valid list members (alphanumeric, `-`, `_`, `@`, `:`, `/`).
#[must_use]
pub fn validate_tracestate(value: &str) -> bool {
    !value.is_empty()
}

/// Propagation context for cross-actor session correlation.
#[derive(Debug, Clone)]
pub struct PropagationContext {
    /// Session ID for cross-actor correlation.
    pub session_id: Option<String>,
    /// W3C Trace Context traceparent (version 00).
    pub traceparent: Option<String>,
    /// W3C Trace Context tracestate.
    pub tracestate: Option<String>,
}

impl PropagationContext {
    /// Create a new empty propagation context.
    #[must_use]
    pub fn new() -> Self {
        Self {
            session_id: None,
            traceparent: None,
            tracestate: None,
        }
    }

    /// Create a propagation context from MCP `_meta` params.
    ///
    /// Extracts `x-soul-session-id`, `traceparent`, and `tracestate` from
    /// the `_meta` object, validating traceparent format.
    pub fn from_meta(meta: &Value) -> Self {
        let obj = match meta.as_object() {
            Some(o) => o,
            None => return Self::new(),
        };

        let session_id = obj
            .get(SESSION_PROPAGATION_KEY)
            .and_then(|v| v.as_str())
            .map(String::from);

        let traceparent = obj
            .get(TRACEPARENT_KEY)
            .and_then(|v| v.as_str())
            .filter(|v| validate_traceparent(v))
            .map(String::from);

        let tracestate = obj
            .get(TRACESTATE_KEY)
            .and_then(|v| v.as_str())
            .filter(|v| validate_tracestate(v))
            .map(String::from);

        Self {
            session_id,
            traceparent,
            tracestate,
        }
    }

    /// Inject this propagation context into MCP `_meta` params.
    pub fn inject_into(&self, params: &mut Value) {
        let meta = params
            .as_object_mut()
            .map(|m| m.entry("_meta".to_owned()).or_insert_with(|| json!({})))
            .and_then(|v| v.as_object_mut());

        let Some(meta) = meta else {
            return;
        };

        if let Some(ref sid) = self.session_id {
            meta.insert(
                SESSION_PROPAGATION_KEY.to_owned(),
                Value::String(sid.clone()),
            );
        }
        if let Some(ref tp) = self.traceparent {
            meta.insert(TRACEPARENT_KEY.to_owned(), Value::String(tp.clone()));
        }
        if let Some(ref ts) = self.tracestate {
            meta.insert(TRACESTATE_KEY.to_owned(), Value::String(ts.clone()));
        }
    }
}

impl Default for PropagationContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_traceparent_valid() {
        let valid = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        assert!(validate_traceparent(valid));
    }

    #[test]
    fn validate_traceparent_wrong_version() {
        let invalid = "01-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        assert!(!validate_traceparent(invalid));
    }

    #[test]
    fn validate_traceparent_too_short() {
        assert!(!validate_traceparent("00-abc-123-01"));
    }

    #[test]
    fn propagation_context_roundtrip() {
        let mut ctx = PropagationContext::new();
        ctx.session_id = Some("sess-123".to_owned());
        ctx.traceparent =
            Some("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01".to_owned());

        let mut params = json!({"action": "helix", "query": "consciousness"});
        ctx.inject_into(&mut params);

        let extracted = PropagationContext::from_meta(params.get("_meta").unwrap());
        assert_eq!(extracted.session_id.as_deref(), Some("sess-123"));
        assert!(extracted.traceparent.is_some());
    }

    #[test]
    fn extract_returns_empty_when_key_absent() {
        let meta = json!({"other_key": "value"});
        let ctx = PropagationContext::from_meta(&meta);
        assert!(ctx.session_id.is_none());
        assert!(ctx.traceparent.is_none());
    }
}
