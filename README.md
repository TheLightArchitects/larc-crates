# la-crates

Public facade crates from [The Light Architects](https://github.com/TheLightArchitect).

## Architecture

All public crates follow the **facade pattern**: traits and types only, no implementations. The private `lightarchitects-sdk` holds the hidden business logic (the moat). Each crate has one clear purpose:

```text
la-soulstrand     ← swappable backends (HelixBackend + data types)
la-benchmark       ← benchmark trait + metrics types
la-gateway         ← plug adapter (how you connect to the ecosystem)
la-ayinspan        ← observability span types (zero deps)
```

```text
┌──────────────────────────────────────────────────────────┐
│                   PUBLIC CRATES (facade)                 │
│  la-soulstrand  la-benchmark  la-gateway  la-ayinspan   │
│       traits + types only, no implementations            │
└──────────────────────┬───────────────────────────────────┘
                       │ implements
┌──────────────────────▼───────────────────────────────────┐
│              lightarchitects-sdk (private)                │
│     4-signal convex fusion · adaptive weights · personality   │
│              MCP transport · sibling dispatch             │
└──────────────────────────────────────────────────────────┘
```

## Crates

| Crate | Purpose | Feature Gates |
|-------|---------|---------------|
| [`la-soulstrand`](./crates/la-soulstrand/) | Knowledge graph backend traits + data types | `embedding`, `graph`, `promotion`, `helix` |
| [`la-benchmark`](./crates/la-benchmark/) | Benchmark trait + metrics types | `longmemeval` |
| [`la-gateway`](./crates/la-gateway/) | Gateway interface — transport, MCP handlers, protocol types | `mcp`, `squad`, `ayin` |
| [`la-ayinspan`](./crates/la-ayinspan/) | Observability span types — TraceSpan, Actor, W3C traceparent | (none) |

## Key design decisions

- **la-gateway** is the plug adapter — Transport, SiblingHandler, Config, JSON-RPC types. Typed sibling clients (SoulClient, CorsoClient) live in the SDK, not here.
- **la-soulstrand** is the swappable backend crate — `HelixBackend` is genuinely implementable by third parties. The other sibling traits aren't.
- **HandlerRegistry** lives in the SDK, not la-gateway. The facade only defines `SiblingHandler` (the trait you implement).
- **One owner per type** — SiblingId lives in la-gateway (the canonical source), re-exported by the SDK.
- **Config** describes gateway connection (endpoint, auth, timeout), not binary spawning.

## Usage

```toml
[dependencies]
# SOUL knowledge graph traits
la-soulstrand = { git = "...", features = ["helix"] }

# Benchmark traits
la-benchmark = { git = "...", features = ["longmemeval"] }

# Gateway interface (how you plug in)
la-gateway = { git = "...", features = ["mcp", "squad"] }

# Observability span types (zero deps)
la-ayinspan = { git = "..." }

# Production implementation (the moat)
lightarchitects = { git = "...", features = ["soul"] }
```

## License

Apache-2.0