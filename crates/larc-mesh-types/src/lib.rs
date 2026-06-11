//! # larc-mesh-types
//!
//! Shared identity and scope vocabulary for multi-agent collective memory.
//!
//! This crate is types-only: no async runtime, no I/O, no transitive heavy
//! dependencies. It is the smallest unit on which higher-level crates
//! (governance gates, mesh registries, lesson stores) can agree about *what*
//! a sibling is, *what* a squad is, *what* a run is, and *what* scope a
//! memory write targets — without coupling those higher-level crates to each
//! other.
//!
//! ## Modules
//!
//! The skeleton is intentionally empty. Types land in subsequent phases:
//!
//! - `identity` — `SiblingId`, `SquadId`, `RunId` (Phase 1)
//! - `scope`    — `MemoryScope` enum + variants (Phase 1)
//!
//! See the lightsquad-soul-mesh build plan for the type-extraction contract.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// Type modules added in subsequent phases of the lightsquad-soul-mesh build.
// Re-exports will live in this top-level lib.rs per the
// `feedback_re_exports_are_api_contract` discipline.
