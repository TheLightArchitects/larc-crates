//! Plan input types — what you submit to start a build.

use serde::{Deserialize, Serialize};

/// Top-level input for starting a build.
///
/// Contains the build description, configuration, and waves of tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PlanInput {
    /// Human-readable name for this build.
    pub name: String,
    /// Build tier — determines phase structure and gate requirements.
    pub tier: crate::task::Tier,
    /// Waves of tasks to execute.
    pub waves: Vec<WaveInput>,
    /// Optional metadata.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// A wave of tasks that can execute in parallel.
///
/// Tasks within a wave have no dependencies on each other.
/// Waves execute sequentially — wave N+1 starts after wave N completes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct WaveInput {
    /// Human-readable name for this wave (e.g., "Phase 3: implementation").
    pub name: String,
    /// Tasks in this wave.
    pub tasks: Vec<TaskInput>,
}

impl WaveInput {
    /// Create a new wave input with the given name and tasks.
    #[must_use]
    pub fn new(name: String, tasks: Vec<TaskInput>) -> Self {
        Self { name, tasks }
    }
}

/// A single task to execute within a wave.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TaskInput {
    /// Unique identifier for this task.
    pub id: String,
    /// The prompt or instruction for the worker.
    pub prompt: String,
    /// Context tiers to include (canon, project, session).
    #[serde(default)]
    pub context_tiers: Vec<crate::task::ContextTier>,
    /// Files this task is allowed to modify.
    #[serde(default)]
    pub file_ownership: Vec<String>,
    /// IDs of tasks this task depends on (must complete first).
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Whether this task is safe to run concurrently with other tasks.
    #[serde(default = "default_concurrency_safe")]
    pub concurrency_safe: bool,
}

impl TaskInput {
    /// Create a new task input with the given ID and prompt.
    #[must_use]
    pub fn new(id: String, prompt: String) -> Self {
        Self {
            id,
            prompt,
            context_tiers: Vec::new(),
            file_ownership: Vec::new(),
            depends_on: Vec::new(),
            concurrency_safe: false,
        }
    }
}

fn default_concurrency_safe() -> bool {
    false
}

impl PlanInput {
    /// Create a new plan input with the given name, tier, and waves.
    #[must_use]
    pub fn new(name: String, tier: crate::task::Tier, waves: Vec<WaveInput>) -> Self {
        Self {
            name,
            tier,
            waves,
            metadata: serde_json::Value::Null,
        }
    }

    /// Validate the plan structure.
    ///
    /// Checks for:
    /// - Non-empty waves
    /// - Unique task IDs within the plan
    /// - Dependency references exist
    pub fn validate(&self) -> Result<(), crate::SquadError> {
        if self.waves.is_empty() {
            return Err(crate::SquadError::ValidationFailed(
                "plan must have at least one wave".to_string(),
            ));
        }

        let mut task_ids = std::collections::HashSet::new();
        let mut duplicates = Vec::new();

        for wave in &self.waves {
            for task in &wave.tasks {
                if !task_ids.insert(task.id.clone()) {
                    duplicates.push(task.id.clone());
                }
            }
        }

        if !duplicates.is_empty() {
            return Err(crate::SquadError::ValidationFailed(format!(
                "duplicate task IDs: {}",
                duplicates.join(", ")
            )));
        }

        // Check dependency references
        for wave in &self.waves {
            for task in &wave.tasks {
                for dep in &task.depends_on {
                    if !task_ids.contains(dep) {
                        return Err(crate::SquadError::ValidationFailed(format!(
                            "task '{}' depends on unknown task '{}'",
                            task.id, dep
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}
