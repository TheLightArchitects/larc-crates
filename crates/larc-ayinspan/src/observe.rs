//! Span observation — pre/post hooks for tool-call interception.

use crate::actor::Actor;
use crate::span::TraceSpan;
use crate::turn::TurnContext;

// ---------------------------------------------------------------------------
// SpanObserve
// ---------------------------------------------------------------------------

/// Hook contract for intercepting tool actions before and after they run.
///
/// Implementors wrap a transport or middleware layer and fire two callbacks
/// around each action — [`on_action_start`] before execution and
/// [`on_action_finish`] after the span is fully built.
///
/// # Separation from [`SpanEmit`]
///
/// [`SpanObserve`] observes; [`SpanEmit`] delivers.  An
/// `ObservableTransport<T>` implements `SpanObserve` and internally holds
/// a `Box<dyn SpanEmit>` — calling `emit` inside `on_action_finish`.  Keeping
/// the two contracts separate allows the SDK to swap backends (file vs HTTP)
/// without changing the observation wiring.
///
/// # Example
///
/// ```rust
/// use larc_ayinspan::observe::SpanObserve;
/// use larc_ayinspan::turn::TurnContext;
/// use larc_ayinspan::{Actor, TraceContext, TraceOutcome, TraceSpan};
///
/// struct LogObserver;
///
/// impl SpanObserve for LogObserver {
///     fn on_action_start(
///         &self,
///         actor: &Actor,
///         action: &str,
///         _ctx: Option<&TurnContext>,
///     ) {
///         let _ = (actor, action); // real impl would log
///     }
///
///     fn on_action_finish(&self, span: &TraceSpan) {
///         let _ = span; // real impl would emit
///     }
/// }
/// ```
///
/// [`SpanEmit`]: crate::emit::SpanEmit
pub trait SpanObserve {
    /// Called immediately before an action is dispatched.
    ///
    /// `actor` and `action` identify what is about to run.  `context` is the
    /// current turn context if one is active — callers should pass `None`
    /// when no [`TurnTracking`] state is available.
    ///
    /// [`TurnTracking`]: crate::turn::TurnTracking
    fn on_action_start(&self, actor: &Actor, action: &str, context: Option<&TurnContext>);

    /// Called after the action completes and the span has been built.
    ///
    /// The span is passed by reference; the observer must not take ownership.
    /// [`SpanEmit::emit`] is the correct place to consume the span.
    ///
    /// [`SpanEmit::emit`]: crate::emit::SpanEmit::emit
    fn on_action_finish(&self, span: &TraceSpan);
}

// ---------------------------------------------------------------------------
// NullSpanObserver
// ---------------------------------------------------------------------------

/// No-op [`SpanObserve`] — both hooks are empty.
///
/// Used when the `observe` feature is disabled or no observer is registered.
#[derive(Debug, Clone, Copy, Default)]
pub struct NullSpanObserver;

impl SpanObserve for NullSpanObserver {
    #[inline]
    fn on_action_start(&self, _actor: &Actor, _action: &str, _ctx: Option<&TurnContext>) {}

    #[inline]
    fn on_action_finish(&self, _span: &TraceSpan) {}
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Actor, TraceContext, TraceOutcome};

    #[test]
    fn null_observer_is_copy() {
        let a = NullSpanObserver;
        let _b = a;
        let _ = a;
    }

    #[test]
    fn null_observer_accepts_calls() {
        let obs = NullSpanObserver;
        obs.on_action_start(&Actor::new("copilot"), "tool.Bash", None);

        let span = TraceContext::new(Actor::new("copilot"), "tool.Bash")
            .outcome(TraceOutcome::Continue)
            .finish()
            .unwrap();
        obs.on_action_finish(&span);
    }
}
