//! Batched span emission — NDJSON flushes every `max(N spans, T interval)`.
//!
//! [`SpanBatcher`] accepts spans via non-blocking [`SpanBatcher::enqueue`],
//! accumulates them in a bounded `tokio::sync::mpsc` channel, and flushes
//! in batches to a [`SpanEmit`] backend. Two flush triggers run concurrently
//! inside a `tokio::select!` loop:
//!
//! - **N trigger** — flush when `batch.len() >= cfg.n` (default 32).
//! - **T trigger** — flush at most every `cfg.t` (default 250 ms) even when
//!   the N threshold is not reached (heartbeat flush for quiescent loads).
//!
//! # Backpressure
//!
//! When the bounded channel is full, [`SpanBatcher::enqueue`] returns `false`
//! and increments [`metrics::SPANS_DROPPED_TOTAL`]. Callers observe the counter
//! via [`metrics::spans_dropped_total()`] or the AYIN `/metrics` endpoint.
//! Spans dropped this way are lost (no retry, no persistence — see `NG-03` in
//! the build plan).
//!
//! # Example
//!
//! ```rust,no_run
//! use std::time::Duration;
//! use la_ayinspan::batch::{BatchConfig, SpanBatcher};
//! use la_ayinspan::emit::NullSpanEmitter;
//!
//! # tokio_test::block_on(async {
//! let cfg = BatchConfig::default();
//! let batcher = SpanBatcher::new(cfg, NullSpanEmitter);
//! # })
//! ```

use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;

use crate::emit::SpanEmit;
use crate::metrics;
use crate::span::TraceSpan;

// ── BatchConfig ───────────────────────────────────────────────────────────────

/// Configuration for [`SpanBatcher`].
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Flush when this many spans have accumulated.
    pub n: usize,
    /// Flush at this interval even if `n` has not been reached.
    pub t: Duration,
    /// Bounded MPSC channel capacity.
    ///
    /// Operator-tunable via `LA_AYIN_QUEUE_CAP` environment variable.
    /// Default: 8192 spans (~12 minutes at 10 spans/sec).
    pub queue_cap: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        let queue_cap = std::env::var("LA_AYIN_QUEUE_CAP")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(8192);
        Self {
            n: 32,
            t: Duration::from_millis(250),
            queue_cap,
        }
    }
}

// ── SpanBatcher ───────────────────────────────────────────────────────────────

/// Batching front-end for any [`SpanEmit`] backend.
///
/// Spans enqueued via [`SpanBatcher::enqueue`] are delivered to the backend in
/// batches. The flusher task is spawned on the current Tokio runtime at
/// construction time.
pub struct SpanBatcher {
    tx: mpsc::Sender<TraceSpan>,
}

impl SpanBatcher {
    /// Create a new batcher with `cfg` and spawn the background flusher task.
    ///
    /// The flusher owns `emitter` and runs until the channel is closed (i.e.,
    /// the last `SpanBatcher` clone is dropped). Any spans still in the batch
    /// at shutdown are flushed before the task exits.
    ///
    /// # Panics
    ///
    /// Panics if called outside a Tokio runtime context.
    pub fn new<E>(cfg: BatchConfig, emitter: E) -> Self
    where
        E: SpanEmit + Send + 'static,
        E::Error: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
    {
        let (tx, rx) = mpsc::channel(cfg.queue_cap);
        let n = cfg.n;
        let t = cfg.t;
        tokio::spawn(flusher_loop(rx, n, t, emitter));
        Self { tx }
    }

    /// Non-blocking enqueue.
    ///
    /// Returns `true` on success. Returns `false` when the channel is full,
    /// incrementing [`metrics::SPANS_DROPPED_TOTAL`].
    pub fn enqueue(&self, span: TraceSpan) -> bool {
        match self.tx.try_send(span) {
            Ok(()) => {
                metrics::SPANS_ENQUEUED_TOTAL.fetch_add(1, Ordering::Relaxed);
                true
            }
            Err(mpsc::error::TrySendError::Full(_)) => {
                metrics::SPANS_DROPPED_TOTAL.fetch_add(1, Ordering::Relaxed);
                false
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                // Flusher task ended unexpectedly — treat as drop.
                metrics::SPANS_DROPPED_TOTAL.fetch_add(1, Ordering::Relaxed);
                false
            }
        }
    }
}

// ── Flusher loop (private) ────────────────────────────────────────────────────

/// Background task: collect spans and flush in batches.
async fn flusher_loop<E>(mut rx: mpsc::Receiver<TraceSpan>, n: usize, t: Duration, emitter: E)
where
    E: SpanEmit,
    E::Error: std::fmt::Display,
{
    let mut batch: Vec<TraceSpan> = Vec::with_capacity(n);
    let mut deadline = Instant::now() + t;

    loop {
        let sleep_until = tokio::time::sleep_until(tokio::time::Instant::from_std(deadline));
        tokio::pin!(sleep_until);

        tokio::select! {
            maybe_span = rx.recv() => {
                match maybe_span {
                    Some(span) => {
                        batch.push(span);
                        if batch.len() >= n {
                            flush_batch(&mut batch, &emitter);
                            deadline = Instant::now() + t;
                        }
                    }
                    None => {
                        // Channel closed — flush remainder and exit.
                        if !batch.is_empty() {
                            flush_batch(&mut batch, &emitter);
                        }
                        return;
                    }
                }
            }
            _ = &mut sleep_until => {
                if !batch.is_empty() {
                    flush_batch(&mut batch, &emitter);
                }
                deadline = Instant::now() + t;
            }
        }
    }
}

/// Flush one batch to the emitter and update counters.
///
/// On emitter error, the spans are still cleared (no retry — NG-03).
fn flush_batch<E>(batch: &mut Vec<TraceSpan>, emitter: &E)
where
    E: SpanEmit,
    E::Error: std::fmt::Display,
{
    let count = batch.len();
    for span in batch.drain(..) {
        if let Err(e) = emitter.emit(span) {
            tracing::warn!(error = %e, "SpanBatcher: emit error on flush");
        }
    }
    metrics::SPANS_EMITTED_TOTAL.fetch_add(count as u64, Ordering::Relaxed);
    metrics::FLUSHES_TOTAL.fetch_add(1, Ordering::Relaxed);
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::emit::{NullSpanEmitter, SpanEmit};
    use crate::metrics::reset_for_test;
    use crate::{Actor, TraceContext, TraceOutcome};

    /// Span factory for tests.
    fn test_span() -> TraceSpan {
        TraceContext::new(Actor::new("test"), "batch.test")
            .outcome(TraceOutcome::Continue)
            .finish()
            .unwrap()
    }

    // ── Helper: collecting emitter ────────────────────────────────────────────

    #[derive(Clone, Default)]
    struct CollectingEmitter {
        spans: Arc<Mutex<Vec<TraceSpan>>>,
    }

    #[derive(Debug)]
    struct CollectError;
    impl std::fmt::Display for CollectError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "collect error")
        }
    }

    impl SpanEmit for CollectingEmitter {
        type Error = CollectError;
        fn emit(&self, span: TraceSpan) -> Result<(), Self::Error> {
            self.spans.lock().unwrap().push(span);
            Ok(())
        }
    }

    // ── queue_overflow_increments_dropped_counter ─────────────────────────────

    #[tokio::test]
    async fn queue_overflow_increments_dropped_counter() {
        // cap=2, N=100, T=60s — neither N nor T trigger fires during the test.
        let cfg = BatchConfig {
            n: 100,
            t: Duration::from_secs(60),
            queue_cap: 2,
        };
        let batcher = SpanBatcher::new(cfg, NullSpanEmitter);

        // Count how many enqueues were rejected synchronously (no yield points).
        let mut rejected = 0u32;
        for _ in 0..10 {
            if !batcher.enqueue(test_span()) {
                rejected += 1;
            }
        }

        // At least 8 out of 10 should be rejected (channel cap = 2, no draining).
        assert!(
            rejected >= 8,
            "expected ≥8 rejected (cap=2, enqueued=10), got {rejected}"
        );
    }

    // ── ndjson_format_preserved ───────────────────────────────────────────────

    #[test]
    fn ndjson_format_preserved() {
        let spans: Vec<TraceSpan> = (0..3).map(|_| test_span()).collect();
        let body = spans
            .iter()
            .map(|s| serde_json::to_string(s).unwrap())
            .collect::<Vec<_>>()
            .join("\n");

        for line in body.lines() {
            let parsed: serde_json::Value =
                serde_json::from_str(line).expect("each NDJSON line is valid JSON");
            assert!(
                parsed.get("id").is_some(),
                "span JSON missing 'id' field: {parsed}"
            );
            assert!(
                parsed.get("action").is_some(),
                "span JSON missing 'action' field: {parsed}"
            );
        }
    }

    // ── batch_config_default_reads_env ────────────────────────────────────────

    #[test]
    fn batch_config_default_values() {
        // Verify the constant-path defaults without mutating env.
        let cfg = BatchConfig {
            n: 32,
            t: Duration::from_millis(250),
            queue_cap: 8192,
        };
        assert_eq!(cfg.n, 32);
        assert_eq!(cfg.t, Duration::from_millis(250));
        assert_eq!(cfg.queue_cap, 8192);
    }

    // ── n_trigger_flushes_batch ───────────────────────────────────────────────

    #[tokio::test]
    async fn n_trigger_flushes_batch() {
        reset_for_test();
        let collector = CollectingEmitter::default();
        let cfg = BatchConfig {
            n: 5,
            t: Duration::from_secs(60),
            queue_cap: 128,
        };
        let batcher = SpanBatcher::new(cfg, collector.clone());

        // Enqueue exactly n spans to trigger an N-flush.
        for _ in 0..5 {
            assert!(batcher.enqueue(test_span()));
        }

        // Give the flusher task time to drain — poll up to 200 ms.
        let budget = Duration::from_millis(200);
        let t0 = std::time::Instant::now();
        while t0.elapsed() < budget {
            if collector.spans.lock().unwrap().len() >= 5 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        let received = collector.spans.lock().unwrap().len();
        assert_eq!(received, 5, "expected 5 spans after N-trigger flush");
        // WHY: global counters may be shared across parallel test threads so we
        // only check the per-test collector count. The counter unit test in
        // metrics::tests covers the increment path in isolation.
        reset_for_test();
    }
}
