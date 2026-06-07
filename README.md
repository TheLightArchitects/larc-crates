# la-crates

Public facade crates from [The Light Architects](https://github.com/TheLightArchitects).

## Architecture

All public crates follow the **facade pattern**: traits and types only, no implementations. The private `lightarchitects-sdk` holds the hidden business logic (the moat). Each crate has one clear purpose:

```text
la-soulvault      ← swappable backends (HelixBackend + data types) + optional production vault
la-benchmark      ← benchmark trait + metrics types
la-gateway        ← plug adapter (how you connect to the ecosystem)
la-ayinspan       ← observability span types (zero deps)
```

```text
┌──────────────────────────────────────────────────────────┐
│                   PUBLIC CRATES (facade)                 │
│  la-soulvault   la-benchmark  la-gateway  la-ayinspan   │
│       traits + types only, no implementations            │
│       (la-soulvault `vault` feature = production backend)│
└──────────────────────┬───────────────────────────────────┘
                       │ implements (vault feature)
┌──────────────────────▼───────────────────────────────────┐
│              lightarchitects-sdk (private)                │
│     4-signal RRF fusion · adaptive weights · personality  │
│              MCP transport · sibling dispatch             │
└──────────────────────────────────────────────────────────┘
```

## Crates

| Crate | Purpose | Feature Gates |
|-------|---------|---------------|
| [`la-soulvault`](./crates/la-soulvault/) | Knowledge graph backend traits + data types + optional production vault | `embedding`, `graph`, `promotion`, `engine`, `embedding-cache`, `vault` |
| [`la-benchmark`](./crates/la-benchmark/) | Benchmark trait + metrics types | `longmemeval` |
| [`la-gateway`](./crates/la-gateway/) | Gateway interface — transport, MCP handlers, protocol types | `mcp`, `squad`, `ayin` |
| [`la-ayinspan`](./crates/la-ayinspan/) | Observability span types — TraceSpan, Actor, W3C traceparent | (none) |

## Key design decisions

- **la-gateway** is the plug adapter — Transport, SiblingHandler, Config, JSON-RPC types. Typed sibling clients (SoulClient, CorsoClient) live in the SDK, not here.
- **la-soulvault** has two layers: (1) open-source traits + types (`engine` feature) — implement your own backend; (2) production vault backend (`vault` feature) — wraps the private SDK via `vault::connect()`.
- **HandlerRegistry** lives in the SDK, not la-gateway. The facade only defines `SiblingHandler` (the trait you implement).
- **One owner per type** — SiblingId lives in la-gateway (the canonical source), re-exported by the SDK.
- **Config** describes gateway connection (endpoint, auth, timeout), not binary spawning.

## Usage

```toml
[dependencies]
# SOUL knowledge graph traits — implement your own backend
la-soulvault = { git = "...", features = ["engine"] }

# Production vault backend (requires SDK access)
la-soulvault = { git = "...", features = ["vault"] }
lightarchitects = { path = "...", optional = false }

# Benchmark traits
la-benchmark = { git = "...", features = ["longmemeval"] }

# Gateway interface (how you plug in)
la-gateway = { git = "...", features = ["mcp", "squad"] }

# Observability span types (zero deps)
la-ayinspan = { git = "..." }
```

### Production vault quick start

```rust,ignore
// From environment variables (SOUL_VAULT_URI / SOUL_VAULT_USER / SOUL_VAULT_PASSWORD):
let client = la_soulvault::vault::connect_from_env().await?;

// From explicit config:
use la_soulvault::vault::VaultConfig;
let client = la_soulvault::vault::connect(VaultConfig::new(
    "bolt://localhost:7687", "neo4j", "password",
)).await?;

// Use the client:
client.upsert_step(step).await?;
let results = client.retrieve_adaptive("helix-id", "my query", 10).await?;
```

## License

Apache-2.0