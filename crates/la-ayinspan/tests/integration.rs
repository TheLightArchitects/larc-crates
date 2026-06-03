//! Integration tests for the `la-ayinspan` batch module.
//!
//! All tests require the `batch` feature. They are written against the
//! public API so they compile as a separate crate (standard Rust integration
//! test layout).

use std::sync::{Arc, Mutex};
use std::time::Duration;

use la_ayinspan::batch::{BatchConfig, SpanBatcher};
use la_ayinspan::emit::SpanEmit;
use la_ayinspan::{Actor, TraceContext, TraceOutcome, TraceSpan};

// ── Helper types ──────────────────────────────────────────────────────────────

/// Collecting emitter used across all integration tests.
#[derive(Clone, Default)]
struct CollectingEmitter {
    spans: Arc<Mutex<Vec<TraceSpan>>>,
}

#[derive(Debug)]
struct CollectError;

impl std::fmt::Display for CollectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "collect error (should not occur in tests)")
    }
}

impl SpanEmit for CollectingEmitter {
    type Error = CollectError;

    fn emit(&self, span: TraceSpan) -> Result<(), Self::Error> {
        self.spans.lock().expect("lock poisoned").push(span);
        Ok(())
    }
}

impl CollectingEmitter {
    fn count(&self) -> usize {
        self.spans.lock().expect("lock poisoned").len()
    }
}

/// Build a test span with the given action label.
fn make_span(action: &str) -> TraceSpan {
    TraceContext::new(Actor::new("integration-test"), action)
        .outcome(TraceOutcome::Continue)
        .finish()
        .expect("test span build")
}

// ── Test 1: heartbeat flush ───────────────────────────────────────────────────

/// G-BATCH-03 — Heartbeat flush delivers spans within T + scheduling slack.
///
/// Enqueue a single span and assert that it arrives at the emitter within
/// 200 ms (T=100 ms + 100 ms scheduling slack), confirming the T-trigger fires.
#[tokio::test]
async fn heartbeat_flush_delivers_within_latency_budget() {
    let collector = CollectingEmitter::default();
    let cfg = BatchConfig {
        n: 100,                        // high N so N-trigger never fires
        t: Duration::from_millis(100), // fast heartbeat for test speed
        queue_cap: 1024,
    };
    let batcher = SpanBatcher::new(cfg, collector.clone());

    let t0 = std::time::Instant::now();
    assert!(batcher.enqueue(make_span("heartbeat.test")));

    // Poll until the span arrives, up to the latency budget.
    let budget = Duration::from_millis(200);
    let mut delivered = false;
    while t0.elapsed() < budget {
        if collector.count() >= 1 {
            delivered = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    let latency = t0.elapsed();
    assert!(
        delivered,
        "heartbeat flush did not deliver span within {budget:?}; elapsed: {latency:?}"
    );
}

// ── Test 2: backcompat — emit_span API still callable ────────────────────────

/// G-BATCH-04 — Backward-compatibility smoke test.
///
/// Verifies that `SpanBatcher::new` + `enqueue` compile and run with the same
/// `TraceSpan` type used throughout the SDK. If the `TraceSpan` type or the
/// `SpanEmit` trait changed in a breaking way, this test would fail to compile.
#[tokio::test]
async fn backcompat_span_batcher_enqueue_compiles_and_runs() {
    let collector = CollectingEmitter::default();
    let cfg = BatchConfig {
        n: 5,
        t: Duration::from_millis(50),
        queue_cap: 128,
    };
    let batcher = SpanBatcher::new(cfg, collector.clone());

    // This is the back-compat pattern: build a TraceSpan with the builder and
    // enqueue it. The types must remain stable across SDK consumers.
    let span = TraceContext::new(Actor::new("sdk-consumer"), "backcompat.check")
        .outcome(TraceOutcome::Continue)
        .finish()
        .expect("TraceContext build must succeed when outcome is set");

    let enqueued = batcher.enqueue(span);
    assert!(
        enqueued,
        "enqueue must return true when channel has capacity"
    );

    // Wait for heartbeat flush.
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        collector.count(),
        1,
        "span must reach emitter via heartbeat"
    );
}

// ── Test 3: NDJSON format round-trip ─────────────────────────────────────────

/// G-BATCH-05 — NDJSON format: each span serialises to one valid JSON line.
///
/// This is a synchronous test — it verifies the serialisation format without
/// needing the batcher infrastructure. The batch flush joins spans with `\n`.
#[test]
fn ndjson_roundtrip_each_line_is_valid_span_json() {
    let actions: Vec<String> = (0..5).map(|i| format!("action.{i}")).collect();
    let spans: Vec<TraceSpan> = actions
        .iter()
        .map(|a| {
            TraceContext::new(Actor::new("test"), a.as_str())
                .outcome(TraceOutcome::Continue)
                .finish()
                .expect("build")
        })
        .collect();

    // Reproduce the flush serialisation: join with '\n'.
    let body = spans
        .iter()
        .map(|s| serde_json::to_string(s).expect("serialise"))
        .collect::<Vec<_>>()
        .join("\n");

    // Each line must deserialise back to a complete TraceSpan.
    for (i, line) in body.lines().enumerate() {
        let parsed: TraceSpan =
            serde_json::from_str(line).unwrap_or_else(|e| panic!("line {i} failed: {e}"));
        assert_eq!(
            parsed.action,
            format!("action.{i}"),
            "action field must round-trip correctly"
        );
    }
}

// ── Test 4: backpressure — queue overflow drops spans ────────────────────────

/// G-BATCH-02 — Overflow increments `spans_dropped_total` and never panics.
///
/// Uses a tiny channel (cap=2) and floods it without giving the flusher time
/// to drain. Asserts that at least 8 out of 10 enqueues were dropped and that
/// `spans_dropped_total` reflects the drops.
#[tokio::test]
async fn backpressure_overflow_increments_dropped_counter() {
    // Reset counters before this test to get a clean baseline.
    la_ayinspan::metrics::reset_for_test();

    let collector = CollectingEmitter::default();
    let cfg = BatchConfig {
        n: 1000,                    // N-trigger won't fire
        t: Duration::from_secs(60), // T-trigger won't fire
        queue_cap: 2,               // tiny channel
    };
    let batcher = SpanBatcher::new(cfg, collector);
    let initial = la_ayinspan::metrics::spans_dropped_total();

    // Enqueue synchronously — no yield points so the flusher can't drain.
    for _ in 0..10 {
        batcher.enqueue(make_span("overflow.test"));
    }

    let dropped = la_ayinspan::metrics::spans_dropped_total() - initial;
    assert!(
        dropped >= 8,
        "expected ≥8 drops for cap=2 / enqueued=10, got {dropped}"
    );

    la_ayinspan::metrics::reset_for_test();
}
