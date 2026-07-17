# Dispatch prepare-review clobbers concluded ledger rows

## Context

Originates from ISS-225, surfaced at SL-220 PHASE-07 conclude (2026-07-17).

The dispatch boundaries ledger (`.doctrine/dispatch/<slice>/boundaries.toml`) is
touched through two mechanisms that disagree on their source of truth:

- **`dispatch_conclude_phase`** (MCP funnel) lands the phase-boundary row
  **object-db only** — read-modify-write over the committed `dispatch/<slice>`
  tip tree via `git::read_path_at` → splice → `commit_on_behalf`, advancing the
  ref without touching the coordination working tree (by design; SL-199,
  commit-on-behalf comment `mcp_server/dispatch.rs:271-279`).
- **`dispatch sync --prepare-review`** opens by committing the boundaries ledger
  **from the coordination working tree** — `commit_boundaries`
  (`dispatch.rs:2795`) reads the on-disk file via `read_boundaries_file`
  (`ledger.rs:528`, `std::fs::read_to_string`) at `dispatch.rs:2802`, splices it
  into the tip tree, and advances the ref whenever the tree-oid differs.

The clash: a conclude that isn't followed by a working-tree sync leaves the
coord `boundaries.toml` stale. prepare-review then re-commits that stale file
over the tip, **deleting the row conclude just landed**, and its own
completeness gate fails on the self-inflicted gap ("completed phase PHASE-NN has
no recorded source-delta row") — a halt with a misleading signature.

This is the exact trap named by the verified memory
`mem.pattern.dispatch.sync-tree-reads-ledger-not-worktree` (SL-064 design §4.1):
sync-side ledger reads must go through the branch tip
(`read_path_at`/`read_ledger`); the filesystem `read_*`/`record_*` are the
funnel's RMW side and are a trap when used in the sync. `commit_boundaries` is
the one sync-side reader that never adopted the invariant. Today the gap is
guarded only by operator ritual (`git restore --source=HEAD --staged --worktree
-- <ledger paths>` after every object-db write), enforced nowhere.

## Scope & Objectives

Close the write/read seam so an object-db conclude can never be clobbered by a
subsequent prepare-review — restoring the established "sync reads from the ref,
not the worktree" invariant for the boundaries-commit step.

Design (`/design`) picks the mechanism between the two candidate fixes named in
ISS-225; both kill the class:

1. **prepare-review sources its ledger commit from the ref** — `commit_boundaries`
   reads the concluded ledger from the `dispatch/<slice>` tip (align with the
   `read_ledger`/`read_path_at` seam the rest of sync already uses) rather than
   the working tree; only commit a working-tree ledger that is strictly newer,
   or refuse on divergence. Localised in `dispatch.rs`. *Author's preference;
   aligns with the verified SL-064 invariant.*
2. **conclude checkout-syncs the coord working tree** for the paths it writes,
   closing the gap at the source (and generalising to every object-db ledger
   writer).

Closure intent: the clobber sequence is impossible by construction, proven by a
regression test that reproduces conclude → prepare-review and asserts the
concluded row survives; the existing `commit_boundaries` idempotency contract
(SL-154 PHASE-04 VT-2 — one `ledger: boundaries` commit, stable blob oid) and
`e2e_dispatch_sync.rs` suite stay green.

## Non-Goals

- **ISS-212 / IMP-272** — phase-*completion-flag* divergence (per-worktree
  gitignored runtime sheets read from the primary tree). A distinct stale-state
  class in the same conclude beat, different mechanism and fix surface. Out.
- **ISS-224** — `dispatch_conclude_phase` stores boundary oids verbatim without
  resolve/validate. Same function as candidate fix 2, but a validation concern,
  not the read/write seam. Out (may be co-touched if fix 2 is chosen; decide at
  design).
- Broader unification of the two compose arms (`commit_on_behalf` vs
  `commit_boundaries`/`commit_journal`) into one shared object-db helper — a
  larger refactor; this slice fixes the boundaries clobber, not the arm split.

## Affected Surface

- `src/dispatch.rs` — `commit_boundaries` (2795), `prepare_review` call site
  (1959, 1972), `read_ledger` (2652), `with_journaled_projection` (2885).
- `src/mcp_server/dispatch.rs` — `dispatch_conclude_phase` (510),
  `conclude_boundary_commit` (543), `commit_on_behalf` (280) — read side if
  candidate fix 2.
- `src/ledger.rs` — `read_boundaries_file` (528), `dispatch_dir` (367).
- `src/git.rs` — `read_path_at`, `tree_with_file`, `commit_tree`,
  `update_ref_cas`.
- Tests: `tests/e2e_dispatch_sync.rs` (idempotency + stale-ref invariants must
  hold), plus a new conclude→prepare-review clobber regression.

## Risks & Assumptions

- **Idempotency contract.** `commit_boundaries` is content-idempotent by tree-oid
  compare (SL-154 PHASE-04 VT-2). Any change to its read source must preserve
  that grain — assert at commit-count + blob-oid, not full-rerun tip equality.
- **Legitimate worktree edits.** If an operator hand-edits `boundaries.toml` in
  the coord tree between conclude and prepare-review, fix 1 must define
  precedence (ref vs newer worktree). Assume ref is authoritative unless the
  worktree is strictly newer; confirm at design.
- **SL-220 binary discipline.** Corpus verbs run via `.dispatch/doctrine-v3-sl220`
  (0.21.0) until `/close` integrates SL-220; the primary tree's older binary
  must not write the migrated corpus.

## Open Questions

- Which candidate fix — read-from-ref (localised, invariant-aligned) vs
  conclude-side checkout-sync (generalises to all object-db writers)? → `/design`.
- If fix 1: on ref/worktree divergence, refuse-and-halt or take-newer? Precedence
  semantics.

## Follow-Ups

- ISS-212 / IMP-272 (completion-flag split-brain) and ISS-224 (oid validation)
  remain open — sibling stale-state items in the same beat, tracked separately.
