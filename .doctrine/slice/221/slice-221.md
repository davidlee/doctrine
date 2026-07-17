# Unify dispatch boundary writes on the object-db ref

<!-- Rescoped 2026-07-17 after RV-278 (inquisition): the localised "merge in
     commit_boundaries" fix was shown unsound (F-2/F-3), so the slice pivots to
     collapsing the read/write seam entirely — one object-db writer, no
     working-tree boundaries ledger. -->

## Context

Originates from ISS-225, surfaced at SL-220 PHASE-07 conclude (2026-07-17).

The dispatch boundaries ledger (`.doctrine/dispatch/<slice>/boundaries.toml`) is
touched through a **split write/read seam**:

- **`dispatch_conclude_phase` → `conclude_boundary_commit`**
  (`mcp_server/dispatch.rs:543`, SL-199) — the primary boundary writer — lands the
  row on the `dispatch/<slice>` **ref** object-db (read-modify-write over the
  committed tip via `commit_on_behalf`), deliberately leaving the coord working
  tree untouched.
- **`dispatch record-boundary` → `run_record_boundary`** (`dispatch.rs:848`) — a
  rare **manual escape hatch** ("correct a range", per `dispatch/SKILL.md:97`) —
  writes the row to the coord **working tree** via `ledger::record_boundary`.
- **`dispatch sync --prepare-review` → `commit_boundaries`** (`dispatch.rs:2795`)
  reads that **working-tree** file and splices it *whole* over the ref tip.

The clash (ISS-225): after conclude advances the ref object-db, the coord working
tree is a stale checkout; `commit_boundaries` re-commits the stale file over the
tip, **deleting** the rows conclude just landed, and prepare-review's own
completeness gate then halts.

The originally-scoped localised fix — merge ref-base ⊕ working-tree inside
`commit_boundaries` — was taken to adversarial review (**RV-278**) and shown
unsound: a working-tree row that differs from the ref is ambiguous (stale checkout
vs a deliberate `record-boundary` correction) and **content cannot distinguish
them**, so no silent merge rule is safe (F-2); and the merge's append-order breaks
`plan_phases`'s strict row-order phase chaining (F-3). The findings point past the
merge to the real defect: **two writers with two source-of-truth models**. The
verified invariant `mem.pattern.dispatch.sync-tree-reads-ledger-not-worktree`
(SL-064 §4.1) already says sync reads the ledger from the ref; the working-tree
boundaries path is the lone holdout.

## Scope & Objectives

**Collapse the seam: make the object-db ref the single source of truth for
boundary rows, and retire the working-tree boundaries ledger entirely.**

1. **Relocate the object-db dispatch-commit engine.** Move `commit_on_behalf` /
   `commit_tree_as` / `Provenance` / `Identity` / `dispatch_identity` (+ the
   dispatch id constants) from `mcp_server/dispatch.rs` **down** to `dispatch.rs`
   (engine tier), so both the MCP funnel and the CLI can compose object-db commits
   without a `dispatch ↔ mcp_server` cycle (ADR-001). Generalise `commit_on_behalf`
   to take an **explicit target ref + expected_old** instead of deriving the CAS
   target from the coord worktree's `HEAD`, removing its "must run from the coord
   worktree" coupling.
2. **Extract one shared boundary-write helper** — `land_boundary_row(coord_root,
   tip, coord_ref, slice, row) -> CommitOutcome`: read committed boundaries at
   `tip`, UPSERT the row by phase, splice, `commit_on_behalf`. `conclude_boundary_commit`
   delegates to it (behaviour-preserving); `run_record_boundary` calls it instead
   of the working-tree write.
3. **Retire the working-tree path.** Delete `commit_boundaries` and its call in
   `prepare_review` (collapses to `tip = tip0`; `plan_phases` already reads
   boundaries from the ref via `read_ledger`). Delete `ledger::read_boundaries_file`
   and `ledger::record_boundary` (the working-tree reader/writer) once unreferenced,
   with their tests.

Closure intent: exactly one boundary-write path (object-db, ref), the escape hatch
still lands corrections (now on the ref, one step), the ISS-225 clobber is
impossible by construction, and the SL-064 §4.1 invariant holds with no exceptions.

## Non-Goals

- **Unifying the journal/orthogonal compose** (`commit_journal`, `commit_boundaries`
  twins) into the same shared seam beyond what boundary-write needs — the shared
  primitive lands, but rewiring `commit_journal` onto it is a separate cleanup.
  Follow-up.
- **The subprocess-arm symmetric ledger derive** (D6 / IMP-171, deferred) — out.
- **ISS-212 / IMP-272** (completion-flag split-brain) and **ISS-224** (oid
  validation) — sibling stale-state items, tracked separately.

## Affected Surface

- `src/mcp_server/dispatch.rs` — relocate commit primitives + `Provenance`/`Identity`;
  `dispatch_import` / `dispatch_conclude_phase` call `crate::dispatch::` for them;
  `conclude_boundary_commit` delegates to `land_boundary_row`.
- `src/dispatch.rs` — new engine home for the commit primitives + `land_boundary_row`;
  `run_record_boundary` writes the ref; delete `commit_boundaries` + its
  `prepare_review` call site.
- `src/ledger.rs` — delete `read_boundaries_file`, `record_boundary` (+ tests) once
  unreferenced; `read_boundaries` (ref-side, via `dispatch_dir`) audited for
  remaining callers.
- Tests: `tests/e2e_dispatch_sync.rs` + the mcp/dispatch unit suites must stay green
  (behaviour-preservation gate); new clobber regression + escape-hatch-lands-on-ref
  tests.

## Risks & Assumptions

- **Behaviour-preservation gate.** Relocating the commit primitives and delegating
  conclude to the shared helper must be provably behaviour-preserving — the existing
  `mcp_server` / `e2e_dispatch_sync` suites are the proof and must stay green
  *unchanged* through phases 1–2.
- **`commit_on_behalf` contract change.** Generalising it to an explicit ref touches
  a primitive with provenance/CAS tests (VT-4 empty-delta, lost-ref-race); those
  tests move with it and must keep asserting the same invariants.
- **Escape-hatch provenance.** `record-boundary` currently stamps
  `boundary::Provenance::Funnel`; writing the ref keeps the row provenance but adds a
  commit-identity — decide the identity variant at design (likely a `Conclude`-shaped
  or a new `Manual` id) so the escape hatch is attributable.
- **SL-220 binary discipline.** Corpus verbs run via `.dispatch/doctrine-v3-sl220`
  (0.21.0) until `/close` integrates SL-220.

## Open Questions

- Commit-identity for the escape-hatch ref write — reuse `dispatch_identity()` or
  introduce an attributable `Manual`/operator identity? → `/design`.
- Does any consumer besides `plan_phases` still read the working-tree boundaries
  (`read_boundaries` vs `read_ledger`)? Audit at `/plan`.

## Follow-Ups

- Rewire `commit_journal` onto the shared object-db-commit seam (the journal twin).
- ISS-212 / IMP-272, ISS-224 remain open.
