//! Pipeline contract types — Context Vector, constraint manifest, and safe feedback wire.
//!
//! Formalises the three-layer context assembly protocol (Builders Cookbook §66.5)
//! and the structural sanitization gate (Security Guardrails §3.4.1) as
//! machine-readable, serde-compatible types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

// ── Layer A: Environment Manifest ─────────────────────────────────────────────

/// Locked execution environment — Layer A of the [`ContextVector`].
///
/// Specifies the runtime, edition, and exact crate versions the Coder agent
/// must target. Prevents version-drift hallucinations by providing a hard
/// API-cutoff rule alongside the lockfile data.
///
/// Corresponds to Builders Cookbook §66.5 Layer A.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EnvironmentManifest {
    /// Runtime identifier, e.g. `"Rust 1.87.0"`.
    pub runtime: String,
    /// Rust edition, e.g. `"2024"`.
    pub edition: String,
    /// Active Cargo features for this build, e.g. `["tokio/full", "serde/derive"]`.
    #[serde(default)]
    pub active_features: Vec<String>,
    /// Exact locked crate versions (`crate_name` → `"x.y.z"`).
    ///
    /// The Coder agent MUST NOT use APIs introduced after these versions.
    /// The orchestrator populates this from the workspace `Cargo.lock`.
    #[serde(default)]
    pub strict_lockfile: HashMap<String, String>,
}

impl EnvironmentManifest {
    /// Create a new environment manifest with the minimum required fields.
    #[must_use]
    pub fn new(runtime: String, edition: String) -> Self {
        Self {
            runtime,
            edition,
            active_features: Vec::new(),
            strict_lockfile: HashMap::new(),
        }
    }
}

// ── Layer B: Interface Stubs ──────────────────────────────────────────────────

/// A single interface stub — struct layouts and `fn` signatures, zero bodies.
///
/// Extracted via LSP or Tree-sitter. Provides the Coder agent with the exact
/// public contract it must satisfy without the implementation details that
/// pollute attention weights.
///
/// Corresponds to Builders Cookbook §66.5 Layer B.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InterfaceStub {
    /// Relative path to the source file within the worktree.
    pub source_path: String,
    /// Struct layouts, trait declarations, and `fn` signatures only.
    ///
    /// Must contain zero implementation bodies — `{...}` blocks are replaced
    /// with `;` for function stubs or left empty `{}` for trait stubs.
    /// This invariant is the assembler's responsibility; this type does not
    /// validate it.
    pub signature_text: String,
}

impl InterfaceStub {
    /// Create a new interface stub.
    #[must_use]
    pub fn new(source_path: String, signature_text: String) -> Self {
        Self {
            source_path,
            signature_text,
        }
    }
}

// ── Layer C: Constraint Manifest ──────────────────────────────────────────────

/// Machine-readable hard constraints — Layer C of the [`ContextVector`].
///
/// Boolean fields map directly to Builders Cookbook §3 obligations. Using a
/// structured type (rather than prose) makes the constraints diff-able,
/// auditable, and testable.
///
/// Corresponds to Builders Cookbook §66.5 Layer C.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ConstraintManifest {
    /// Forbid heap allocation in inner loops (§3: stack-primitive preference).
    #[serde(default)]
    pub no_heap_alloc_inner_loops: bool,
    /// Forbid `.unwrap()` calls (§3: no unwrap in production).
    #[serde(default)]
    pub no_unwrap: bool,
    /// Forbid `.expect()` calls (§3: no expect in production).
    #[serde(default)]
    pub no_expect: bool,
    /// Require `Send + Sync` bounds on all shared state (§3: thread-safety).
    #[serde(default)]
    pub send_sync_required: bool,
    /// Forbid importing crates absent from `EnvironmentManifest::strict_lockfile`.
    #[serde(default)]
    pub no_unlisted_deps: bool,
    /// Additional absolute constraints as free-form prose.
    ///
    /// Use sparingly — prefer the boolean fields for machine-verifiable rules.
    #[serde(default)]
    pub custom: Vec<String>,
}

impl ConstraintManifest {
    /// All boolean flags disabled.
    ///
    /// Enable specific constraints via field mutation (struct expression syntax
    /// with `..` is unavailable outside the crate because of `#[non_exhaustive]`):
    ///
    /// ```rust
    /// use larc_lightsquad::ConstraintManifest;
    ///
    /// let mut m = ConstraintManifest::default_off();
    /// m.no_unwrap = true;
    /// m.no_expect = true;
    /// assert!(m.no_unwrap);
    /// assert!(!m.send_sync_required);
    /// ```
    #[must_use]
    pub fn default_off() -> Self {
        Self {
            no_heap_alloc_inner_loops: false,
            no_unwrap: false,
            no_expect: false,
            send_sync_required: false,
            no_unlisted_deps: false,
            custom: Vec::new(),
        }
    }

    /// All Cookbook §3 Rust obligations enabled.
    ///
    /// `no_heap_alloc_inner_loops` is left `false` — it is context-dependent
    /// and must be opted into by the caller.
    #[must_use]
    pub fn rust_canon() -> Self {
        Self {
            no_heap_alloc_inner_loops: false,
            no_unwrap: true,
            no_expect: true,
            send_sync_required: true,
            no_unlisted_deps: true,
            custom: Vec::new(),
        }
    }
}

// ── Context Vector ────────────────────────────────────────────────────────────

/// Assembled three-layer context passed to a pipeline agent at spawn.
///
/// The orchestrator is the **sole assembler** of a `ContextVector`
/// (Builders Cookbook §66.5 — extends §66.3). A sealed `ContextVector`
/// arrives at the agent spawn boundary; agents receive it, they do not
/// build it. An agent constructing its own context violates §66.3.
///
/// | Layer | Field | §66 Tier | Content |
/// |-------|-------|----------|---------|
/// | A | [`env`](ContextVector::env) | Tier 1 (stable) | Locked runtime + versions |
/// | B | [`stubs`](ContextVector::stubs) | Tier 1 (dynamic) | Interface signatures |
/// | C | [`constraints`](ContextVector::constraints) | Tier 1 (stable) | §3 hard bounds |
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ContextVector {
    /// Layer A — locked execution environment.
    pub env: EnvironmentManifest,
    /// Layer B — interface stubs (signatures only, zero bodies).
    #[serde(default)]
    pub stubs: Vec<InterfaceStub>,
    /// Layer C — hard constraint manifest mapping to Cookbook §3 obligations.
    pub constraints: ConstraintManifest,
}

impl ContextVector {
    /// Create a new context vector.
    #[must_use]
    pub fn new(
        env: EnvironmentManifest,
        stubs: Vec<InterfaceStub>,
        constraints: ConstraintManifest,
    ) -> Self {
        Self {
            env,
            stubs,
            constraints,
        }
    }
}

// ── Safe Feedback Wire ────────────────────────────────────────────────────────

/// A single allowlist-parsed compiler or test-runner diagnostic.
///
/// The only safe unit of feedback from the Tester agent to the Coder agent.
/// All fields are produced by a structural allowlist parser — freeform strings,
/// ANSI escape sequences, and role-override tokens are stripped before this
/// type is populated.
///
/// Per Security Guardrails §3.4.1 (OWASP LLM01, CWE-77): `"cleaned"` is a
/// parsing heuristic, not a security property. Structural parsing is.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CompilerDiagnostic {
    /// Relative file path where the diagnostic originates.
    pub file: String,
    /// Line number (1-indexed).
    pub line: u32,
    /// Rust error code, if applicable (e.g. `"E0308"`).
    /// `None` for warnings or test failures without a compiler error code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// Diagnostic message, stripped of ANSI escape sequences and truncated
    /// at 512 bytes. Must not contain raw stack frame addresses.
    pub message: String,
}

impl CompilerDiagnostic {
    /// Create a diagnostic without an error code.
    #[must_use]
    pub fn new(file: String, line: u32, message: String) -> Self {
        Self {
            file,
            line,
            error_code: None,
            message,
        }
    }

    /// Create a diagnostic with a Rust error code.
    #[must_use]
    pub fn with_error_code(file: String, line: u32, error_code: String, message: String) -> Self {
        Self {
            file,
            line,
            error_code: Some(error_code),
            message,
        }
    }
}

/// Sanitized Tester-to-Coder feedback — the only safe feedback wire between
/// the Tester and Coder agents.
///
/// All compiler and test-runner output MUST be parsed through a structural
/// allowlist before producing a `SanitizedTrace`. Raw strings must not be
/// injected into the Coder agent's prompt (Security Guardrails §3.4.1).
///
/// # Constructing safely
///
/// Use [`SanitizedTrace::from_cargo_json`] to parse `cargo --message-format=json`
/// NDJSON output. The parser applies the §3.4.1 allowlist — only fields in
/// `{file, line, error_code, message ≤512 bytes}` survive. Malformed lines are
/// silently dropped; only `SanitizeError::EmptyInput` is a hard error.
///
/// `SanitizedTrace::new()` and `SanitizedTrace::with_diagnostics()` are provided
/// for callers who have already parsed their output — they do not re-validate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SanitizedTrace {
    /// Allowlist-parsed diagnostics — one entry per compiler error or test failure.
    #[serde(default)]
    pub diagnostics: Vec<CompilerDiagnostic>,
    /// Zero-indexed correction iteration (0..=2 for the §64.3 ceiling of 3).
    ///
    /// Allows the Coder to calibrate correction aggression by iteration.
    pub loop_index: u8,
}

impl SanitizedTrace {
    /// Create an empty sanitized trace for the given iteration.
    #[must_use]
    pub fn new(loop_index: u8) -> Self {
        Self {
            diagnostics: Vec::new(),
            loop_index,
        }
    }

    /// Create a sanitized trace with diagnostics.
    #[must_use]
    pub fn with_diagnostics(loop_index: u8, diagnostics: Vec<CompilerDiagnostic>) -> Self {
        Self {
            diagnostics,
            loop_index,
        }
    }

    /// Parse `cargo --message-format=json` NDJSON output into a sanitized trace.
    ///
    /// Handles two cargo output formats:
    ///
    /// 1. **Compiler messages** (`reason: "compiler-message"`) — from
    ///    `cargo build/check --message-format=json`. Extracts `file_name`,
    ///    `line_start`, `code.code`, and `message` from the primary span.
    ///
    /// 2. **Test failures** (`type: "test", event: "failed"`) — from
    ///    `cargo test --message-format=json`. Extracts test name and `stdout`.
    ///
    /// All other NDJSON lines are silently dropped — structural allowlist
    /// (Security Guardrails §3.4.1). ANSI escape sequences are stripped.
    /// Messages are truncated to 512 bytes. Malformed JSON lines are skipped.
    ///
    /// # Errors
    ///
    /// Returns `SanitizeError::EmptyInput` if `input` is blank or whitespace-only.
    pub fn from_cargo_json(input: &str, loop_index: u8) -> Result<Self, SanitizeError> {
        if input.trim().is_empty() {
            return Err(SanitizeError::EmptyInput);
        }
        let mut diagnostics = Vec::new();
        for raw_line in input.lines() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }
            let Ok(json) = serde_json::from_str::<serde_json::Value>(line) else {
                continue; // silently drop malformed lines — structural allowlist
            };
            if let Some(diag) = parse_compiler_message(&json) {
                diagnostics.push(diag);
            } else if let Some(diag) = parse_test_failure(&json) {
                diagnostics.push(diag);
            }
        }
        Ok(Self {
            diagnostics,
            loop_index,
        })
    }
}

/// Error returned by [`SanitizedTrace::from_cargo_json`].
#[derive(Debug, Error, PartialEq)]
pub enum SanitizeError {
    /// The compiler output string was empty or contained only whitespace.
    ///
    /// This is a hard error — a blank output string cannot represent real
    /// compiler feedback and likely indicates a pipeline configuration mistake.
    #[error("compiler output is empty; expected cargo --message-format=json NDJSON")]
    EmptyInput,
}

// ── Internal parsing helpers ──────────────────────────────────────────────────

/// Extract a `CompilerDiagnostic` from a `cargo --message-format=json` compiler
/// message line. Returns `None` for non-compiler-message lines.
fn parse_compiler_message(json: &serde_json::Value) -> Option<CompilerDiagnostic> {
    if json.get("reason")?.as_str()? != "compiler-message" {
        return None;
    }
    let msg = json.get("message")?;
    let error_code = msg
        .get("code")
        .and_then(|c| c.get("code"))
        .and_then(|c| c.as_str())
        .map(str::to_owned);
    let raw_text = msg.get("message")?.as_str()?;
    let text = truncate_to_bytes(&strip_ansi(raw_text), 512);
    // Use the primary span for location; fall back to the first span.
    let spans = msg.get("spans")?.as_array()?;
    let span = spans
        .iter()
        .find(|s| {
            s.get("is_primary")
                .and_then(|p| p.as_bool())
                .unwrap_or(false)
        })
        .or_else(|| spans.first())?;
    let file = span.get("file_name")?.as_str()?.to_owned();
    let line = span.get("line_start")?.as_u64()? as u32;
    if file.is_empty() || line == 0 {
        return None;
    }
    Some(CompilerDiagnostic {
        file,
        line,
        error_code,
        message: text,
    })
}

/// Extract a `CompilerDiagnostic` from a `cargo test --message-format=json`
/// test-failure line. Returns `None` for non-failure lines.
fn parse_test_failure(json: &serde_json::Value) -> Option<CompilerDiagnostic> {
    if json.get("type")?.as_str()? != "test" {
        return None;
    }
    if json.get("event")?.as_str()? != "failed" {
        return None;
    }
    let name = json.get("name")?.as_str()?;
    let stdout = json.get("stdout").and_then(|s| s.as_str()).unwrap_or("");
    let text = truncate_to_bytes(&strip_ansi(stdout), 512);
    Some(CompilerDiagnostic {
        file: format!("test::{name}"),
        line: 0,
        error_code: None,
        message: if text.is_empty() {
            format!("{name} failed")
        } else {
            text
        },
    })
}

/// Strip ANSI CSI escape sequences (`ESC [ ... <letter>`) from a string.
pub(crate) fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            chars.next(); // consume '['
            for c in chars.by_ref() {
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

/// Truncate `s` to at most `max_bytes`, respecting UTF-8 char boundaries.
pub(crate) fn truncate_to_bytes(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_owned();
    }
    let cut = (0..=max_bytes.min(s.len()))
        .rev()
        .find(|&i| s.is_char_boundary(i))
        .unwrap_or(0);
    s[..cut].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constraint_manifest_rust_canon_has_expected_flags() {
        let m = ConstraintManifest::rust_canon();
        assert!(m.no_unwrap);
        assert!(m.no_expect);
        assert!(m.send_sync_required);
        assert!(m.no_unlisted_deps);
        assert!(!m.no_heap_alloc_inner_loops);
        assert!(m.custom.is_empty());
    }

    #[test]
    fn constraint_manifest_default_off_all_false() {
        let m = ConstraintManifest::default_off();
        assert!(!m.no_unwrap);
        assert!(!m.no_expect);
        assert!(!m.send_sync_required);
        assert!(!m.no_unlisted_deps);
        assert!(!m.no_heap_alloc_inner_loops);
    }

    #[test]
    fn context_vector_roundtrip_json() {
        let cv = ContextVector::new(
            EnvironmentManifest::new("Rust 1.87.0".into(), "2024".into()),
            vec![InterfaceStub::new(
                "src/lib.rs".into(),
                "pub fn foo();".into(),
            )],
            ConstraintManifest::rust_canon(),
        );
        let json = serde_json::to_string(&cv).unwrap();
        let back: ContextVector = serde_json::from_str(&json).unwrap();
        assert_eq!(back.env.runtime, "Rust 1.87.0");
        assert_eq!(back.stubs.len(), 1);
        assert!(back.constraints.no_unwrap);
    }

    #[test]
    fn sanitized_trace_loop_index_preserved() {
        let diag = CompilerDiagnostic::with_error_code(
            "src/main.rs".into(),
            42,
            "E0308".into(),
            "mismatched types".into(),
        );
        let trace = SanitizedTrace::with_diagnostics(1, vec![diag.clone()]);
        assert_eq!(trace.loop_index, 1);
        assert_eq!(trace.diagnostics[0].error_code.as_deref(), Some("E0308"));
        let json = serde_json::to_string(&trace).unwrap();
        let back: SanitizedTrace = serde_json::from_str(&json).unwrap();
        assert_eq!(back.diagnostics[0], diag);
    }
}
