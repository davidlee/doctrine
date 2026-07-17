# Design SL-221: Dispatch prepare-review clobbers concluded ledger rows

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

`dispatch sync --prepare-review` can delete phase-boundary rows that
`dispatch_conclude_phase` has already committed, then halt its own completeness
gate on the self-inflicted gap (ISS-225). Close the write/read seam so an
object-db conclude can never be clobbered by a subsequent prepare-review, while
keeping the CLI/subprocess arm's working-tree-recorded rows landing as before.

## 2. Current State

The dispatch boundaries ledger (`.doctrine/dispatch/<slice>/boundaries.toml`) has
**two writers on two arms**:

- **CLI / subprocess (pi) arm** — `run_record_boundary` (`dispatch.rs:848`, the
  `dispatch record-boundary` verb) → `ledger::record_boundary` (`ledger.rs:555`)
  writes the **working tree**, uncommitted. The rows reach the ref only when
  `commit_boundaries` splices them at prepare-review (ISS-039, design §5.2 step 1).
- **claude arm** — `dispatch_conclude_phase` → `conclude_boundary_commit`
  (`mcp_server/dispatch.rs:543`, SL-199) writes the **ref** object-db by
  read-modify-write over the committed tip, **deliberately** leaving the working
  tree untouched (the fault-atomicity argument at `mcp_server/dispatch.rs:505-509`
  rests on the working tree staying byte-unchanged).

`commit_boundaries` (`dispatch.rs:2795-2823`) reads the working-tree ledger
(`read_boundaries_file`, `ledger.rs:528`), serialises it *whole* (`to_toml`), and
splices it as the **entire** `boundaries.toml` over the ref tip tree
(`tree_with_file`, line 2811). The ref tip is only the splice *base*; all content
comes from the working tree. So when the working tree is a stale prefix of the ref
(the claude arm never refreshes it after conclude advances the ref), the whole-file
replace **drops** the rows the ref gained — the ISS-225 sequence: conclude lands
01–07 on the ref, the coord checkout still holds 01–06, prepare-review re-commits
01–06 over the tip, PHASE-07 is deleted, and the completeness gate bails
("completed phase PHASE-07 has no recorded source-delta row").

This violates the verified invariant `mem.pattern.dispatch.sync-tree-reads-ledger-not-worktree`
(SL-064 design §4.1): sync-side ledger reads go through the ref
(`read_path_at`/`read_ledger`); the filesystem `read_*` are the funnel's RMW side.
`commit_boundaries` is the one sync-side reader that never adopted it — but naive
adoption ("always read from ref") is wrong: it would drop the CLI arm's
working-tree-only rows.

## 3. Forces & Constraints

- **Both arms must keep working.** CLI-arm rows live only in the working tree until
  `commit_boundaries`; claude-arm rows live only on the ref. Neither source can be
  discarded.
- **SL-199 working-tree-free conclude** — `conclude_boundary_commit` must not be
  made to write the working tree; its fault-atomicity property depends on it.
- **Behaviour-preservation gate** — `tests/e2e_dispatch_sync.rs` (idempotency,
  stale-ref, refused-row) must stay green *unchanged* (AGENTS.md; the existing
  suites are the proof when touching shared dispatch machinery).
- **F1 idempotency** (SL-154 PHASE-04 VT-2) and **F3 validate-before-commit** must
  survive the change.
- **STD-001** — no new magic strings; reuse existing path/message literals.
- **ADR-012** — dispatch integration topology; the coord ref is the run SSoT.

## 4. Guiding Principles

The ref is the single source of committed truth; the working tree is a staging
area for the CLI arm's not-yet-committed rows. `commit_boundaries` *folds* staging
into truth — it never lets staging *replace* truth. Arm-agnostic: no arm detection;
the merge is correct for both because their row-sets are disjoint by phase.

## 5. Proposed Design

### 5.1 System Model

`commit_boundaries` changes from **working-tree-as-whole-content** to
**ref-base ⊕ working-tree overlay**:

1. Read the committed ref boundaries at `parent` (the splice base) as the merge
   base — never dropped.
2. Read the working-tree boundaries; fold each row in **by phase, for phases the
   ref does not already carry** (ref-wins-existing).
3. Serialise the merge and splice it (unchanged F1 tree-oid idempotency + CAS
   advance from line 2808 onward).

Worked cases:

| ref (committed) | working tree | merged result |
|---|---|---|
| 01–07 (conclude) | 01–06 (stale prefix) | **01–07** — 07 survives (the fix) |
| ∅ | 01–03 (CLI arm) | **01–03** — CLI arm unchanged |
| 01–02 | 01–03 (CLI new phase) | **01–03** — genuinely-new 03 lands |

### 5.2 Interfaces & Contracts

Signature unchanged: `fn commit_boundaries(root, parent, coord_ref, coord, slice)
-> anyhow::Result<String>`. Body:

```rust
let slice3 = format!("{slice:03}");
let path = format!(".doctrine/dispatch/{slice3}/boundaries.toml");

// Base: committed ref boundaries, read at `parent` (the splice base) — never dropped.
let mut merged: Boundaries = read_ledger(root, parent, &slice3, "boundaries.toml")?;

// Overlay: fold the CLI arm's uncommitted working-tree rows for NEW phases only.
if let Some(raw) = crate::ledger::read_boundaries_file(&coord.path, slice)? {
    let working = Boundaries::parse(&raw).with_context(|| {
        format!("commit_boundaries: working boundaries.toml for dispatch/{slice3} is malformed")
    })?;
    for row in working.rows {
        if !merged.rows.iter().any(|r| r.phase == row.phase) {
            merged.rows.push(row);
        }
    }
}

let canonical = merged.to_toml()?;
let tip_tree = tree_of(root, parent)?;
let candidate = git::tree_with_file(root, &tip_tree, &path, &canonical)?;
if candidate == tip_tree { return Ok(parent.to_owned()); }
let commit = git::commit_tree(root, &candidate, parent, "ledger: boundaries")?;
// CAS advance — unchanged.
```

`read_ledger(root, parent, …)` passes the `parent` **oid** as the read ref;
`git::read_path_at` (`cat-file -p <oid>:<path>`) resolves oids, so the base is read
at exactly the splice base — no TOCTOU re-resolution of `coord_ref`.

### 5.3 Data, State & Ownership

`BoundaryRow` (`boundary.rs:20`) and `Boundaries` (`ledger.rs`) unchanged. The
merge is over `Boundaries.rows`, keyed by `phase` (immutable id). No new fields, no
schema change, no new file. Ordering: ref rows first (their committed order), then
appended new working-tree rows — `plan_phases` (`dispatch.rs:2702`) chains phases
by their own order and skips empty-code phases, and does not assume a global sort,
so append order is safe. (Confirmed at plan time.)

### 5.4 Lifecycle, Operations & Dynamics

No change to when `commit_boundaries` runs (top of `prepare_review`,
`dispatch.rs:1972`), to `conclude_boundary_commit`, `commit_on_behalf`, or any
`ledger.rs` writer. The operator ritual `git restore --source=HEAD --staged
--worktree -- <ledger paths>` after an object-db write becomes **unnecessary** for
the boundaries clobber (its raison d'être here); it may still matter for the
staged-reversal class (`mem.pattern.dispatch.mcp-import-lands-object-db-coord-tree-stale`)
which is out of scope.

### 5.5 Invariants, Assumptions & Edge Cases

- **F1 idempotency preserved** — merge is a pure deterministic function of (ref,
  working); a re-run yields the identical `Boundaries` → identical canonical TOML →
  identical tree → no ref advance. Assert at commit-grain (one `ledger: boundaries`
  commit, stable blob oid), never full-rerun tip equality (journal churn — see
  `mem.pattern.dispatch.prepare-review-rerun-not-idempotent-until-gate`).
- **F3 validate-before-commit preserved** — a malformed working ledger still
  `Err`s with the tip untouched. (A1) *Assumption:* strictness stays even when the
  ref alone would suffice (claude arm with a corrupt working file); a malformed
  coord ledger is a real fault worth surfacing. Named as OQ-1 in case that bites.
- **ref-wins-existing tie-break** — safe because the two arms' phase-sets are
  disjoint in a real run and the claude-arm working tree is only ever a stale
  *prefix* of the ref (never a newer version of a ref phase). (E1) *Accepted
  limitation:* a CLI operator re-recording an *already-committed* phase with a
  corrected oid will not overwrite the ref row. Pathological; flagged, not solved.
- **No arm detection** — correctness does not depend on knowing the arm; the merge
  is right for both.

## 6. Open Questions & Unknowns

- **OQ-1** — Should a *malformed working* `boundaries.toml` be tolerated when the
  ref is complete (claude arm), or keep the hard `Err` (F3)? Default: keep strict.
  Revisit only if it bites an operator.
- **OQ-2** — Test altitude: unit on `commit_boundaries` vs an e2e case in
  `e2e_dispatch_sync.rs`. Resolve at `/plan` against the existing harness shape.

## 7. Decisions, Rationale & Alternatives

- **D1: Merge inside `commit_boundaries` (ref-base ⊕ working overlay).** *Chosen.*
  Fixes the class at the sync read seam (ISS-225 author's preference), keeps
  conclude working-tree-free (SL-199), needs no arm awareness, localised to one
  function. Restores the SL-064 §4.1 invariant for the last sync-side reader.
- **D1-alt (A): conclude checkout-syncs the working tree.** *Rejected.* Simpler
  splice, but reintroduces the working-tree write SL-199 deliberately removed
  (breaks the fault-atomicity argument at `mcp_server/dispatch.rs:505-509`), and
  only closes *this* writer's gap — any future object-db ledger writer reopens it.
- **D1-alt (B): "always read from ref", drop the working-tree read.** *Rejected.*
  Drops the CLI/subprocess arm's working-tree-only rows entirely → prepare-review
  emits no phases on that arm.
- **D2: ref-wins-existing (add new working phases only).** *Chosen* over
  working-wins (would re-clobber the claude stale-prefix case) and over a
  provenance-aware merge (unneeded complexity — `BoundaryProvenance` exists but the
  disjoint-phase-set argument makes it moot).

## 8. Risks & Mitigations

- **R1: silent divergence not surfaced.** If ref and working genuinely disagree on
  a phase's oid, ref-wins hides it. *Mitigation:* disjoint-by-arm makes this
  unreachable in real runs; E1 documents the one contrived path. No gate added
  (would be dead code).
- **R2: regression in the CLI-arm empty-ref path.** *Mitigation:* on an empty ref
  the merge is byte-identical to today's whole-file splice; the existing
  `e2e_dispatch_sync.rs` suite (which exercises that path) must pass unchanged —
  the behaviour-preservation proof.

## 9. Quality Engineering & Validation

- **VT-1 (new, red-first): conclude→prepare-review clobber regression.** Seed a
  coord ref with boundaries 01–07 committed object-db (mirroring conclude), write a
  stale working-tree `boundaries.toml` at 01–06, run `commit_boundaries`, assert
  the resulting ref `boundaries.toml` still contains PHASE-07 (and 01–06). Fails on
  today's whole-file splice; passes on the merge.
- **VT-2 (preserved): F1 idempotency** — two `commit_boundaries` runs ⇒ exactly one
  `ledger: boundaries` commit + stable committed blob oid (SL-154 PHASE-04 VT-2).
- **VT-3 (preserved): `e2e_dispatch_sync.rs`** idempotency / stale-ref (EX-5) /
  refused-row (VT-4) green *unchanged*.
- **VT-4 (new): CLI-arm new-phase overlay** — ref 01–02, working 01–03 ⇒ merged
  01–03 (guards ref-wins-existing still adds genuinely new phases).

## 10. Review Notes

(Adversarial pass appended below after the internal hostile review.)
