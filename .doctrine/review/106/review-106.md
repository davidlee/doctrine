# Review RV-106 — reconciliation of SL-120

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Audit surface**: `refs/heads/review/120` (squash commit `e1fa7c30`).
Candidate `cand-120-review-001` creation conflicted (main deleted SL-119's pi
extension substrate via commit `587d4403`). Audit runs against review/120
directly; the integration gap surfaces as a finding.

**Lines of attack**:

1. Extension file conformance to design.md — pure functions, process lifecycle,
   error handling, session_start/shutdown, tool registration
2. Boot.rs generation (PHASE-02) — `plan_mcp_extension`, `install_mcp_extension`,
   `RefreshReport` integration, wire reporting, behaviour-preservation
3. Test coverage — unit tests for pure functions, plan install unit tests,
   integration tests against live doctrine MCP server
4. Cross-phase invariants: the extension is self-contained; `BIN_PATH` is baked
   at install time; foreign files are skipped; Claude carries `NotApplicable`
5. Main-branch regression: commit `587d4403` deleted SL-119's pi extension
   machinery (`ExtAction`, `ExtOutcome`, `PI_EXT_HEADER`, `Harness::Pi` →
   `Harness::Codex` rename without preserving the extension arm), leaving the
   Codex arm of `install_refresh` as a no-op — this blocks SL-120 integration.

## Synthesis

The SL-120 implementation on `review/120` is clean and conforms well to its
design. The extension file (`.pi/extensions/doctrine/mcp.ts`, 378 lines)
implements the full MCP bridge: spawn, handshake, tool discovery, registration
with `pi.registerTool()`, tool execution via JSON-RPC `tools/call`, graceful
shutdown, and comprehensive error handling (timeouts, process death, stderr
capture). The boot.rs generation side (`plan_mcp_extension`,
`install_mcp_extension`, wire reporting) follows the established SL-119 pattern
closely. Test coverage is strong: 22 vitest tests (16 unit + 6 integration)
plus 15 new boot tests, all passing.

Four findings were raised and disposed:

- **F-1** (minor, aligned): `stripPiPrefix` present in tests but correctly
  omitted from extension — dead-code removal is a DRY improvement.
- **F-2** (minor, tolerated): `parseResponse` uses unchecked type assertion
  (`as JsonRpcResponse`). Accepted per design's pass-through philosophy — the
  MCP server is a trusted local process.
- **F-3** (minor, aligned): `BIN_PATH` fallback chain (baked → env → hardcoded)
  diverges from design's "baked only" but is a beneficial improvement for
  dev/test workflows.
- **F-4** (blocker, fix-now, RESOLVED): Main-branch regression — commit
  `587d4403` deleted SL-119's pi extension machinery. Fixed by reverting the
  harmful deletion while preserving the Pi→Codex harness rename. Main now
  builds cleanly, all 99 boot tests pass, and `review/120` merges cleanly.

**Standing risks**:

1. The `include_str!` pattern for extension templates requires the TS files to
   be git-tracked (force-added past `.gitignore`). A fresh checkout won't build
   without a prior `doctrine boot install` run — a bootstrapping hazard shared
   with SL-119.
2. The `parseResponse` type assertion means malformed MCP responses produce
   confusing errors rather than clear validation failures. Low risk in practice
   (trusted local server).
3. Backlog items IMP-107 (wire ReviewError variants), IMP-111 (Codex MCP server
   registration during install), and IMP-117 (pi extension bridging `.mcp.json`)
   bear on the MCP surface this extension bridges — the pi install surface
   should not be left behind when those items widen the MCP server.

**Tradeoffs accepted**:

- Pass-through params (`Type.Object({}, {additionalProperties: true})`) over
  typed schemas — simpler, matches MCP's JSON Schema descriptions, trusts the
  LLM to construct correct args from tool descriptions.
- `BIN_PATH` duplicated rather than shared between `index.ts` and `mcp.ts` —
  deliberate independence so either extension can exist without the other.
- Runtime BIN_PATH fallback over pure compile-time baking — pragmatic dev
  affordance at minor cost to the "baked only" design ideal.

**Gate results**:

- `cargo test --bin doctrine -- boot`: 99/99 pass (including 15 new MCP tests)
- `cargo clippy --bins`: zero warnings
- Vitest: 22/22 pass (16 unit + 6 integration)
- Behaviour-preservation: no pre-existing tests harmed
- Merge: `review/120` merges cleanly onto current main after F-4 fix

## Reconciliation Brief

### Per-slice (direct edit)

- **design.md §Pure functions**: Remove `stripPiPrefix` from the listed pure
  functions, or add a note that it's test-only. The implementation correctly
  omits it from the extension.
- **design.md §Tool registration / BIN_PATH**: Document the BIN_PATH fallback
  chain (`baked → env → hardcoded`) so the design reflects the implementation's
  dev affordance.

### Governance/spec (REV)

(No governance or spec changes are needed — the audit found no design-wrong or
spec-governance findings. The only blocker was a code regression on main, fixed
by commit `b059eac4`.)
