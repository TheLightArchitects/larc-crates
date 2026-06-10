//! Archetype trait — what role a worker plays in the squad.
//!
//! See the crate-level documentation for a complete example.

/// What role an archetype plays in the squad.
///
/// This is `pub(crate)` until the dispatch feature provides `Executor` and
/// `ReviewGate` traits that reference it. Do not expose at v0.1.0.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ArchetypeRole {
    /// Produces work output — implements `Executor`.
    Worker,
    /// Reviews work against standards — implements `ReviewGate`.
    /// Has veto authority on its gate dimensions.
    Gatekeeper,
    /// Watches and records — no output, no veto, just traces.
    Observer,
}

/// Archetype trait — declares what role a worker plays in the squad.
///
/// Implement this trait to define a worker, gatekeeper, or observer
/// that the squad engine can route work to.
///
/// The production implementations (CORSO, EVA, SOUL, QUANTUM, SERAPH, LÆX, AYIN)
/// live in `lightarchitects-sdk`. External teams implement this trait for
/// their own domain.
///
/// # Example
///
/// ```rust
/// use larc_lightsquad::{Archetype, GateDimension, ToolDescriptor};
///
/// struct MySecurityWorker;
///
/// impl Archetype for MySecurityWorker {
///     fn name(&self) -> &str { "my-security-worker" }
///     fn domain(&self) -> &str { "security" }
///     fn role(&self) -> &str { "Reviews artifacts for vulnerabilities" }
///     fn gate_dimensions(&self) -> &[GateDimension] {
///         &[GateDimension::Security, GateDimension::Quality]
///     }
///     fn tools(&self) -> &[ToolDescriptor] { &[] }
/// }
/// ```
pub trait Archetype: Send + Sync {
    /// Human-readable name for this archetype (e.g., "my-security-worker").
    fn name(&self) -> &str;

    /// Domain this archetype operates in (e.g., "security", "medical_diagnostics").
    fn domain(&self) -> &str;

    /// Brief description of what this archetype does.
    fn role(&self) -> &str;

    /// Quality gate dimensions this archetype is responsible for.
    fn gate_dimensions(&self) -> &[GateDimension];

    /// Tools this archetype provides.
    fn tools(&self) -> &[ToolDescriptor];

    /// Optional personality tag — influences prompt construction in the engine.
    fn personality(&self) -> Option<&str> {
        None
    }
}

use crate::GateDimension;
use crate::ToolDescriptor;
use serde::{Deserialize, Serialize};
