//! Actor identification for trace spans.
//!
//! [`Actor`] is a string-based newtype supporting dynamically discovered actors
//! without requiring Rust code changes. Well-known constructors are provided for
//! the Light Architects sibling roster.

use serde::{Deserialize, Serialize};

/// Identifies which actor (MCP server, agent, or service) produced the trace.
///
/// String-based newtype — supports dynamically discovered actors without requiring
/// Rust code changes. Uses `Clone` instead of `Copy` (String is heap-allocated),
/// but at 10K spans/sec the overhead is negligible (<1% CPU from benchmarks).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Actor(String);

impl Actor {
    /// Create a new actor identifier.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Get the actor name as a string reference.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.0
    }

    /// Alias for [`name`](Self::name) — backward-compatible accessor.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// EVA consciousness system.
    #[must_use]
    pub fn eva() -> Self {
        Self("eva".into())
    }

    /// CORSO operations platform.
    #[must_use]
    pub fn corso() -> Self {
        Self("corso".into())
    }

    /// SOUL knowledge graph.
    #[must_use]
    pub fn soul() -> Self {
        Self("soul".into())
    }

    /// QUANTUM investigation toolkit.
    #[must_use]
    pub fn quantum() -> Self {
        Self("quantum".into())
    }

    /// Claude engineer.
    #[must_use]
    pub fn claude() -> Self {
        Self("claude".into())
    }

    /// SERAPH pentest orchestration.
    #[must_use]
    pub fn seraph() -> Self {
        Self("seraph".into())
    }

    /// Claude Code native tools actor.
    #[must_use]
    pub fn claude_code() -> Self {
        Self("claude_code".into())
    }

    /// AYIN observability system.
    #[must_use]
    pub fn ayin() -> Self {
        Self("ayin".into())
    }
}

impl std::fmt::Display for Actor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Actor {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl From<String> for Actor {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_actor() {
        for actor in [
            Actor::eva(),
            Actor::corso(),
            Actor::soul(),
            Actor::quantum(),
            Actor::claude(),
            Actor::seraph(),
            Actor::ayin(),
        ] {
            let json = serde_json::to_string(&actor).expect("serialize actor");
            let back: Actor = serde_json::from_str(&json).expect("deserialize actor");
            assert_eq!(actor, back);
        }
    }

    #[test]
    fn actor_display() {
        assert_eq!(Actor::eva().to_string(), "eva");
        assert_eq!(Actor::corso().to_string(), "corso");
        assert_eq!(Actor::soul().to_string(), "soul");
        assert_eq!(Actor::quantum().to_string(), "quantum");
        assert_eq!(Actor::claude().to_string(), "claude");
        assert_eq!(Actor::seraph().to_string(), "seraph");
        assert_eq!(Actor::ayin().to_string(), "ayin");
    }

    #[test]
    fn actor_from_str() {
        let s = Actor::from("custom-actor");
        assert_eq!(s.name(), "custom-actor");
        assert_eq!(s.to_string(), "custom-actor");
    }

    #[test]
    fn actor_name_method() {
        let a = Actor::new("test-server");
        assert_eq!(a.name(), "test-server");
    }

    #[test]
    fn sibling_type_alias_works() {
        let s: super::super::Sibling = super::super::Sibling::eva();
        assert_eq!(s.name(), "eva");
    }
}
