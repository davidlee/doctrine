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
