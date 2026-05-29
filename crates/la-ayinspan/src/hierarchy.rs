//! Span hierarchy trait for turn-aware tracing.

use uuid::Uuid;

/// Trait for querying span hierarchy context.
///
/// Consumers implement this trait to provide turn-tracking state
/// without pulling in the state machine itself. The canonical
/// implementation lives in the gateway's `SpanContext` task-local.
pub trait SpanHierarchy {
    /// The session ID for cross-actor correlation.
    fn session_id(&self) -> Option<&str>;

    /// The turn index within the session (0-based).
    fn turn_index(&self) -> Option<u32>;

    /// The parent span ID for nested traces.
    fn parent_id(&self) -> Option<Uuid>;

    /// Whether this span is the root of a turn.
    fn is_turn_root(&self) -> bool;
}
