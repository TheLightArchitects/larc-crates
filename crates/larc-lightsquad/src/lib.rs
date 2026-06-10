//! # la-lightsquad
//!
//! Structured Delivery Protocol — archetype framework, delivery vocabulary,
//! and executor traits for autonomous squad orchestration.
//!
//! This crate defines the **protocol and vocabulary** for structured software
//! delivery. The private `lightarchitects-sdk` provides the engine that
//! executes it.
//!
//! ## Default (no features) — zero async, pure data + Archetype trait
//!
//! Import the vocabulary and declare archetypes without pulling in tokio:
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
//! Enables `Executor`, `ReviewGate`, `WaveDispatcher`, and `WorktreeManager`.
//!
//! ## Architecture
//!
//! ```text
//! la-lightsquad (open — protocol + vocabulary + archetype shapes)
//!        │
//!        │ implements
//!        ▼
//! lightarchitects-sdk (private — engine + canon + implementations)
//! ```
//!
//! The archetype shapes (SecurityArchetype, CanonKeeperArchetype, etc.)
//! live in documentation as examples, not in the crate. Anything in the
//! crate at v0.1.0 is a semver promise. Fictional doc examples are not.
//!
//! ## What stays in the SDK (the moat)
//!
//! - The Coordinator (7-slot worker pool, wave dispatch)
//! - The Decision Pipeline (Canon → Northstar → LightArchitect → User)
//! - The personality engine (prompt construction per archetype)
//! - 4-signal RRF retrieval
//! - ReviewGate loop (MAX_GATE_ITERATIONS = 3)
//! - Context tier token budgeting
//! - HMAC task verification
//! - CORSO's 7-pillar methodology, EVA's consciousness model, etc.
//!
//! ## Pre-Publish Migration Debt (v0.1.0 gate)
//!
//! | # | Item | Status | Resolution |
//! |---|------|--------|------------|
//! | D-1 | `EscalationEvent.worker_slot` — required `u8` in Rust, optional in TS | **RESOLVED** | `Option<u8>` with `#[serde(default)]` |
//! | D-2 | `EscalationEvent.canon_ref` — present in TS, absent in Rust | **RESOLVED** | `Option<String>` with `#[serde(default, skip_serializing_if)]` |
//! | D-3 | `DecisionEntryDto` — TS DTO shape vs SDK `HashChain::DecisionEntry` internal | **RESOLVED** | Promoted to `src/events.rs`; crypto fields never exposed |
//! | D-4 | `ContextTier.tier` — `String` in SDK (`"T1"/"T2"/"T3"`), `u8` in this crate | **RESOLVED** | `u8` canonical; `tier_from_string`/`tier_to_string` bridge methods |
//! | D-5 | `GateDimension` duplication — SDK has its own 10-variant copy without `Custom` | **RESOLVED** | SDK re-exports `larc_lightsquad::GateDimension` |
//! | D-6 | SSE envelope strategy — TS sends `"type"` inline; Rust structs omit it | **RESOLVED** | No `deny_unknown_fields`; serde silently ignores unknown fields (documented in `events.rs`) |

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
    SanitizedTrace,
};
pub use plan::{PlanInput, TaskInput, WaveInput};
pub use projection::Pillar;
pub use task::{AgentStatus, BuildStatus, ContextTier, Task, TaskStatus, Tier, WaveStatus};
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
