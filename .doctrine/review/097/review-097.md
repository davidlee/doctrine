# Review RV-097 — reconciliation of SL-109

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit of SL-109 (MCP server for review commands) against its
design.md, plan.toml, and the implemented code. Four phases, all completed.

**Lines of attack:**

1. **Design fidelity** — does the implementation honour all eight design
   decisions (D1-D8) and the output contract (§4)?
2. **Behaviour preservation** — do the 74 existing review unit tests pass
   unchanged (the gate from the design)?
3. **MCP protocol integrity** — do the 10 tools, error mapping, and JSON-RPC
   transport match the design's protocol mapping (§5)?
4. **VT coverage** — do the 9 integration tests (PHASE-04) satisfy all 9 VT
   criteria from plan.toml?
5. **Error propagation** — do the three `anyhow::bail!` → `ReviewError`
   conversions from PHASE-04 notes land correctly in review.rs?
6. **Forward compatibility** — are the unused `ReviewError` variants
   (`LockContention`, `DanglingRef`) a design gap or a documented follow-up?

**Evidence:** `just check` passes (1894 tests, 0 failures); `cargo test --bin
doctrine -- review::` passes (74/74, unchanged); `cargo test --test
e2e_mcp_server` passes (9/9); `cargo clippy` zero warnings; `cargo fmt` clean.

## Synthesis

### Summary Judgement

**Clean conformance — no blockers.** The implementation faithfully realises all
eight design decisions (D1-D8), produces CLI output identical to the pre-refactor
code (74 golden + unit tests unchanged), exposes all 10 review verbs as MCP tools
with correct JSON Schema, and maps errors by variant identity (never string-parsing).
The three `anyhow::bail!` → `ReviewError` propagation fixes from PHASE-04 (step 4
role check, per-finding gate, gate() state check) are correctly wired.

Two minor `ReviewError` variants (`LockContention`, `DanglingRef`) are defined in
the enum and handled in the MCP error mapper but never constructed — the source
code paths that should produce them (`lock_guard.rs`, `run_new`'s target validation)
still use `anyhow::bail!`. The `#[expect(dead_code)]` on the enum acknowledges this.
Both are documented in notes.md as follow-up work; neither blocks functionality —
the errors surface correctly, just as `Internal` instead of the dedicated variant.

### What survived scrutiny

- **ReviewOutput** — 11 variants, one per verb, carrying typed fields (D1).
  Externally-tagged serde serialization produces `{"Created":{...}}` etc.
- **ReviewError** — 6 variants, all handled in the MCP error mapper by variant
  identity via `downcast_ref` (D8, RV-092 F-1 penance).
- **with_turn** — generic over `F: FnOnce(...) -> anyhow::Result<T>` (D2).
  Unit test `with_turn_accepts_non_unit_closure_return` verifies.
- **print_review** — single formatting pass, one match arm per variant, output
  matching the §4 contract. Golden tests validate all 10 action/render variants.
- **MCP server** — thin wrapper calling `run_*` directly (D3). Zero new crate
  dependencies (D4). Project root resolved once at startup (D5). `--mcp` flag
  gates serve mode (D6). 10 tools with JSON Schema (D4).
- **Error mapper** — downcasts by variant identity (D8). Unit tests verify
  `RoleMismatch → -32602`, `NotFound → -32000`, `StateMismatch → -32602`,
  `LockContention → -32000`, `Internal → -32000`.
- **Integration tests** — 9 tests covering VT-1 through VT-9. All 9 pass.
  Tests spawn the real binary, drive JSON-RPC 2.0 over stdio, and verify
  authored state on disk.
- **Behaviour preservation** — 74 existing review unit tests pass without a
  single assertion change. The design's behaviour-preservation gate holds.

### Findings Disposition

| Finding | Severity | Charge | Disposition |
|---------|----------|--------|-------------|
| F-1 | minor | `ReviewError::LockContention` defined but never constructed | **follow-up** — additive wiring, documented in notes.md |
| F-2 | minor | `ReviewError::DanglingRef` defined but never constructed | **follow-up** — additive wiring, documented in notes.md |

### Standing Risks

- **RSK-001 (design, mitigated):** Hand-rolled MCP protocol (~300 lines).
  Mitigated by 9 integration tests exercising the full protocol handshake
  and all 10 tool round-trips against the real binary.
- **RSK-003 (design, verified):** Baton CAS under batch mutation. The
  existing per-review lock (ADR-007 D-C4a) serialises concurrent access.
  The MCP path rides `with_turn` exactly as the CLI does — no new concurrency
  surface.

## Reconciliation Brief

No spec or governance changes required. The two minor gaps (F-1, F-2) are
code-level follow-ups already captured in notes.md; they do not touch design.md,
ADRs, or spec documents.

### Per-slice (direct edit)

- None.

### Governance/spec (REV)

- None.

## Reconciliation Outcome

No-op reconcile — all findings (F-1, F-2) are code-level `follow-up` items
already captured in SL-109 notes.md. No per-slice direct edits, no governance
or spec changes, no REVs required. The two `ReviewError` variants
(`LockContention`, `DanglingRef`) are defined and mapped but have wiring gaps
in upstream call sites — additive work, not a defect in shipped behaviour.

Reconcile pass complete — handoff to /close.
