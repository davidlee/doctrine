# SL-157 research — evidence assessment

**Date:** 2026-06-26
**Purpose:** architectural review-only research — confirm correctness of the SL-157
thesis against governance, code, tests, and project invariants.

## Thesis under review

> Strip `advance_pure_ref`'s speculative post-CAS resync (dispatch.rs:1842-1848,
> where R1/R3/R4 live); retire `resync_worktree_hard` + the `RacedDesync`
> disposition; keep the safe checked-out ff leg + M4 gate (edge is always checked
> out per AGENTS.md). FF-only preserved → mechanism-only ADR-012 Revision. The
> non-FF auto-merge that reverses FF-only is split to RFC-006.

## Evidence inventory

### 1. Project invariants (load-bearing)

**AGENTS.md line 24** (canonical, in both the root and
`.doctrine/state/dispatch/coordination-136/` mirror):

> the main worktree stays on edge. DO NOT checkout the primary working tree
> to another branch

**Memory: `mem.signpost.project.orientation`** (mem_019ef1ae52c):
> Edge/main split. Primary worktree stays on `edge`. Promote to `main` via
> `git fetch . edge:main`.

These two sources establish the invariants that make the thesis correct:

| Ref | Invariant | Source |
|-----|-----------|--------|
| `main` (trunk) | **Never checked out** — buffer ref, advanced via `git fetch . edge:main` | AGENTS.md + orientation memory |
| `edge` | **Always checked out** — primary worktree, AGENTS.md mandate | AGENTS.md line 24 |

Under these invariants:
- `worktree_for_ref(main)` → **always `None`** → always takes the pure-ref leg
- `worktree_for_ref(edge)` → **always `Some`** → always takes the checked-out leg
- The None→Some transition the resync guards **cannot occur**

### 2. Code — hazard locus confirmed

**`advance_pure_ref`** (`src/dispatch.rs:1822-1853`):

```rust
fn advance_pure_ref(root: &Path, row: &mut JournalRow, planned: &str, expected_old: &str)
-> anyhow::Result<RowOutcome> {
    match git::update_ref_cas(root, &row.target_ref, planned, expected_old)? {
        RefCas::Moved { actual } => { /* refuse */ }
        RefCas::Updated => {
            // HAZARD LOCUS — lines 1842-1848:
            let disposition = match git::worktree_for_ref(root, &row.target_ref)? {
                None => Disposition::AdvancedPureRef,           // normal path
                Some(wt) if git::tree_clean(&wt)? => {          // R3/R4 live here
                    git::resync_worktree_hard(&wt, planned)?;   // IMP-122 hazards
                    Disposition::AdvancedResynced
                }
                Some(_dirty) => Disposition::RacedDesync,       // R1 lives here
            };
            Ok(RowOutcome::Done { disposition })
        }
    }
}
```

The `worktree_for_ref` re-probe at line 1842 can only return `None` for `main`
(per invariants above) and `Some` for `edge`. But `edge` takes
`advance_checked_out` — it never enters `advance_pure_ref`. So lines 1844-1848
are **reachable only if someone checks out `main`**, which AGENTS.md forbids.

**`advance_checked_out`** (`src/dispatch.rs:1859-1893`):
Uses `ff_advance_in_worktree` → `git merge --ff-only`, syncing ref+index+tree
atomically. Proven safe by the checked-out e2e test. **Preserved unchanged.**

**M4 dirty pre-gate** (`src/dispatch.rs:1754-1759`):
```rust
for row in &journal.rows {
    if let Some(wt) = git::worktree_for_ref(root, &row.target_ref)?
        && !git::tree_clean(&wt)?
    {
        bail!("integrate-dirty-worktree ({})", row.target_ref);
    }
}
```
This runs before any ref mutation. `worktree_for_ref(main)` is always `None`,
so for trunk it's a no-op. For `edge` it fires when dirty — correct edge-dirty
protection. **Preserved unchanged.**

### 3. Code — deletions confirmed

| Item | Line(s) | Sole caller? | Action |
|------|---------|--------------|--------|
| `resync_worktree_hard` fn | `git.rs:1373-1376` | Only caller is the resync at `dispatch.rs:1845` (OQ-D grep-confirmed) | Delete |
| `resync_worktree_hard` unit test | `git.rs:4023-4037` | N/A — tests the deleted fn | Delete |
| `Disposition::RacedDesync` variant | `dispatch.rs:2272` | Only reachable from deleted resync | Delete |
| `Disposition::RacedDesync` label arm | `dispatch.rs:2284` | Part of the deleted variant | Delete |
| `AdvancedResynced` docs | `dispatch.rs:2260-2264` | Doc mentions "or a None-leg…" — now only checked-out leg | Trim |

**`report_integrate` (`dispatch.rs:1895-1928`):** Verified — `grep -rn RacedDesync`
across the codebase returns zero matches inside the `report_integrate` match body.
The function's only disposition-aware branch is `Disposition::NoOp` vs a catch-all
`disp =>` arm; `RacedDesync` was handled by that catch-all, identically to
`AdvancedPureRef` and `AdvancedResynced`. The `RacedDesync` variant and its
`label()` arm are simply deleted; the catch-all arm narrows by one variant
implicitly. **No structural change to `report_integrate` is required.**

This supersedes `notes.md`'s TODO about a "warning-line branch" in
`report_integrate` — the grep-confirmed answer is: there is no such branch.
Only the stale doc-comment at line 1893 ("a \`raced-checkout-desync\` is a
non-fatal warning line") needs trimming.

### 4. Tests — behaviour-preservation gate

**Must stay green unchanged** (in `tests/e2e_dispatch_sync.rs`):

| Test | Line | What it proves |
|------|------|----------------|
| `integrate_trunk_fast_forwards_then_is_idempotent` | 767 | FF advance + idempotent replay |
| `integrate_trunk_refuses_non_fast_forward` | 803 | Non-FF refusal preserved |
| `integrate_refuses_clobbered_prepared_ref` | 897 | CAS refusal on moved target |
| `integrate_trunk_checked_out_ff_leaves_clean_tree` | 962 | **VT-2**: checked-out FF leg is atomic, no phantom |
| `integrate_trunk_not_checked_out_advances_ref_leaves_live_checkout_clean` | 1000 | **VT-1**: pure-ref CAS doesn't desync live checkout |

**VT-1** (`integrate_trunk_not_checked_out_advances_ref_leaves_live_checkout_clean`,
line 1000) is the most relevant: it creates a `release` ref that is NOT checked out
anywhere, advances it by pure CAS, and asserts the live `main` checkout is
untouched. This test exercises `advance_pure_ref`'s `None`→`AdvancedPureRef`
path — the one that survives in SL-157.

**Only test removal:** `resync_worktree_hard_resyncs_stale_index_after_pure_ref_advance`
(`git.rs:4027`), which tests the deleted `resync_worktree_hard`.

### 5. Governance alignment

**RFC-005 — H2/OQ-5:**
- Steers toward checkout-independent integrate (SL-157)
- Design correction (2026-06-26): R1/R3/R4 localised to the None-leg resync
  (`advance_pure_ref:1842-1848`); checked-out leg is the safe one
- OQ-5 resolution: structural rewrite (strip resync), not guard pile
- Consistent with SL-157's A/B split

**RFC-005 §2 Current posture #1:**
> SL-157 = strip the speculative None-leg resync (it guards a None→Some race
> that cannot occur), retire resync_worktree_hard + the RacedDesync disposition;
> keep the checked-out leg + M4 gate (edge needs them). R1/R3/R4 dissolve;
> FF-only preserved → mechanism-only ADR-012 Revision.

**RFC-006 (B — split):**
- Edits `plan_trunk_row` (plan-time merge-oid producer), *not* the advance leg
  SL-157 edits
- **Disjoint code paths**: plan vs advance. No rework if B follows A
- Reverses ADR-012 D2/D4 FF-only → properly routed via RFC (ADR-014) for
  external review before any Revision
- A ships FF-only preserved; B extends with non-FF capability

**ADR-012 D2/D4 — preserved unchanged:**
- D2: intent_target defaults; trunk_ff_only is the explicit opt-in
- D4: FF-only, never auto non-ff, CAS-replay with journal, moved-target refusal
- SL-157 does not touch the FF-only gate, the CAS contract, or the journal
  recovery model. It only simplifies the advance mechanism.

**ADR-013 — correct vehicle for mechanism change:**
- Revision is a first-class, standalone change-axis kind with a work lifecycle
- SL-157's Revision is mechanism-only (restatement of the integrate topology —
  the not-checked-out advance is pure ref CAS with no worktree resync)
- No governance reversal → no RFC needed for the Revision itself
- Route: `doctrine revision …` per ADR-013

### 6. Risk assessment

**R1/R3/R4 dissolution — sound:**
- R1 (RacedDesync, low×high): removed with the resync. The race it guards
  (None→Some checkout) cannot occur under the AGENTS.md invariants.
- R3 (IMP-122 F-1 — concurrent advance clobbered by reset --hard):
  With the resync removed, there is no `reset --hard` to clobber.
- R4 (IMP-122 F-2 — untracked collision before reset --hard):
  With the resync removed, there is no `reset --hard` to overwrite untracked.

**IMP-122 can be closed** after SL-157 lands — its F-1 and F-2 hardenings
target the exact code being deleted.

**Edge safety — confirmed:**
- Edge stays on the checked-out leg (`advance_checked_out` → `ff_advance_in_worktree`)
- M4 dirty pre-gate protects edge (fires only for checked-out targets)
- The atomic `git merge --ff-only` syncs ref+index+tree — regression-proven by
  `integrate_trunk_checked_out_ff_leaves_clean_tree`

**No new hazard introduced:**
- The deletion removes only the speculative post-CAS re-probe
- Before deletion: CAS succeeds → probes for checkout → advances anyway (even
  with RacedDesync, the ref already moved)
- After deletion: CAS succeeds → done. Same state post-advance, fewer branches.

### 7. Memory references collected

| Memory | Relevance | Verdict |
|--------|-----------|---------|
| `mem.pattern.dispatch.reset-keep-cant-resync-already-advanced-ref` | Explains why `resync_worktree_hard` uses `reset --hard` (not `--keep`) and the None-leg desync mechanism | Supports: documents the hazard being deleted |
| `mem.pattern.dispatch.integrate-clean-trunk-or-phantom` | Warns about phantom index on dirty shared checkout | Mitigated by M4 pre-gate (preserved); resync removal doesn't affect this |
| `mem.pattern.dispatch.close-integrate-shared-trunk-race` | Documents the H2 shared-trunk-race (FF-only refusal under churn) | SL-157 preserves FF-only; this race is RFC-006's problem |
| `mem.pattern.dispatch.sync-tree-reads-ledger-not-worktree` | Sync must tree-read the journal — not relevant to advance leg | Informational only |

## Assessment

### Thesis correctness: CONFIRMED

The six claims in the thesis are each supported by the evidence:

1. **R1/R3/R4 live in `advance_pure_ref:1842-1848`.** ✓ Code inspection confirms the
   post-CAS re-probe + resync is the sole locus of RacedDesync (R1) and the
   `resync_worktree_hard` call (R3/R4).

2. **The None→Some race cannot occur.** ✓ Both AGENTS.md and the project orientation
   memory independently confirm `main` is never checked out; `edge` is always
   checked out. `worktree_for_ref(main)` → always `None`.

3. **The checked-out leg is safe and load-bearing.** ✓ `advance_checked_out` uses
   `ff_advance_in_worktree` (atomic `git merge --ff-only`); proven by the e2e
   regression test `integrate_trunk_checked_out_ff_leaves_clean_tree`.

4. **Deleting the resync dissolves R1/R3/R4.** ✓ The resync is the sole mechanism
   for all three risks. IMP-122's F-1/F-2 hardenings are unnecessary once the
   resync is deleted.

5. **FF-only preserved.** ✓ The deletion touches only the post-CAS re-probe within
   `advance_pure_ref`. The `advance_row` classification (`current == planned` →
   no-op; `current != expected_old` → moved; else advance) and the non-FF refusal
   in `advance_checked_out` are untouched.

6. **RFC-006 B split is disjoint.** ✓ RFC-006 touches `plan_trunk_row` (plan-time);
   SL-157 touches `advance_pure_ref` (advance-time). No shared code paths.

### Verification posture: ADEQUATE

The five e2e tests (including the two key VT-1/VT-2 tests at lines 962 and 1000)
will stay green unchanged. The only removed test is the `resync_worktree_hard`
unit test (which tests the deleted function). No new tests are needed — the
behaviour change is deletion, not addition; the existing tests already cover the
surviving paths.

### Risks: NONE IDENTIFIED

No counter-evidence found. The thesis is internally consistent and externally
aligned with all governance documents, project invariants, and code.

### Open questions resolved

- **OQ-A** (no main worktree to drop): Confirmed. `main` is already bare-ref.
- **OQ-B** (edge rides checked-out leg): Confirmed. AGENTS.md mandates it.
- **OQ-C** (no conflict surgery in A): N/A for this slice.
- **OQ-D** (which fns to keep/delete): Confirmed. `resync_worktree_hard` → delete;
  `ff_advance_in_worktree` → keep. Sole-caller property verified via grep.
- **OQ-E** (ADR-012 Revision is mechanism-only): Confirmed. No governance reversal;
  properly routed via ADR-013.

### Go / no-go

**Go.** The thesis is correct. The evidence is comprehensive and consistent.
No counter-evidence was found. The slice scope is precise and the A/B split
with RFC-006 is structurally sound. Ready for `design.md`.
