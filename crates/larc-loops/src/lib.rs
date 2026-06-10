//! # larc-loops
//!
//! Agentic loop convergence traits — strategy-agnostic interfaces for deciding
//! when to halt, continue, or escalate an agentic execution loop.
//!
//! Provides [`ConvergenceResult`] and six convergence strategy traits. Implement
//! whichever fits your loop topology; compose multiple traits for hybrid strategies.
//!
//! ## Traits
//!
//! | Trait | Topology |
//! |-------|----------|
//! | [`BlastScore`] | Compound risk/score threshold |
//! | [`ConvergenceGate`] | Gate-by-gate with phase-back |
//! | [`NPassVerifier`] | N independent verification rounds |
//! | [`QueueDrain`] | Bounded queue exhaustion |
//! | [`InterestDecay`] | Simulated annealing cooling |
//! | [`IntervalWatch`] | Interval polling with deadline |
//!
//! ## Quick start
//!
//! ```rust
//! use larc_loops::{ConvergenceResult, BlastScore};
//!
//! struct ScoreCheck;
//! impl BlastScore for ScoreCheck {
//!     fn check(&self, score: f64, threshold: f64) -> ConvergenceResult {
//!         if score <= threshold {
//!             ConvergenceResult::Converged
//!         } else {
//!             ConvergenceResult::Blocked {
//!                 reason: format!("blast score {score:.2} > {threshold:.2}"),
//!             }
//!         }
//!     }
//! }
//! ```

// ── ConvergenceResult ─────────────────────────────────────────────────────────

/// Outcome of a single convergence check.
#[derive(Debug, Clone, PartialEq)]
pub enum ConvergenceResult {
    /// Loop should continue — threshold not yet reached.
    Continue,
    /// Loop has converged — enough evidence to halt or advance.
    Converged,
    /// Loop cannot converge on this path — caller should phase-back or abort.
    Blocked { reason: String },
}

impl ConvergenceResult {
    /// Returns `true` if the result is [`Converged`](Self::Converged).
    #[must_use]
    pub fn is_converged(&self) -> bool {
        matches!(self, Self::Converged)
    }

    /// Returns `true` if the result is [`Blocked`](Self::Blocked).
    #[must_use]
    pub fn is_blocked(&self) -> bool {
        matches!(self, Self::Blocked { .. })
    }
}

// ── BlastScore ────────────────────────────────────────────────────────────────

/// Convergence via compound risk scoring.
///
/// The loop converges when accumulated evidence or a normalised risk estimate
/// crosses a caller-defined threshold. Suitable for FAIR/Bowtie-style blast
/// score models where confidence accumulates incrementally.
pub trait BlastScore: Send + Sync {
    /// Evaluate whether the current blast score satisfies the convergence threshold.
    ///
    /// `score` is a `[0.0, 1.0]` normalised risk estimate; `threshold` is the
    /// minimum score at which the loop should advance.
    fn check(&self, score: f64, threshold: f64) -> ConvergenceResult;
}

// ── ConvergenceGate ───────────────────────────────────────────────────────────

/// Gate-by-gate convergence with phase-back capability.
///
/// Each gate must pass before the next can be evaluated. A failing gate
/// returns `Blocked` with the owning gate label, allowing the caller to
/// phase-back to the correct remediation point.
pub trait ConvergenceGate: Send + Sync {
    /// Evaluate a single gate pass.
    ///
    /// Returns [`Converged`](ConvergenceResult::Converged) if the gate passes,
    /// or [`Blocked`](ConvergenceResult::Blocked) with the owning gate label if not.
    fn evaluate_gate(&self, gate_label: &str, passed: bool) -> ConvergenceResult;
}

// ── NPassVerifier ─────────────────────────────────────────────────────────────

/// Convergence via N independent verification rounds.
///
/// The loop converges when `required_passes` rounds all succeed (unanimous),
/// or when a majority threshold is reached under a configurable policy.
/// Suitable for adversarial verification where a single pass may be unreliable.
pub trait NPassVerifier: Send + Sync {
    /// Record the result of pass `pass_index` and check whether convergence is reached.
    ///
    /// `pass_index` is 0-based. `passed` indicates whether this pass succeeded.
    /// `required_passes` is the total number of passes required for convergence.
    fn record_pass(&self, pass_index: u32, passed: bool, required_passes: u32)
        -> ConvergenceResult;
}

// ── QueueDrain ────────────────────────────────────────────────────────────────

/// Convergence via bounded queue exhaustion.
///
/// The loop converges when the processing queue is empty or falls below a
/// caller-defined residual threshold. Suitable for work-stealing or
/// fan-out loops that drain a finite item set.
pub trait QueueDrain: Send + Sync {
    /// Check whether the queue has drained to the point of convergence.
    ///
    /// `remaining` is the number of unprocessed items. `initial_size` is the
    /// queue size at the start of the drain loop.
    fn check_drained(&self, remaining: usize, initial_size: usize) -> ConvergenceResult;
}

// ── InterestDecay ─────────────────────────────────────────────────────────────

/// Convergence via simulated annealing cooling (Kirkpatrick 1983).
#[doc(hidden)] // Not yet assigned to a strategy; reserved for future use.
///
/// Models a "temperature" that decreases over time, making convergence
/// progressively more likely as the loop explores the solution space.
///
/// Not yet assigned to a specific strategy; reserved for future use.
pub trait InterestDecay: Send + Sync {
    /// Evaluate whether the current temperature has cooled to the convergence threshold.
    ///
    /// `temperature` decreases monotonically. `min_temperature` is the threshold
    /// at which the loop is considered converged.
    fn check_cooled(&self, temperature: f64, min_temperature: f64) -> ConvergenceResult;
}

// ── IntervalWatch ─────────────────────────────────────────────────────────────

/// Convergence via classic interval polling.
///
/// Models a watched condition that is polled on a fixed interval; convergence
/// occurs when the condition becomes true within a deadline.
///
/// Not yet assigned to a specific strategy; reserved for future use.
#[doc(hidden)] // Not yet assigned to a strategy; reserved for future use.
pub trait IntervalWatch: Send + Sync {
    /// Check whether the watched condition has become true.
    ///
    /// `elapsed_ms` is the total time elapsed since the watch began.
    /// `deadline_ms` is the maximum time before the watch expires.
    fn check_condition(&self, elapsed_ms: u64, deadline_ms: u64) -> ConvergenceResult;
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    struct ScoreCheck;
    impl BlastScore for ScoreCheck {
        fn check(&self, score: f64, threshold: f64) -> ConvergenceResult {
            if score <= threshold {
                ConvergenceResult::Converged
            } else {
                ConvergenceResult::Blocked {
                    reason: format!("score {score:.2} exceeds threshold {threshold:.2}"),
                }
            }
        }
    }

    #[test]
    fn blast_score_converges_at_or_below_threshold() {
        let b = ScoreCheck;
        assert_eq!(b.check(0.5, 0.5), ConvergenceResult::Converged);
        assert_eq!(b.check(0.4, 0.5), ConvergenceResult::Converged);
    }

    #[test]
    fn blast_score_blocks_above_threshold() {
        let b = ScoreCheck;
        assert!(matches!(
            b.check(0.9, 0.5),
            ConvergenceResult::Blocked { .. }
        ));
    }

    #[test]
    fn convergence_result_helpers() {
        assert!(ConvergenceResult::Converged.is_converged());
        assert!(!ConvergenceResult::Continue.is_converged());
        assert!(ConvergenceResult::Blocked { reason: "x".into() }.is_blocked());
        assert!(!ConvergenceResult::Converged.is_blocked());
    }

    #[test]
    fn convergence_result_clone_and_eq() {
        let r = ConvergenceResult::Blocked {
            reason: "test".into(),
        };
        assert_eq!(r.clone(), r);
    }
}
