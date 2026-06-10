//! ReviewGate trait — the quality gate interface for Gatekeeper archetypes.

use crate::{Archetype, Decision, Task};

/// ReviewGate trait — implemented by anything that reviews work against standards.
///
/// Gatekeepers have veto authority on their gate dimensions. If a Gatekeeper
/// returns [`Verdict::Reject`], the work must be revised before proceeding.
///
/// Implement this trait to define a quality gate. A single gate may own
/// multiple [`GateDimension`]s (see [`Archetype::gate_dimensions`]).
///
/// [`Verdict::Reject`]: crate::Verdict::Reject
/// [`GateDimension`]: crate::GateDimension
/// [`Archetype::gate_dimensions`]: crate::Archetype::gate_dimensions
///
/// # Example
///
/// ```rust,ignore
/// use larc_lightsquad::{Archetype, ReviewGate, Decision, Verdict, Task, DimensionScore};
///
/// struct MyQualityGate;
///
/// impl Archetype for MyQualityGate {
///     fn gate_dimensions(&self) -> &[GateDimension] {
///         &[GateDimension::Quality, GateDimension::Testing]
///     }
///     // ...
/// }
///
/// #[async_trait::async_trait]
/// impl ReviewGate for MyQualityGate {
///     async fn review(&self, task: &Task) -> Decision {
///         // Review the work produced by the task
///         Decision {
///             verdict: Verdict::Approve { message: None },
///             scores: vec![],
///             aggregate_score: 95.0,
///         }
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait ReviewGate: Archetype {
    /// Review the work produced by a task and return a decision.
    ///
    /// The decision includes a verdict (Approve/Reject/Defer),
    /// per-dimension scores, and an aggregate score.
    async fn review(&self, task: &Task) -> Decision;
}
