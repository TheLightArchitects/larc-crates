//! WaveDispatcher and Coordinator — orchestration shapes for multi-wave delivery.

use crate::{AgentStatus, BuildStatus, PlanInput, SquadError, Task, TaskStatus};

/// WaveDispatcher trait — dispatches tasks within a wave to available executors.
///
/// Implement this trait to define a dispatch strategy for tasks within a wave
/// (priority queues, GPU affinity, cost-based routing, fixed worker pools, etc.).
/// Tasks with `concurrency_safe: true` and non-overlapping file ownership
/// can run in parallel; others must run sequentially.
#[async_trait::async_trait]
pub trait WaveDispatcher: Send + Sync {
    /// Dispatch a set of tasks within a single wave.
    ///
    /// Tasks with `concurrency_safe: true` and non-overlapping file ownership
    /// can run in parallel. Others must run sequentially.
    async fn dispatch_wave(&self, tasks: &[Task]) -> Result<Vec<TaskStatus>, SquadError>;
}

/// Coordinator trait — manages the full build lifecycle across multiple waves.
///
/// Implement this trait to define an orchestration strategy. A typical
/// coordinator handles:
/// 1. Wave sequencing (wave N+1 starts after wave N completes)
/// 2. Dependency resolution (tasks wait for their `depends_on`)
/// 3. Quality gates between phases
/// 4. HITL checkpoints for gate deferrals
/// 5. Worktree lifecycle (create before, remove after)
#[async_trait::async_trait]
pub trait Coordinator: Send + Sync {
    /// Submit a build plan for execution.
    ///
    /// Returns a build ID for tracking progress.
    async fn submit_plan(&self, plan: PlanInput) -> Result<String, SquadError>;

    /// Get the current status of a build.
    async fn build_status(&self, build_id: &str) -> Result<BuildStatus, SquadError>;

    /// Get the status of all agents in the pool.
    async fn agent_statuses(&self) -> Result<Vec<(String, AgentStatus)>, SquadError>;

    /// Cancel a running build.
    async fn cancel_build(&self, build_id: &str) -> Result<(), SquadError>;

    /// Approve a deferred gate (HITL checkpoint).
    async fn approve_gate(&self, build_id: &str, gate_id: &str) -> Result<(), SquadError>;

    /// Reject a deferred gate (HITL checkpoint).
    async fn reject_gate(
        &self,
        build_id: &str,
        gate_id: &str,
        reason: &str,
    ) -> Result<(), SquadError>;
}
