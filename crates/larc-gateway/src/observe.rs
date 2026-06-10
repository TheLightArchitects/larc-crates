//! Observable transport — wraps trace context from `larc-ayinspan`.

use crate::Transport;
use async_trait::async_trait;

/// Transport wrapper that emits trace spans for observability.
///
/// Wraps any [`Transport`] implementation and adds trace spans for each
/// request/notification, compatible with the [`larc_ayinspan`] span model.
/// Consumers provide the concrete emitter (file, HTTP, in-memory).
#[async_trait]
pub trait ObservableTransport: Transport {
    /// Get the trace context for the current request.
    fn trace_context(&self) -> &larc_ayinspan::TraceContext;

    /// Get the actor that initiated this transport connection.
    fn actor(&self) -> &larc_ayinspan::Actor;
}
