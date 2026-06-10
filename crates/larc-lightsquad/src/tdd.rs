//! Test-Driven Generation types — structured test assertion stubs.
//!
//! Implements the `target_test_assertions` contract from Builders Cookbook §83.
//! The Coder agent is blocked from generating production code until a non-empty
//! `TestAssertions` record exists — one stub per acceptance criterion.

use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

// ── TestAssertion ─────────────────────────────────────────────────────────────

/// A single test assertion stub from the TDG protocol (Cookbook §83).
///
/// Maps one acceptance criterion to the test scaffold that will verify it.
/// Stubs are committed BEFORE production code; the Coder agent writes the
/// production implementation to satisfy them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TestAssertion {
    /// The acceptance criterion this assertion validates.
    ///
    /// Plain text, e.g. `"fetch_user returns Ok(User) for a valid id"`.
    pub criterion: String,
    /// The test function name or describe block that will verify the criterion.
    ///
    /// E.g. `"test_fetch_user_returns_ok_for_valid_id"` (Rust) or
    /// `"returns 200 for valid id"` (Jest describe block label).
    pub test_stub: String,
}

impl TestAssertion {
    /// Create a new test assertion stub.
    #[must_use]
    pub fn new(criterion: String, test_stub: String) -> Self {
        Self {
            criterion,
            test_stub,
        }
    }
}

// ── EmptyAssertionsError ──────────────────────────────────────────────────────

/// Error returned when attempting to create an empty `TestAssertions` collection.
///
/// Per Cookbook §83: `target_test_assertions` must be non-empty. An empty
/// collection is a gate failure.
#[derive(Debug, Error, PartialEq)]
#[error("target_test_assertions must be non-empty (Builders Cookbook §83)")]
pub struct EmptyAssertionsError;

// ── TestAssertions ────────────────────────────────────────────────────────────

/// A non-empty collection of test assertion stubs.
///
/// The TDG gate (Cookbook §83) requires at least one stub per acceptance
/// criterion BEFORE the Coder agent generates production code. The non-empty
/// invariant is enforced at:
///
/// 1. **Construction** — [`TestAssertions::new`] rejects empty `Vec`.
/// 2. **Deserialization** — the custom `Deserialize` impl rejects empty JSON arrays.
///
/// There is no public way to construct an empty `TestAssertions`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TestAssertions(Vec<TestAssertion>);

impl TestAssertions {
    /// Create a non-empty `TestAssertions` collection.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyAssertionsError`] if `assertions` is empty.
    pub fn new(assertions: Vec<TestAssertion>) -> Result<Self, EmptyAssertionsError> {
        if assertions.is_empty() {
            return Err(EmptyAssertionsError);
        }
        Ok(Self(assertions))
    }

    /// Returns the assertions as a slice.
    #[must_use]
    pub fn as_slice(&self) -> &[TestAssertion] {
        &self.0
    }

    /// Returns the number of assertion stubs.
    ///
    /// Always ≥ 1 by construction.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Always returns `false`.
    ///
    /// `TestAssertions` is non-empty by construction. This method exists to
    /// satisfy the `clippy::len_without_is_empty` lint.
    #[must_use]
    #[allow(clippy::unused_self)]
    pub fn is_empty(&self) -> bool {
        false
    }
}

impl<'de> Deserialize<'de> for TestAssertions {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let v = Vec::<TestAssertion>::deserialize(deserializer)?;
        if v.is_empty() {
            return Err(serde::de::Error::custom(
                "target_test_assertions must be non-empty (Builders Cookbook §83)",
            ));
        }
        Ok(Self(v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assertions_new_rejects_empty() {
        assert_eq!(TestAssertions::new(vec![]), Err(EmptyAssertionsError));
    }

    #[test]
    fn test_assertions_new_accepts_nonempty() {
        let ta = TestAssertion::new("fetch_user returns Ok".into(), "test_fetch_user_ok".into());
        let assertions = TestAssertions::new(vec![ta.clone()]).unwrap();
        assert_eq!(assertions.len(), 1);
        assert!(!assertions.is_empty());
        assert_eq!(assertions.as_slice()[0], ta);
    }

    #[test]
    fn test_assertions_deserialize_rejects_empty_array() {
        let json = "[]";
        let result = serde_json::from_str::<TestAssertions>(json);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("non-empty"),
            "error message should mention non-empty: {msg}"
        );
    }

    #[test]
    fn test_assertions_roundtrip_json() {
        let stubs = TestAssertions::new(vec![
            TestAssertion::new("criterion A".into(), "test_a".into()),
            TestAssertion::new("criterion B".into(), "test_b".into()),
        ])
        .unwrap();
        let json = serde_json::to_string(&stubs).unwrap();
        let back: TestAssertions = serde_json::from_str(&json).unwrap();
        assert_eq!(stubs, back);
        assert_eq!(back.len(), 2);
    }
}
