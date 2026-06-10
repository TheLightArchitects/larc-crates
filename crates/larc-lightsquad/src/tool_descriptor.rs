//! Tool descriptor — declares a capability an archetype provides.

use serde::{Deserialize, Serialize};

/// A tool that an archetype provides.
///
/// Tools are descriptive — they declare what an archetype can do,
/// not how it does it. The engine routes work to archetypes based on
/// their declared tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ToolDescriptor {
    /// Short name for this tool (e.g., "audit", "scan", "review").
    pub name: String,
    /// One-line description of what this tool does.
    pub description: String,
}

impl ToolDescriptor {
    /// Create a new tool descriptor.
    #[must_use]
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
        }
    }
}
