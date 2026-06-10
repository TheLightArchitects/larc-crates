//! # la-ayinspan
//!
//! Observability span types for the Light Architects platform.
//!
//! Provides [`TraceSpan`], [`Actor`], [`TraceOutcome`], [`TraceContext`] (builder),
//! and W3C traceparent propagation — with zero runtime dependencies (no tokio, no axum).
//!
//! This is the canonical source for AYIN span types. The SDK re-exports these;
//! AYIN-DEV and the gateway both consume this crate directly.
//!
//! ## Quick start
//!
//! ```rust
//! use larc_ayinspan::{TraceContext, Actor, TraceOutcome};
//!
//! let span = TraceContext::new(Actor::corso(), "guard")
//!     .session_id("sess-123")
//!     .outcome(TraceOutcome::Continue)
//!     .finish()
//!     .expect("outcome is set");
//! assert_eq!(span.actor, Actor::corso());
//! ```

mod actor;
mod decision;
mod error;
mod hierarchy;
mod outcome;
mod propagation;
mod semconv;
mod span;
mod strand;

pub mod emit;
pub mod observe;
pub mod turn;

/// Atomic counters for batch emission observability.
///
/// Available unconditionally — reads are always valid even when the `batch`
/// feature is disabled (counters will simply remain at zero).
pub mod metrics;

/// Batched span emission — requires the `batch` feature.
#[cfg(feature = "batch")]
pub mod batch;

pub use actor::Actor;
pub use decision::DecisionPoint;
pub use error::TraceError;
pub use hierarchy::SpanHierarchy;
pub use outcome::TraceOutcome;
pub use propagation::{PropagationContext, SESSION_PROPAGATION_KEY, TURN_INDEX_KEY};
pub use semconv::lasdlc;
pub use span::{TraceContext, TraceSpan};
pub use strand::StrandActivation;

// Flat re-exports for the most-used turn/emit/observe types.
pub use emit::{NullSpanEmitter, SpanEmit};
pub use observe::{NullSpanObserver, SpanObserve};
pub use turn::{TurnContext, TurnTracker, TurnTracking};

/// Backward-compatible alias: `Sibling` is now [`Actor`].
pub type Sibling = Actor;
