//! # larc-lightsquad
//!
//! Structured Delivery Protocol — archetype framework, delivery vocabulary,
//! pipeline contract types, and executor traits for autonomous squad orchestration.
//!
//! Provides the **vocabulary** for multi-agent software delivery systems: tasks,
//! waves, gate dimensions, archetypes, review verdicts, evidence bundles,
//! context vectors, and SSE event payloads. Bring your own engine.
//!
//! ## Default (no features) — zero async, pure data + [`Archetype`] trait
//!
//! Import the vocabulary and declare archetypes without pulling in an async runtime:
//!
//! ```rust
//! use larc_lightsquad::{Archetype, GateDimension, ToolDescriptor};
//!
//! struct MySecurityWorker;
//!
//! impl Archetype for MySecurityWorker {
//!     fn name(&self) -> &str { "my-security-worker" }
//!     fn domain(&self) -> &str { "security" }
//!     fn role(&self) -> &str { "Reviews artifacts for vulnerabilities" }
//!     fn gate_dimensions(&self) -> &[GateDimension] {
//!         &[GateDimension::Security, GateDimension::Quality]
//!     }
//!     fn tools(&self) -> &[ToolDescriptor] { &[] }
//! }
//! ```
//!
//! ## Feature `dispatch` — async delivery traits
//!
//! Enables [`Executor`], [`ReviewGate`], [`WaveDispatcher`], and [`WorktreeManager`].
//! Pulls in `tokio` and `async-trait`.
//!
//! ## Pipeline contract types
//!
//! For supervised multi-agent pipelines (Researcher → Coder → Reviewer → Tester):
//!
//! - [`ContextVector`] — orchestrator-assembled context (env manifest, interface stubs,
//!   constraint manifest)
//! - [`CriticReview`] + [`Vulnerability`] — structured review output with calibration baseline
//! - [`EvidenceBundle`] — correction loop input with allowlist-parsed diagnostics
//! - [`SanitizedTrace`] + [`CompilerDiagnostic`] — safe Tester→Coder feedback wire
//! - [`TestAssertions`] — Test-Driven Generation stubs (non-empty invariant)

// ── Always available: vocabulary + Archetype trait (zero async) ────────────

mod archetype;
mod contract;
mod critic;
mod error;
mod events;
mod evidence;
mod gate_dimension;
mod pipeline;
mod plan;
mod projection;
mod task;
mod tdd;
mod tool_descriptor;

pub use archetype::Archetype;
pub use contract::{Decision, DimensionScore, TaskContract, Verdict};
pub use critic::{
    CriticCalibration, CriticExemplar, CriticReview, FindingStatus, Vulnerability,
    VulnerabilityKind,
};
pub use error::SquadError;
pub use events::{
    AgentHeartbeatEvent, ConductorTickEvent, DecisionEntryDto, EscalationEvent,
    FixAgentIterationEvent, IterationMetricsEvent, MergeAgentStatusEvent,
};
pub use evidence::EvidenceBundle;
pub use gate_dimension::GateDimension;
pub use pipeline::{
    CompilerDiagnostic, ConstraintManifest, ContextVector, EnvironmentManifest, InterfaceStub,
    SanitizeError, SanitizedTrace,
};
pub use plan::{PlanInput, TaskInput, WaveInput};
pub use projection::Pillar;
pub use task::{AgentStatus, BuildStatus, ContextTier, Task, TaskStatus, Tier, WaveStatus};
pub use tdd::{EmptyAssertionsError, TestAssertion, TestAssertions};
pub use tool_descriptor::ToolDescriptor;

// ── Feature "dispatch": async delivery traits ─────────────────────────────

#[cfg(feature = "dispatch")]
mod executor;

#[cfg(feature = "dispatch")]
pub use executor::Executor;

#[cfg(feature = "dispatch")]
mod gate;

#[cfg(feature = "dispatch")]
pub use gate::ReviewGate;

#[cfg(feature = "dispatch")]
mod worktree;

#[cfg(feature = "dispatch")]
pub use worktree::{WorktreeManager, WorktreeStatus};

#[cfg(feature = "dispatch")]
mod dispatcher;

#[cfg(feature = "dispatch")]
pub use dispatcher::{Coordinator, WaveDispatcher};

#[cfg(feature = "dispatch")]
mod program;

#[cfg(feature = "dispatch")]
pub use program::BuildProgram;
