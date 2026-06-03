//! Atomic counters for span batching observability.
//!
//! All counters are monotonically increasing and use [`Ordering::Relaxed`]
//! because they are independent accumulators — no happens-before relationship
//! is required between them.
//!
//! Read them via the getter functions for a consistent [`MetricsSnapshot`].

use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

/// Total spans successfully enqueued to the batch channel.
pub static SPANS_ENQUEUED_TOTAL: AtomicU64 = AtomicU64::new(0);

/// Total spans delivered to the backend (emitted on flush).
pub static SPANS_EMITTED_TOTAL: AtomicU64 = AtomicU64::new(0);

/// Total spans dropped because the channel was full.
pub static SPANS_DROPPED_TOTAL: AtomicU64 = AtomicU64::new(0);

/// Total flush operations executed by the background flusher.
pub static FLUSHES_TOTAL: AtomicU64 = AtomicU64::new(0);

/// Spans successfully enqueued.
#[must_use]
pub fn spans_enqueued_total() -> u64 {
    SPANS_ENQUEUED_TOTAL.load(Ordering::Relaxed)
}

/// Spans delivered to the backend.
#[must_use]
pub fn spans_emitted_total() -> u64 {
    SPANS_EMITTED_TOTAL.load(Ordering::Relaxed)
}

/// Spans dropped due to a full queue.
#[must_use]
pub fn spans_dropped_total() -> u64 {
    SPANS_DROPPED_TOTAL.load(Ordering::Relaxed)
}

/// Flush operations executed.
#[must_use]
pub fn flushes_total() -> u64 {
    FLUSHES_TOTAL.load(Ordering::Relaxed)
}

/// Point-in-time snapshot of all four counters.
#[derive(Debug, Clone, Serialize)]
pub struct MetricsSnapshot {
    /// Total spans enqueued.
    pub spans_enqueued: u64,
    /// Total spans emitted to backend.
    pub spans_emitted: u64,
    /// Total spans dropped on full queue.
    pub spans_dropped: u64,
    /// Total flush operations.
    pub flushes: u64,
}

/// Capture a consistent snapshot of all counters.
///
/// Because each counter is read independently with `Relaxed` ordering,
/// the snapshot is not strictly atomic — it is a best-effort point-in-time
/// view suitable for dashboards and health checks.
#[must_use]
pub fn snapshot() -> MetricsSnapshot {
    MetricsSnapshot {
        spans_enqueued: SPANS_ENQUEUED_TOTAL.load(Ordering::Relaxed),
        spans_emitted: SPANS_EMITTED_TOTAL.load(Ordering::Relaxed),
        spans_dropped: SPANS_DROPPED_TOTAL.load(Ordering::Relaxed),
        flushes: FLUSHES_TOTAL.load(Ordering::Relaxed),
    }
}

/// Reset all counters to zero.
///
/// Intended for use in tests only — not safe to call from production code
/// because it introduces a global-state mutation that races with concurrent
/// enqueue/emit operations.
///
/// Available when the `test-utils` feature is enabled (integration test crates)
/// or during `#[cfg(test)]` compilation (unit tests).
#[cfg(any(test, feature = "test-utils"))]
pub fn reset_for_test() {
    SPANS_ENQUEUED_TOTAL.store(0, Ordering::Relaxed);
    SPANS_EMITTED_TOTAL.store(0, Ordering::Relaxed);
    SPANS_DROPPED_TOTAL.store(0, Ordering::Relaxed);
    FLUSHES_TOTAL.store(0, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_reflects_counter_state() {
        reset_for_test();
        SPANS_ENQUEUED_TOTAL.fetch_add(5, Ordering::Relaxed);
        SPANS_EMITTED_TOTAL.fetch_add(3, Ordering::Relaxed);
        SPANS_DROPPED_TOTAL.fetch_add(2, Ordering::Relaxed);
        FLUSHES_TOTAL.fetch_add(1, Ordering::Relaxed);
        let s = snapshot();
        assert_eq!(s.spans_enqueued, 5);
        assert_eq!(s.spans_emitted, 3);
        assert_eq!(s.spans_dropped, 2);
        assert_eq!(s.flushes, 1);
        reset_for_test();
    }

    #[test]
    fn snapshot_serializes_to_json() {
        reset_for_test();
        let s = snapshot();
        let json = serde_json::to_string(&s).expect("serialization");
        assert!(json.contains("spans_dropped"));
        assert!(json.contains("flushes"));
        reset_for_test();
    }
}
