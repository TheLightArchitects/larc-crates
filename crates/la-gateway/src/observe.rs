//! Observable transport — wraps la-ayinspan trace context.

use crate::Transport;
use async_trait::async_trait;

/// Transport wrapper that emits trace spans for observability.
///
/// Wraps any `Transport` implementation and adds AYIN-compatible
/// trace spans for each request/notification. The production implementation
/// lives in the `lightarchitects-sdk`.
#[async_trait]
pub trait ObservableTransport: Transport {
    /// Get the trace context for the current request.
    fn trace_context(&self) -> &la_ayinspan::TraceContext;

    /// Get the actor that initiated this transport connection.
    fn actor(&self) -> &la_ayinspan::Actor;
}
