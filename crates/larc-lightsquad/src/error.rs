//! Errors for `larc-lightsquad`.

/// Errors for the Structured Delivery Protocol.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SquadError {
    /// Plan validation failed.
    #[error("plan validation failed: {0}")]
    ValidationFailed(String),

    /// Task execution failed.
    #[error("task execution failed: {0}")]
    ExecutionFailed(String),

    /// Gate review returned a rejection.
    #[error("gate rejected: {dimensions:?} — {reason}")]
    GateRejected {
        dimensions: Vec<String>,
        reason: String,
    },

    /// Gate review deferred for further investigation.
    #[error("gate deferred: {dimensions:?} — {reason}")]
    GateDeferred {
        dimensions: Vec<String>,
        reason: String,
    },

    /// A dependency is not yet complete.
    #[error("dependency not met: task {task_id} is {status}")]
    DependencyNotMet { task_id: String, status: String },

    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// Worktree operation failed.
    #[error("worktree error: {0}")]
    Worktree(String),
}
