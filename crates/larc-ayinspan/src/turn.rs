//! Turn tracking — lifecycle contract for session→turn→span hierarchy.

use uuid::Uuid;

use crate::actor::Actor;
use crate::error::TraceError;
use crate::outcome::TraceOutcome;
use crate::span::{TraceContext, TraceSpan};

// ---------------------------------------------------------------------------
// TurnContext
// ---------------------------------------------------------------------------

/// Snapshot of the current turn's identity within a session.
///
/// Carried by [`TurnTracking::current_context`] and threaded into child
/// spans as `parent_id` + `turn_index` so every tool call in a turn shares
/// the same ancestry.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TurnContext {
    /// Session this turn belongs to.
    pub session_id: String,
    /// 0-based monotonic counter — increments once per [`TurnTracking::start_turn`].
    pub turn_index: u32,
    /// UUID of the root span minted by [`TurnTracking::start_turn`].
    ///
    /// All child spans in this turn set `parent_id` to this value.
    pub turn_root_id: Uuid,
}

impl TurnContext {
    /// Create a new turn context.
    #[must_use]
    pub fn new(session_id: String, turn_index: u32, turn_root_id: Uuid) -> Self {
        Self {
            session_id,
            turn_index,
            turn_root_id,
        }
    }
}

// ---------------------------------------------------------------------------
// TurnTracking trait
// ---------------------------------------------------------------------------

/// Contract for managing the session → turn → span hierarchy.
///
/// Each call to [`start_turn`] mints a root span, bumps the turn counter,
/// and makes a [`TurnContext`] available for child spans.  [`finish_turn`]
/// clears that context so the next [`start_turn`] starts fresh.
///
/// # Implementors
///
/// - [`TurnTracker`] — pure in-process state machine (zero I/O, zero async).
/// - Async wrappers may use task-local storage (e.g., `tokio::task_local!`)
///   to thread the current [`TurnContext`] through awaited tasks.
///
/// # Example
///
/// ```rust
/// use larc_ayinspan::{Actor, TraceOutcome};
/// use larc_ayinspan::turn::{TurnTracking, TurnTracker};
///
/// let mut tracker = TurnTracker::new("sess-abc");
///
/// // Turn 0: user sends a message.
/// let root = tracker
///     .start_turn(Actor::new("user"), "user.message")
///     .expect("outcome always set");
///
/// let ctx = tracker.current_context().unwrap();
/// assert_eq!(ctx.turn_index, 0);
/// assert_eq!(ctx.turn_root_id, root.id);
///
/// tracker.finish_turn();
///
/// // Turn 1: next message bumps the index.
/// tracker.start_turn(Actor::new("user"), "user.message").unwrap();
/// assert_eq!(tracker.current_context().unwrap().turn_index, 1);
/// ```
pub trait TurnTracking {
    /// Mint the root span for a new turn and advance the turn counter.
    ///
    /// The returned [`TraceSpan`] has `parent_id: None` and carries
    /// `session_id` + `turn_index` from the tracker's internal state.
    ///
    /// # Errors
    ///
    /// Returns [`TraceError`] only if the internal span builder is mis-used
    /// (in practice this never triggers since `outcome` is always supplied).
    fn start_turn(&mut self, actor: Actor, action: &str) -> Result<TraceSpan, TraceError>;

    /// Current turn context, or `None` if [`finish_turn`] was called and
    /// no new turn has started.
    fn current_context(&self) -> Option<&TurnContext>;

    /// Close the current turn.  The next [`start_turn`] will use the next
    /// turn index.
    fn finish_turn(&mut self);

    /// Session ID this tracker is bound to.
    fn session_id(&self) -> &str;
}

// ---------------------------------------------------------------------------
// TurnTracker — pure state machine, zero I/O
// ---------------------------------------------------------------------------

/// In-process turn state machine implementing [`TurnTracking`].
///
/// Tracks which session we're in, what turn we're on, and the UUID of the
/// current turn-root span.  No file I/O, no async runtime — safe to use in
/// any context.
///
/// Async wrappers typically wrap this in a `tokio::task_local!` scope to
/// propagate context automatically across async boundaries.
#[derive(Debug, Clone)]
pub struct TurnTracker {
    session_id: String,
    /// Next turn index to assign (incremented by [`start_turn`]).
    next_index: u32,
    /// Active turn context; `None` between turns.
    context: Option<TurnContext>,
}

impl TurnTracker {
    /// Create a new tracker bound to `session_id`.
    #[must_use]
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            next_index: 0,
            context: None,
        }
    }

    /// Build a child [`TraceContext`] pre-wired with the current turn's
    /// `session_id`, `turn_index`, and `parent_id`.
    ///
    /// Returns `None` when called outside an active turn (between
    /// [`finish_turn`] and the next [`start_turn`]).
    #[must_use]
    pub fn child_context(&self, actor: Actor, action: &str) -> Option<TraceContext> {
        let ctx = self.context.as_ref()?;
        Some(
            TraceContext::new(actor, action)
                .session_id(&ctx.session_id)
                .turn_index(ctx.turn_index)
                .parent(ctx.turn_root_id),
        )
    }
}

impl TurnTracking for TurnTracker {
    fn start_turn(&mut self, actor: Actor, action: &str) -> Result<TraceSpan, TraceError> {
        let turn_index = self.next_index;
        let span = TraceContext::new(actor, action)
            .session_id(&self.session_id)
            .turn_index(turn_index)
            .outcome(TraceOutcome::Continue)
            .finish()?;

        self.context = Some(TurnContext {
            session_id: self.session_id.clone(),
            turn_index,
            turn_root_id: span.id,
        });
        self.next_index = turn_index.saturating_add(1);
        Ok(span)
    }

    fn current_context(&self) -> Option<&TurnContext> {
        self.context.as_ref()
    }

    fn finish_turn(&mut self) {
        self.context = None;
    }

    fn session_id(&self) -> &str {
        &self.session_id
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turn_index_increments() {
        let mut t = TurnTracker::new("sess-1");

        let r0 = t.start_turn(Actor::new("user"), "user.message").unwrap();
        assert_eq!(t.current_context().unwrap().turn_index, 0);
        assert_eq!(r0.turn_index, Some(0));
        t.finish_turn();

        let r1 = t.start_turn(Actor::new("user"), "user.message").unwrap();
        assert_eq!(t.current_context().unwrap().turn_index, 1);
        assert_eq!(r1.turn_index, Some(1));
        t.finish_turn();

        assert!(t.current_context().is_none());
    }

    #[test]
    fn turn_root_id_matches_span_id() {
        let mut t = TurnTracker::new("sess-2");
        let span = t.start_turn(Actor::claude(), "assistant.response").unwrap();
        assert_eq!(t.current_context().unwrap().turn_root_id, span.id);
    }

    #[test]
    fn child_context_wires_parent_and_turn() {
        let mut t = TurnTracker::new("sess-3");
        let root = t.start_turn(Actor::new("user"), "user.message").unwrap();

        let child_ctx = t
            .child_context(Actor::new("copilot"), "tool.Bash")
            .expect("inside active turn");

        let child = child_ctx.outcome(TraceOutcome::Continue).finish().unwrap();

        assert_eq!(child.parent_id, Some(root.id));
        assert_eq!(child.turn_index, Some(0));
        assert_eq!(child.session_id.as_deref(), Some("sess-3"));
    }

    #[test]
    fn child_context_none_outside_turn() {
        let t = TurnTracker::new("sess-4");
        assert!(t
            .child_context(Actor::claude(), "assistant.response")
            .is_none());
    }

    #[test]
    fn saturating_add_prevents_overflow() {
        let mut t = TurnTracker::new("sess-5");
        t.next_index = u32::MAX;
        t.start_turn(Actor::new("user"), "user.message").unwrap();
        // saturating_add(1) on MAX stays at MAX — no panic.
        assert_eq!(t.next_index, u32::MAX);
    }
}
