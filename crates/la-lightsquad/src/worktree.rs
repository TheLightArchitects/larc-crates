//! WorktreeManager trait — git worktree isolation for delivery tasks.

use crate::SquadError;
use serde::{Deserialize, Serialize};

/// Status of a worktree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum WorktreeStatus {
    /// Worktree exists and is ready for use.
    Active,
    /// Worktree has been cleaned up.
    Removed,
}

/// WorktreeManager trait — manages isolated git worktrees for task execution.
///
/// Each delivery task runs in its own worktree to prevent file conflicts
/// between concurrent tasks. The production implementation uses
/// `git worktree add/remove` under the hood.
///
/// The SDK provides the real implementation. External users can provide
/// alternative isolation strategies (Docker containers, chroot, etc.)
/// by implementing this trait.
#[async_trait::async_trait]
pub trait WorktreeManager: Send + Sync {
    /// Create a new worktree for a task.
    ///
    /// Returns the absolute path to the worktree root.
    async fn create(&self, task_id: &str, branch: &str) -> Result<String, SquadError>;

    /// Remove a worktree after task completion.
    async fn remove(&self, task_id: &str) -> Result<(), SquadError>;

    /// Check whether a worktree exists for a task.
    async fn status(&self, task_id: &str) -> Result<WorktreeStatus, SquadError>;

    /// List all active worktrees.
    async fn list_active(&self) -> Result<Vec<String>, SquadError>;
}
