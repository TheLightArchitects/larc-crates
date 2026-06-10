# larc-crates

**Light ARChitects Rust Crates** — public type vocabulary and protocol traits for building agentic multi-agent systems.

Six crates, zero business logic. Bring your own engine.

## Crates

| Crate | Purpose | Feature Gates |
|-------|---------|---------------|
| [`larc-ayinspan`](./crates/larc-ayinspan/) | Observability span types — `TraceSpan`, `Actor`, W3C traceparent propagation | `batch`, `test-utils` |
| [`larc-loops`](./crates/larc-loops/) | Agentic loop convergence traits — `BlastScore`, `ConvergenceGate`, `NPassVerifier`, `QueueDrain`, `InterestDecay`, `IntervalWatch` | (none) |
| [`larc-benchmark`](./crates/larc-benchmark/) | Domain-agnostic benchmark framework for knowledge retrieval | `longmemeval` |
| [`larc-soulvault`](./crates/larc-soulvault/) | Knowledge graph backend traits + data types (`Step`, `Helix`, `Strand`, `HelixLink`, `ResearchReport`) | `embedding`, `graph`, `promotion`, `engine`, `embedding-cache` |
| [`larc-lightsquad`](./crates/larc-lightsquad/) | Structured Delivery Protocol — `Archetype`, gate dimensions, `ContextVector`, `CriticReview`, `EvidenceBundle`, `SanitizedTrace`, `TestAssertions`, async executor traits | `dispatch`, `full` |
| [`larc-gateway`](./crates/larc-gateway/) | Gateway interface — transport abstraction, MCP handler traits, JSON-RPC protocol types | `mcp`, `ayin` |

## Architecture

All crates follow the **facade pattern**: traits and types only, no implementations. Implement the traits with whatever engine, model, or transport fits your stack.

```text
┌──────────────────────────────────────────────────────────┐
│              PUBLIC CRATES (this repo)                   │
│   larc-ayinspan   larc-loops      larc-benchmark         │
│   larc-soulvault  larc-lightsquad larc-gateway           │
│       traits + types only, no implementations            │
└──────────────────────────────────────────────────────────┘
```

## Quick start

```toml
[dependencies]
# Observability span types (zero deps)
larc-ayinspan = "0.1"

# Agentic loop convergence traits
larc-loops = "0.1"

# Knowledge graph backend traits — implement your own
larc-soulvault = { version = "0.1", features = ["engine"] }

# Structured delivery protocol — archetype framework + pipeline contract types
larc-lightsquad = { version = "0.1", features = ["dispatch"] }

# Gateway interface (transport + MCP handler traits)
larc-gateway = { version = "0.1", features = ["mcp"] }

# Benchmark traits + metrics
larc-benchmark = { version = "0.1", features = ["longmemeval"] }
```

## Design principles

- **One owner per type** — each shared type has a single canonical source.
- **Compile-time decoupling** — no async runtime unless a feature is enabled.
- **Non-exhaustive structs and enums** — additive evolution, no breaking changes from new fields.
- **Structural validation** — invariants enforced at construction *and* deserialization (e.g., [`TestAssertions`] non-empty guarantee).

## License

Apache-2.0
