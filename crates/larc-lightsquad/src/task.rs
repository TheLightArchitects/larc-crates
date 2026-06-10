//! Task and build status types.

use serde::{Deserialize, Serialize};

/// Build tier — determines phase structure and gate requirements.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Tier {
    /// 4 phases, lighter gate requirements.
    Small,
    /// 6 phases, standard gate requirements.
    Medium,
    /// 7 phases, full gate requirements.
    Large,
}

/// Context tier — determines what context is provided to a task.
///
/// | Tier | Content | Token budget |
/// |------|---------|-------------|
/// | Canon | Standards documents, cookbooks | ~15K tokens |
/// | Project | CLAUDE.md, existing code | ~10K tokens |
/// | Session | Recent conversation, task-specific context | ~5K tokens |
///
/// The `tier` field uses `u8` (0, 1, 2) as the canonical representation.
/// The SDK uses `String` (`"T1"`, `"T2"`, `"T3"`) internally. Conversion
/// methods bridge the two: [`tier_from_string`] for SDK → public, and
/// [`tier_to_string`] for public → SDK.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ContextTier {
    /// Tier number (0 = canon, 1 = project, 2 = session).
    ///
    /// The canonical wire format. The SDK's `"T1"/"T2"/"T3"` string format
    /// converts to/from this via [`tier_from_string`] and [`tier_to_string`].
    pub tier: u8,
    /// Human-readable label.
    pub label: String,
    /// Files included in this tier.
    pub files: Vec<String>,
    /// Estimated token count.
    pub token_estimate: usize,
}

impl ContextTier {
    /// Create a new context tier.
    #[must_use]
    pub fn new(tier: u8, label: String, files: Vec<String>, token_estimate: usize) -> Self {
        Self {
            tier,
            label,
            files,
            token_estimate,
        }
    }

    /// Convert from the SDK's string tier format (`"T1"`, `"T2"`, `"T3"`)
    /// to the canonical `u8` tier number.
    ///
    /// Returns `None` for unrecognized tier strings.
    #[must_use]
    pub fn tier_from_string(s: &str) -> Option<u8> {
        match s {
            "T1" | "t1" => Some(0),
            "T2" | "t2" => Some(1),
            "T3" | "t3" => Some(2),
            _ => None,
        }
    }

    /// Convert from the canonical `u8` tier number to the SDK's string format.
    ///
    /// Returns `None` for tier numbers outside 0–2.
    #[must_use]
    pub fn tier_to_string(tier: u8) -> Option<&'static str> {
        match tier {
            0 => Some("T1"),
            1 => Some("T2"),
            2 => Some("T3"),
            _ => None,
        }
    }
}

/// Status of a delivery task.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TaskStatus {
    /// Not yet started.
    Pending,
    /// Currently executing.
    InProgress,
    /// Completed successfully.
    Complete,
    /// Failed with an error.
    Failed,
}

/// Status of a delivery wave.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum WaveStatus {
    /// Not yet started.
    Pending,
    /// Currently executing.
    Running,
    /// All tasks completed.
    Complete,
    /// One or more tasks failed.
    Failed,
}

/// Status of a build pipeline.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BuildStatus {
    /// Not yet started.
    Pending,
    /// Currently executing waves.
    Running,
    /// Running quality gates.
    Gating,
    /// All phases and gates passed.
    Complete,
    /// A gate or task failed.
    Failed,
}

/// Status of a delivery agent in the worker pool.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AgentStatus {
    /// Agent is available for work.
    Idle,
    /// Agent is currently executing a task.
    Running,
    /// Agent is paused (e.g., waiting for HITL approval).
    Paused,
    /// Agent has completed its assignment.
    Done,
}

/// A delivery task with full context.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Task {
    /// Unique identifier.
    pub id: String,
    /// Git branch for this task.
    pub branch: String,
    /// IDs of tasks this task depends on.
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Files this task is allowed to modify.
    #[serde(default)]
    pub file_ownership: Vec<String>,
    /// Whether this task is safe to run concurrently.
    #[serde(default)]
    pub concurrency_safe: bool,
    /// Context tiers for this task.
    #[serde(default)]
    pub context_tiers: Vec<ContextTier>,
    /// The prompt or instruction for the worker.
    pub prompt: String,
    /// Current status.
    pub status: TaskStatus,
    /// Trace identifier linking this task to its originating pipeline span.
    ///
    /// Set by the orchestrator at pipeline entry and threaded through every
    /// agent handoff. Format: UUID v4 string. `None` for tasks created
    /// outside a traced pipeline.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

impl Task {
    /// Create a new task with required fields and defaults for all optional fields.
    #[must_use]
    pub fn new(id: String, branch: String, prompt: String, status: TaskStatus) -> Self {
        Self {
            id,
            branch,
            prompt,
            status,
            depends_on: Vec::new(),
            file_ownership: Vec::new(),
            concurrency_safe: false,
            context_tiers: Vec::new(),
            trace_id: None,
        }
    }

    /// Check if this task can run given the current state of other tasks.
    ///
    /// Returns `true` if all dependencies are `Complete` and this task is `Pending`.
    #[must_use]
    pub fn can_run(&self, tasks: &[Task]) -> bool {
        if self.status != TaskStatus::Pending {
            return false;
        }
        self.depends_on.iter().all(|dep_id| {
            tasks
                .iter()
                .find(|t| &t.id == dep_id)
                .is_some_and(|t| t.status == TaskStatus::Complete)
        })
    }
}
