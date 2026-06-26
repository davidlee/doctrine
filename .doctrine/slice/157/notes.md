# SL-157 notes — Checkout-independent integrate

Status: **in `/design`.** Scope (`slice-157.md`) re-baselined to **A** (see split
below); design sections next.

## Design decision log

- **2026-06-26 — A/B split (foundational).** The original maximal scope bundled
  two separable things: (A) checkout-independence — advance via object-DB CAS,
  retire the live-checkout leg + resync + M4 gate (pure R1/R3/R4 hazard
  dissolution; FF-only preserved); and (B) non-FF trunk auto-merge + conflict
  surgery (absorbs IMP-127) — which **reverses ADR-012 D2/D4 FF-only**. Confirmed
  orthogonal & independently shippable: the merge oid must be produced at *plan*
  time (committer-date non-determinism would break D4 replay otherwise), so the
  *land seam is identical* in both — a policy-free CAS to the journaled
  `planned_new_oid`. **Decision: SL-157 = A** (mechanism-only ADR-012 Revision);
  **B → RFC-006** (new, linked) for external review of the ADR reversal. SL-157's
  land seam is kept policy-free so RFC-006 extends it with no rework. The single
  rework trap to avoid in A: do **not** bake FF-only into the land seam — keep
  FF-policy at plan time (`plan_trunk_row` ff-gate, unchanged).
- IMP-127 reference moved off SL-157 onto RFC-006 (the surgery leg is B's).
- **2026-06-26 — scope premise corrected (the design caught it).** The original
  scope said "retire the checked-out leg, always pure CAS — the live checkout buys
  only hazard." **Backwards.** Evidence: (1) RFC-005 H2 localizes R1/R3/R4 to the
  *not-checked-out* leg's speculative post-CAS resync (`advance_pure_ref:1842-1848`);
  it calls the checked-out leg the **safe** one (`ff` syncs atomically, proven by
  `integrate_trunk_checked_out_ff_leaves_clean_tree`). (2) `edge` IS checked out —
  AGENTS.md mandates primary-on-edge — so `--edge` rides the checked-out leg;
  force-CASing it would desync the dev's tree (ISS-038 phantom). (3) `main` is
  never checked out → pure-ref leg; no `main` worktree exists to drop (OQ-A). So
  **corrected A = strip the speculative None-leg resync** (it guards a None→Some
  race impossible under these invariants), retire `resync_worktree_hard` +
  `RacedDesync`; **keep** the checked-out leg + M4 gate. User chose (i) keep the
  safe atomic edge leg over (ii) pure-one-leg (which fights AGENTS.md). OQ-D:
  `resync_worktree_hard` sole caller is the deleted resync → delete; `ff_advance_in_worktree`
  keeps its caller → stays. OQ-B: edge rides the (safe) checked-out leg. OQ-C: N/A
  (no surgery in A). OQ-E: mechanism-only Revision.

## Where this came from

RFC-005 (dispatch funnel integrity) surveyed the integrate hazard (H2). After
SL-152 (H1) and SL-154 (H3) closed, **OQ-5 became the sole live structural
question**: refactor integrate to be checkout-independent, or harden the existing
live-checkout path with more guards? User steer (2026-06-26): **the rewrite —
"high pain, high purity."** Don't guard the race windows; delete the conditions
that create them. SL-157 is that rewrite.

The shape was worked out conversationally and lives in **RFC-005 OQ-5 + Current
posture #1**. The model:

```
land(ref, planned, expected_old):          # ONE seam
  FF?          → CAS
  clean non-FF → merge-tree --write-tree → commit-tree → CAS
  conflict     → ephemeral PRIVATE worktree → human resolves → same CAS  (IMP-127)
```

Key premise that unlocks it: **`main` is never worked in** — it's purely a
contention-buffer ref. No live reader to keep current ⇒ the checkout leg buys
only hazard.

## Preflight evidence map (read-only, 2026-06-26) — cite these

**Integrate path (`src/dispatch.rs`):**
- `integrate` entry — `1696-1777`. M4 dirty pre-gate — `1753-1759`.
- `advance_row` branch point — `1812-1815` (keys on `git::worktree_for_ref`).
- `advance_pure_ref` (the leg to **promote**) — `1822-1853`; CAS at `1828`,
  post-CAS re-probe/resync `1842-1848`, RacedDesync disposition `1848`.
- `advance_checked_out` (the leg to **retire**) — `1859-1888`; `ff_advance` call
  `1867`, non-FF refusal `1885`.

**Object-DB primitives (`src/git.rs`) — already exist, reuse:**
- `merge_tree` (`merge-tree --write-tree --merge-base`) — `844-869`.
- `commit_tree` — `818-825`; `commit_tree_merge` (2-parent) — `875-895`.
- `update_ref_cas` (the universal CAS lander primitive) — `913-926`.
- `worktree_for_ref` — `1236-1242`; `ff_advance_in_worktree` — `1308-1351`;
  `resync_worktree_hard` — `1373-1376`.

**The seam to mirror — candidate flow already does object-DB merge + ephemeral
surgery:** `candidate_create` (`dispatch.rs:889-973`), merge-tree dispatch at
`946`, clean→`commit_tree_merge` `948`, conflict→`--worktree` park. **This is the
pattern to lift into integrate.** DRY: no parallel implementation.

**Checkout-assuming surface to unwind (the risk list):**
- M4 dirty pre-gate `dispatch.rs:1753` — moot if no checked-out trunk.
- post-CAS resync `1842-1848` + `resync_worktree_hard` — moot.
- candidate guards I9 `889`, `1113` — refuse if on `review/*`/`phase/*`; check
  whether they still apply (probably orthogonal, keep).
- `prepare_review` reads primary tree registry `1571`, live coord splice `1540` —
  orthogonal to trunk checkout; leave.

**Conflict handling today:** non-FF on checked-out ref → **refuse**
(`integrate-nonff-checkout`, `1885`); non-FF on pure ref → CAS accepts. No
conflict resolution in integrate (only candidate flow has it). IMP-127 (ingest
hand-resolved conflict) has **no** integrate code path yet — the surgery leg IS
IMP-127.

**Tests that gate the rewrite:**
- `tests/e2e_dispatch_sync.rs` PHASE-05 integrate set, `727-927`:
  `integrate_default_replays_prepared_refs_*` (727),
  `integrate_trunk_fast_forwards_then_is_idempotent` (767),
  `integrate_trunk_refuses_non_fast_forward` (803),
  `prepare_review_projects_off_pinned_fork_point_*` (835),
  `integrate_refuses_clobbered_prepared_ref` (897),
  `integrate_edge_is_opt_in_*` (927). Worker-mode refusal VT-5 `1652+`.
- `src/git.rs` unit tests for the primitives: `ff_advance_in_worktree_*`
  (3958/3985/4007), `resync_worktree_hard_*` (4027), `update_ref_cas_*` (3313),
  `worktree_for_ref_*` (3827) — adapt as legs retire.
- `tests/e2e_dispatch_candidate.rs` — the merge/worktree-surgery reference behaviour.

## Governance / reading list for design

1. **RFC-005** — `doctrine rfc show RFC-005`. Read **Current posture** (top) + H2
   section + OQ-5 + Tension 4. The whole H2 case + R1/R3/R4 definitions are there.
2. **ADR-012** (`.doctrine/adr/012/`) — the integrate topology this Revision
   touches. Bindings: D1 (isolated coordination worktree = write target), D2
   (trunk FF-only opt-in, report-never-auto-resolve), **D4 (CAS replay — PRESERVE
   this)**, D5 (RV refuses on fork). The Revision restates *how the tree is
   produced*, not the CAS safety contract.
3. **ADR-013** — Revision as the change-axis kind; mechanism changes to accepted
   ADRs route through a Revision. This slice needs one against ADR-012.
4. **ISS-038** (H2 phantom, R1 residual), **IMP-122** (R3/R4 resync guards —
   these *dissolve*, don't get built), **IMP-127** (hand-resolved conflict ingest
   — this slice *absorbs* it), **SL-121** (the integrate rework + M4 gate, the
   prior art), **SL-126** (H2 belt — becomes belt-and-braces under the rewrite).
5. Memories: `mem.pattern.dispatch.close-integrate-shared-trunk-race`,
   `mem.pattern.dispatch.review-branch-extraneous-deletions`,
   `mem.pattern.dispatch.sync-tree-reads-ledger-not-worktree`.

## Open questions for `/design`

- **OQ-A.** Does anything actually require `main` to be a *checked-out* worktree
  (vs a bare ref)? Preflight says no live reader — confirm, then decide whether to
  drop the `main` worktree entirely.
- **OQ-B.** Edge aggregate (`integrate_edge_is_opt_in`) — does it have its own
  checkout assumption, or does it ride the same lander?
- **OQ-C.** Ephemeral surgery worktree: where materialised, how isolated, how
  discarded — and does it reuse the candidate `--worktree` machinery wholesale or
  need an integrate-specific variant?
- **OQ-D.** Retire `ff_advance_in_worktree`/`resync_worktree_hard` outright, or do
  any non-integrate callers keep them alive? (grep before deleting.)
- **OQ-E.** ADR-012 Revision scope — minimal (restate integrate mechanism) vs
  broader (does dropping the trunk checkout ripple into D1's coordination-worktree
  posture?).

## Out of scope (don't let design absorb these)

- R2 `/close` ISS-030 recovery — independent skill fix.
- IMP-174 split-brain authored-state — separate axis.
- Candidate-flow re-authoring — reuse only.
