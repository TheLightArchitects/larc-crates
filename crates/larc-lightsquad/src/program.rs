//! BuildProgram trait — the top-level build lifecycle interface.

use crate::{BuildStatus, PlanInput, SquadError};

/// BuildProgram trait — defines a complete build lifecycle from plan to completion.
///
/// This is the highest-level orchestration interface. It takes a `PlanInput`
/// and drives it through all phases, waves, and quality gates to completion.
///
/// The production implementation in the SDK follows the LASDLC tier model:
/// - Small (4 phases, lighter gates)
/// - Medium (6 phases, standard gates)
/// - Large (7 phases, full gates)
///
/// External users implement this to define custom build pipelines
/// (CI/CD integration, multi-environment promotion, etc.).
#[async_trait::async_trait]
pub trait BuildProgram: Send + Sync {
    /// Start executing a build plan.
    ///
    /// Returns a build ID for tracking. The build runs asynchronously;
    /// use `status()` to check progress and `cancel()` to abort.
    async fn start(&self, plan: PlanInput) -> Result<String, SquadError>;

    /// Get the current status of a build.
    async fn status(&self, build_id: &str) -> Result<BuildStatus, SquadError>;

    /// Cancel a running build.
    ///
    /// In-progress tasks complete their current step before cancellation
    /// takes effect. Quality gates are not evaluated for cancelled builds.
    async fn cancel(&self, build_id: &str) -> Result<(), SquadError>;

    /// Wait for a build to complete, returning the final status.
    ///
    /// This blocks until the build reaches a terminal state
    /// (Complete or Failed).
    async fn wait(&self, build_id: &str) -> Result<BuildStatus, SquadError>;
}
