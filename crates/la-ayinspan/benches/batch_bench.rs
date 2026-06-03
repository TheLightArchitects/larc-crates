//! Criterion benchmarks for `la-ayinspan` batched span emission.
//!
//! # G-BATCH-01 — `bench_emit_burst_1000`
//!
//! Enqueues 1 000 spans into a [`SpanBatcher`] backed by a [`NullSpanEmitter`]
//! and waits for the batcher to drain. Total wall-clock must be ≤ 2 s.
//!
//! # `bench_emit_sequential_100`
//!
//! Enqueues 100 spans with a 1 ms sleep between each, simulating a quiescent
//! strategy loop. Verifies that heartbeat flushes don't accumulate unbounded
//! latency under sparse loads.

use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use la_ayinspan::batch::{BatchConfig, SpanBatcher};
use la_ayinspan::emit::NullSpanEmitter;
use la_ayinspan::{Actor, TraceContext, TraceOutcome};

/// Build a benchmark span.
fn bench_span() -> la_ayinspan::span::TraceSpan {
    TraceContext::new(Actor::new("bench"), "bench.emit")
        .outcome(TraceOutcome::Continue)
        .finish()
        .expect("bench span build")
}

// ── bench_emit_burst_1000 ─────────────────────────────────────────────────────

/// G-BATCH-01: enqueue 1 000 spans and drain within 2 s.
///
/// Uses `NullSpanEmitter` to eliminate disk/network I/O from the measurement.
/// The metric under test is the channel enqueue + flusher dispatch overhead.
fn bench_emit_burst_1000(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");

    c.bench_function("emit_burst_1000", |b| {
        b.iter(|| {
            rt.block_on(async {
                let cfg = BatchConfig {
                    n: 32,
                    t: Duration::from_millis(250),
                    queue_cap: 8192,
                };
                let batcher = SpanBatcher::new(cfg, NullSpanEmitter);

                for _ in 0..1000 {
                    batcher.enqueue(bench_span());
                }

                // Wait for all batches to flush.
                // At N=32, 1000 spans → 32 full batches (31 × N-trigger) + 1 heartbeat.
                // 250 ms heartbeat + 50 ms slack = 300 ms total.
                tokio::time::sleep(Duration::from_millis(350)).await;
            });
        });
    });
}

// ── bench_emit_sequential_100 ─────────────────────────────────────────────────

/// Sequential 100-span workload with 1 ms inter-span delay.
///
/// Models a strategy loop that emits one span per step at roughly 10 spans/sec.
/// Measures cumulative enqueue overhead under sparse load.
fn bench_emit_sequential_100(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");

    c.bench_function("emit_sequential_100", |b| {
        b.iter(|| {
            rt.block_on(async {
                let cfg = BatchConfig {
                    n: 32,
                    t: Duration::from_millis(250),
                    queue_cap: 8192,
                };
                let batcher = SpanBatcher::new(cfg, NullSpanEmitter);

                for _ in 0..100 {
                    batcher.enqueue(bench_span());
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }

                // Flush remaining partial batch.
                tokio::time::sleep(Duration::from_millis(300)).await;
            });
        });
    });
}

criterion_group!(benches, bench_emit_burst_1000, bench_emit_sequential_100);
criterion_main!(benches);
