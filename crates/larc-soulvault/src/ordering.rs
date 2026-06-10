//! Ordering mode for helix steps.

use serde::{Deserialize, Serialize};

/// Ordering mode for steps within a helix.
///
/// Declared on the `Helix` node, determines step sort order.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum HelixOrderingMode {
    /// Order by `step_date` ASC — consciousness entries, journals, builds.
    Temporal,
    /// Order by `step_index` ASC — bible chapters, plan phases, numbered sequences.
    Indexed,
    /// Order by metadata key — application-specific ordering.
    Custom,
}

impl Default for HelixOrderingMode {
    fn default() -> Self {
        Self::Temporal
    }
}
