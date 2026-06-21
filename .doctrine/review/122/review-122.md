# Review RV-122 — reconciliation of SL-115

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

This reconciliation review audits SL-115 (decompose `main.rs`) against its design,
plan, and governance. The slice relocated orphan command runners and the entire
clap surface out of `main.rs` (7264 → 170 production LOC), enforcing the D1a
sideways-call rule and the `commands` sink invariant.

**Audit surface:** candidate interaction branch `cand-115-review-001` at
a786fd2b, created from `refs/heads/review/115` (dispatch tip 1f068709 on
`refs/heads/dispatch/115`, refreshed over main at 415d32d).

**Lines of attack:**

1. **D1a enforcement** — MemoryCommand::Sync NOT in memory.rs (carve-out held);
   SpecReqCommand in spec.rs, not requirement.rs (own-module fold).
2. **Sink invariant** — no kind/engine module imports `commands`; `command = 120`
   unchanged in layering gate.
3. **Behaviour preservation** — 2142 tests pass; PHASE-01 parse-regression net
   (21 tests) byte-identical; `tests/e2e_*` goldens untouched.
4. **Completeness** — no `enum *Command` or `fn run_*` in main.rs production
   code; `CommonListArgs` at crate root (not commands/).
5. **Gate pre-filter** — source-file existence filter correct for current
   codebase; verified `Command` is the only non-module target in the 394-edge
   graph.
6. **House standard** — `cargo clippy --bin doctrine` zero-warn;
   `cargo clippy --workspace --exclude cordage` zero-warn.

**Governing doctrine:** ADR-001 module layering; ADR-007 adversarial review;
the storage rule; the pure/imperative split (no impure additions in this
mechanical refactor).

## Synthesis

SL-115 achieved its objective: `main.rs` reduced from 7264 to 170 production LOC
(plus ~1200 LOC of test code that was never in scope for relocation). All four
phases executed cleanly — the verification net (PHASE-01) proved behaviour
preservation end-to-end, the orphan runner relocation (PHASE-02) moved 7 shell
groups into `commands/` sink modules, the kind enum redistribution (PHASE-03)
folded 23 kind enums + dispatch into own-module or `commands/` shells across 5
domain batches, and the dispatch core collapse (PHASE-04) moved `Command` +
`ExportCommand` + the dispatch match into `commands/cli.rs`.

The design's two safety-critical decisions held under audit:

- **D1a** correctly carved out `MemoryCommand::Sync` (→ `commands/cli.rs` sink
  shell, avoiding a `corpus↔memory` 2-cycle) and routed `SpecReqCommand` →
  `spec.rs` (own-module, avoiding a `spec↔requirement` cycle).
- **The sink invariant** (`commands` has zero command-tier inbound edges) proved
  by inspection: no kind/engine module imports `commands`. The gate confirms
  `command = 120` unchanged.

Four findings were raised and resolved:

- **F-1 (minor, aligned):** Trunk merge surfaced unfulfilled lint expectations
  in `main.rs` test module. Fixed by removing `#[expect(unused_imports)]` and
  adding `#[cfg(test)]` during audit.
- **F-2 (minor, tolerated):** ~1200 LOC test module remains in `main.rs`. This
  was never in scope — the design only requires production code relocation.
  Tolerated with backlog item to move to `tests/cli_verification.rs`.
- **F-3 (minor, tolerated):** Stale `review/115` + `phase/115-*` refs required
  manual `git update-ref -d` cleanup before prepare-review could run.
- **F-4 (nit, aligned):** Gate pre-filter by source-file existence is a
  heuristic, not a structural fix. Correct for today's codebase; latent
  limitation for future crate-root types.

**Standing risks:** none. The decomposition is mechanical and behaviour-preserving.
The gate pre-filter is a known tradeoff; the alternative (moving `Command` to its
own module) is a small future improvement, not a risk.

**Tradeoffs consciously accepted:**
- Test module stays in `main.rs` — moving it is pure-code-move with zero
  behavioural risk; not worth blocking closure.
- Gate pre-filter heuristic over structural fix — the edge extractor limitation
  is deep (can't distinguish `crate::Type` from `crate::module` without name
  resolution); the source-file existence check is the correct minimal fix.

## Reconciliation Brief

### Per-slice (direct edit)

- `src/main.rs`: apply the F-1 fix (remove `#[expect(unused_imports)]`, add
  `#[cfg(test)]` to test module) — this was done during audit on the candidate
  worktree; needs committing on the coordination branch.

### Governance/spec (REV)

None. No finding touches design, governance, or spec documents. All four
findings are code-level or tooling observations.

### Follow-up work (backlog)

- Move `main.rs` test module (~1200 LOC) to `tests/cli_verification.rs`
  (handover §6, F-2).

## Reconciliation Outcome

All findings were resolved during audit with no further writes needed.

### Direct edits applied (during audit)

- `src/main.rs` (candidate + coordination): removed unfulfilled `#[expect(unused_imports)]`, added `#[cfg(test)]` to test module — committed as `482a90c8` on `dispatch/115` (RV-122 F-1).

### REVs completed

None. No finding touches design, governance, or spec documents.

### Withdrawn / tolerated

- RV-122 F-2: tolerated — ~1200 LOC test module in main.rs was never in scope; backlog CHR-018 filed for follow-up.
- RV-122 F-3: tolerated — stale review refs needed manual cleanup; tooling UX issue, not a slice defect.
- RV-122 F-4: aligned — gate pre-filter is a known heuristic; no change needed.

Reconcile pass complete — handoff to /close.
