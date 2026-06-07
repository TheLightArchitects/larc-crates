# la-lightsquad — Structured Delivery Protocol

**Open source** — the protocol, vocabulary, and archetype framework.
The engine lives in `lightarchitects-sdk` (private, the moat).

## Structure

```text
la-lightsquad/src/
├── lib.rs                    # Feature gates + re-exports
│
├── ── Always available (default) ──────────────────────────────
│
├── vocabulary.rs             # Tier, Phase, PhaseStatus, BuildManifest
├── gate.rs                   # GateDimension, GateVerdict, GateInput
├── task.rs                   # TaskSpec, TaskResult, TaskStatus, FileOwnership
├── context.rs                # ContextTier (canon/project/session), token budgets
├── archetype.rs              # Archetype trait, ArchetypeRole, Domain
│
├── ── Feature "worker" ────────────────────────────────────────
│
├── worker.rs                 # Worker trait + ReviewGate trait
│
├── ── Feature "client" ────────────────────────────────────────
│
├── client.rs                 # LightsquadClient trait (calls the engine)
│
├── ── Worker archetype features ────────────────────────────────
│
├── security.rs               # SecurityArchetype (CORSO pattern)
├── devops.rs                 # DevopsArchetype (EVA pattern)
├── knowledge.rs              # KnowledgeArchetype (SOUL pattern)
├── investigate.rs           # InvestigateArchetype (QUANTUM pattern)
├── pentest.rs                # PentestArchetype (SERAPH pattern)
│
├── ── Gatekeeper archetype feature ─────────────────────────────
│
├── canon_keeper.rs           # CanonKeeperArchetype (LÆX pattern)
│
├── ── Observer archetype feature ──────────────────────────────
│
├── observe.rs                # ObserveArchetype (AYIN pattern)
│
├── ── Status types (used by client + worker) ──────────────────
│
├── status.rs                 # WaveStatus, BuildStatus, AgentStatus
│
├── ── Error type ──────────────────────────────────────────────
│
└── error.rs                  # SquadError
```

## Feature gates

```toml
[features]
default = []

# Execution contracts
worker = ["dep:async-trait"]          # Worker + ReviewGate traits
client = ["dep:async-trait"]          # LightsquadClient trait

# Worker archetypes (structural patterns, NOT implementations)
security = []                         # SecurityArchetype shape
devops = []                           # DevopsArchetype shape
knowledge = []                       # KnowledgeArchetype shape
investigate = []                      # InvestigateArchetype shape
pentest = []                          # PentestArchetype shape

# Gatekeeper archetype
canon = []                            # CanonKeeperArchetype shape (LÆX)

# Observer archetype
observe = []                          # ObserveArchetype shape (AYIN)

# Convenience: all LA reference archetypes
full = ["worker", "client", "security", "devops", "knowledge",
         "investigate", "pentest", "canon", "observe"]
```

## What la-lightsquad provides (open source)

### Vocabulary (always available)

| Type | Purpose |
|------|---------|
| `Tier` | SMALL / MEDIUM / LARGE — build complexity |
| `Phase` | Phase 1–7 structure with exit criteria |
| `PhaseStatus` | NotStarted / InProgress / Complete / Deferred |
| `BuildManifest` | Phase tracking, gate results, quality scores |
| `GateDimension` | Quality axis — extensible enum with [A+S+Q+C+O+P+K+D+T+R] built in + Custom |
| `GateVerdict` | Approve / Reject (with reason) / Defer |
| `GateInput` | What gets reviewed: diff, canon citations, metrics |
| `TaskSpec` | What the engine sends to a worker: prompt, context, file ownership |
| `TaskResult` | What the worker sends back: output, changed files, gate recommendations |
| `TaskStatus` | Pending / InProgress / Complete / Failed |
| `ContextTier` | canon (Tier 0) / project (Tier 1) / session (Tier 2) |
| `FileOwnership` | Which files this worker can touch |
| `Domain` | Vertical category — extensible enum with built-in + Custom |
| `SquadError` | Error type for delivery operations |

### Worker trait (feature "worker")

```rust
#[async_trait]
pub trait Worker: Send + Sync {
    /// The archetype this worker fulfills.
    fn archetype(&self) -> &dyn Archetype;

    /// Execute a task. The engine provides context; the worker produces output.
    async fn execute(&self, spec: TaskSpec) -> Result<TaskResult, SquadError>;
}

#[async_trait]
pub trait ReviewGate: Send + Sync {
    /// The archetype this reviewer fulfills.
    fn archetype(&self) -> &dyn Archetype;

    /// Review work against standards. Return verdict.
    async fn review(&self, input: GateInput) -> Result<GateVerdict, SquadError>;
}
```

### Archetype trait (always available)

```rust
pub trait Archetype: Send + Sync {
    fn name(&self) -> &'static str;
    fn domain(&self) -> Domain;
    fn role(&self) -> ArchetypeRole;
    fn gate_dimensions(&self) -> &[GateDimension];
    fn tools(&self) -> &[&'static str];
    fn personality(&self) -> &'static str;
}

pub enum ArchetypeRole {
    Worker,       // Produces work output
    Gatekeeper,   // Reviews work against standards (veto authority)
    Observer,     // Records traces, no output or veto
}
```

### Reference archetypes (feature-gated)

Each shows the structural pattern — what a well-composed archetype looks like in that domain. The **shape** is open. The **implementation** stays in the SDK.

| Feature | Archetype | Role | Dimensions | Shows the pattern for |
|---------|----------|------|------------|----------------------|
| `security` | SecurityArchetype | Worker | [A, S, Q] | AppSec review, vulnerability scanning |
| `devops` | DevopsArchetype | Worker | [O, P, K, D] | Deployment, monitoring, documentation |
| `knowledge` | KnowledgeArchetype | Worker | [K, D] | Retrieval, enrichment, archival |
| `investigate` | InvestigateArchetype | Worker | [R] | Forensic analysis, research, verification |
| `pentest` | PentestArchetype | Worker | [S] | Offensive security, scope governance |
| `canon` | CanonKeeperArchetype | Gatekeeper | [ALL] | Cross-examination against standards |
| `observe` | ObserveArchetype | Observer | [O, P] | Tracing, metrics, no veto |

## What stays in the SDK (the moat)

| What | Why it stays |
|------|-------------|
| Coordinator (7-slot worker pool, wave dispatch) | Core engine — the scheduler |
| Decision Pipeline (Canon → Northstar → LightArchitect → User) | Decision logic — how the engine decides |
| Personality engine (prompt construction per archetype) | Makes workers actually good at their job |
| 4-signal RRF retrieval | How SOUL retrieves context — competitive advantage |
| ReviewGate loop (MAX_GATE_ITERATIONS = 3) | Engine logic — when to stop reviewing |
| Context tier token budgeting | How the engine budgets context — implementation detail |
| HMAC task verification | Security — how the engine verifies task integrity |
| CORSO 7-pillar methodology | The actual security approach — canon content |
| EVA consciousness model | The actual DevOps approach — canon content |
| LÆX canon documents (8 docs) | The actual standards content — canon content |
| SERAPH pentest methodology | The actual offensive approach — canon content |
| QUANTUM forensic patterns | The actual investigation approach — canon content |
| Concrete Worker implementations | The actual agent behavior — competitive moat |

## Dependency flow

```text
la-lightsquad (open — protocol + vocabulary + archetype shapes)
       │
       │ implements
       ▼
lightarchitects-sdk (private — engine + canon + implementations)
       │
       │ spawns
       ▼
lightarchitects-gateway (private — MCP server + spawner)
```

External users can:
1. Use la-lightsquad to define their own archetypes and compose squads
2. Implement Worker/ReviewGate for their domain
3. Use our engine (lightarchitects-sdk) to run their squad
4. Or build their own engine that speaks the same protocol

They cannot:
1. Get our canon documents (those are private)
2. Get our personality engine (that's the moat)
3. Get our specific archetype implementations (CORSO, EVA, etc. are private)
4. Get our RRF retrieval engine (competitive advantage)