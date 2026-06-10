//! Span emission — contract for where completed spans are delivered.

use crate::span::TraceSpan;

// ---------------------------------------------------------------------------
// SpanEmit
// ---------------------------------------------------------------------------

/// Sink contract for delivering a completed [`TraceSpan`] to a backend.
///
/// Each implementor decides where the span goes:
///
/// | Implementor | Backend |
/// |-------------|---------|
/// | File emitter (consumer-provided) | atomic JSON file per actor/date |
/// | HTTP emitter (consumer-provided) | `POST /ingest/span` on an HTTP server |
/// | [`NullSpanEmitter`] | `/dev/null` — useful for tests and feature-off builds |
///
/// # Object safety
///
/// `SpanEmit` is object-safe: `Box<dyn SpanEmit<Error = …>>` works, but
/// callers typically erase the error with a type alias like
/// `type DynSpanEmitter = Box<dyn SpanEmit<Error = Box<dyn std::error::Error>>>`.
///
/// # Example
///
/// ```rust
/// use larc_ayinspan::emit::{SpanEmit, NullSpanEmitter};
/// use larc_ayinspan::{TraceContext, Actor, TraceOutcome};
///
/// let emitter = NullSpanEmitter;
/// let span = TraceContext::new(Actor::corso(), "guard")
///     .outcome(TraceOutcome::Continue)
///     .finish()
///     .unwrap();
/// emitter.emit(span).unwrap();
/// ```
pub trait SpanEmit {
    /// Emission error type.
    type Error: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static;

    /// Deliver `span` to the backend.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` on backend failure (I/O error, HTTP error, etc.).
    /// Implementations must not block the calling thread for more than the
    /// time needed to enqueue the span.
    fn emit(&self, span: TraceSpan) -> Result<(), Self::Error>;
}

// ---------------------------------------------------------------------------
// NullSpanEmitter
// ---------------------------------------------------------------------------

/// No-op [`SpanEmit`] implementation — drops every span silently.
///
/// Used when the `observe` feature is disabled, in unit tests, and as the
/// default emitter for contexts where no backend is configured.
#[derive(Debug, Clone, Copy, Default)]
pub struct NullSpanEmitter;

/// Infallible error type for [`NullSpanEmitter`].
#[derive(Debug)]
pub struct NullEmitError;

impl std::fmt::Display for NullEmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("null emit error (unreachable)")
    }
}

impl std::error::Error for NullEmitError {}

impl SpanEmit for NullSpanEmitter {
    type Error = NullEmitError;

    #[inline]
    fn emit(&self, _span: TraceSpan) -> Result<(), Self::Error> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Actor, TraceContext, TraceOutcome};

    #[test]
    fn null_emitter_accepts_any_span() {
        let emitter = NullSpanEmitter;
        let span = TraceContext::new(Actor::eva(), "speak")
            .outcome(TraceOutcome::Continue)
            .finish()
            .unwrap();
        assert!(emitter.emit(span).is_ok());
    }

    #[test]
    fn null_emitter_is_copy() {
        let a = NullSpanEmitter;
        let _b = a; // Copy
        let _ = a; // original still usable
    }
}
