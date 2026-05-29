# la-crates

Public Rust crates from [The Light Architects](https://github.com/TheLightArchitect).

## Crates

| Crate | Description | Feature Gates |
|-------|-------------|---------------|
| [`la-soulstrand`](./crates/la-soulstrand/) | Knowledge graph + retrieval library (96.2% Recall@5) | `sqlite` (default), `helix` (Neo4j) |
| [`la-benchmark`](./crates/la-benchmark/) | Domain-agnostic performance benchmark framework | `longmemeval` |
| [`la-mcp`](./crates/la-mcp/) | MCP server framework with multi-transport and squad routing | `stdio` (default), `http`, `websocket`, `lightsquad` |

## Quick start

```bash
# Tier 1: SQLite only (no Neo4j needed)
cargo add la-soulstrand

# Tier 2: Full Neo4j + 4-signal RRF
cargo add la-soulstrand --features helix

# Benchmark framework with LongMemEval
cargo add la-benchmark --features longmemeval

# MCP server with all transports
cargo add la-mcp --features stdio,http,websocket
```

## Architecture

All public crates follow the **self-contained** pattern (Option 1): source is public, moat is benchmark result + ecosystem, not hidden code. The private `lightarchitects` SDK holds the secret sauce and is consumed via git dependency.

## License

Apache-2.0