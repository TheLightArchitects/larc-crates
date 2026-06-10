//! Executor trait — the core delivery interface for Worker archetypes.

use crate::{Archetype, Task, TaskContract, TaskStatus};

/// Executor trait — implemented by anything that can perform work.
///
/// Implement this trait to define a worker for your domain. The executor
/// receives a [`Task`] and reports a [`TaskStatus`] when complete; the
/// [`TaskContract`] declares which files the worker owns and whether the
/// work is safe to run concurrently with other tasks.
///
/// # Example
///
/// ```rust,ignore
/// use larc_lightsquad::{Archetype, Executor, Task, TaskContract, TaskStatus};
///
/// struct MyWorker;
///
/// impl Archetype for MyWorker {
///     // ... archetype implementation
/// }
///
/// #[async_trait::async_trait]
/// impl Executor for MyWorker {
///     async fn execute(&self, task: &Task) -> Result<TaskStatus, crate::SquadError> {
///         // Do the work described by the task prompt
///         Ok(TaskStatus::Complete)
///     }
///
///     fn contract(&self) -> TaskContract {
///         TaskContract {
///             file_ownership: vec!["src/".to_string()],
///             concurrency_safe: true,
///         }
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait Executor: Archetype {
    /// Execute a task and return its final status.
    ///
    /// Implementations should perform the work described by `task.prompt`
    /// and return `TaskStatus::Complete` on success or `TaskStatus::Failed`
    /// with an appropriate error on failure.
    async fn execute(&self, task: &Task) -> Result<TaskStatus, crate::SquadError>;

    /// Return the contract for this executor — which files it owns
    /// and whether it can run concurrently with other tasks.
    fn contract(&self) -> TaskContract;
}
