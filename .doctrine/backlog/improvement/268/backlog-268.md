# IMP-268: Guard arm-spawn default-base to dispatch branch

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

SL-199 chose option (b) for the confined arm's base sourcing: `dispatch
arm-spawn` defaults `--base` to the coord-root `HEAD` when omitted, removing the
last per-spawn LLM responsibility for base selection (F13 determinism). The
accepted worst case of (b) is a **late** miss: a wrong base is only reachable
off-arm (main thread run from a cwd resolving to a different doctrine root) and
fails loud downstream at `worker_commit`'s `C^==B` precondition — a halted phase,
no security regression, but a *later* catch than option (a)'s exit-2-on-omission.

## The improvement (b+)

Have `run_arm_spawn` assert the resolved root is on a `dispatch/<NNN>` branch
before defaulting `--base` to its `HEAD` — refuse the default otherwise. Restores
loud-**early** failure for the wrong-root case at ~zero cost. Explicit `--base`
is unaffected (an explicit value always wins).

## Trigger

Do this **if token waste is observed** as a result of the late miss — i.e. if a
wrong-root arm actually reaches `worker_commit` and burns a halt/retry cycle in
practice. Until then the downstream belt is sufficient and this is not worth the
engine churn.

Origin: SL-199 PHASE-05 design delta (base-sourcing decision).

## Trigger satisfied, then done (2026-07-29)

**Why it fired now.** The trigger asked for observed token waste from the late
miss. It arrived via the **sibling defect in the same function**: **IMP-331** (a
half `--slice`/`--phase` arm) went undetected until `worker_commit` refused
`unprovable-fork` at hand-back — **~265k tokens and ~28min** after the fact
(SL-231 PHASE-04; RFC-011 case-notes `[dispatch/phase-plan/execute;
SL-231-PHASE-05-orchestrator-inline]`). Its recorded conclusion was *"the durable
fix is failing closed at arm time, which converts a worker-turn loss into a free
error"*, and that arg-shape gate landed. The **base-provenance** gate — this item
— was the remaining member of the same class, in the same function, with the same
remedy already accepted. Fixed rather than waiting for the identical accident to
recur on the other argument.

**Shape delivered** — pure `classify_default_base_root(branch, linked,
bound_slice)` in `dispatch`, called from `run_arm_spawn`'s default path before any
arming file is written (IMP-331's ordering lesson). Two distinct refusals, because
they have different fixes:

- `default-base-off-coord` — the root classifies `primary` or `fork`, so its HEAD
  is a moving trunk or a worker's tip, never B.
- `default-base-wrong-slice` — right role, wrong slice: the coord tree belongs to
  another slice, so its HEAD is *that* slice's B. **Beyond this card's original
  ask**, taken because `binding` is already classified before the base resolves,
  so the cross-check was free. One coord tree per slice ⇒ a mismatch is always an
  error.

Both refusals name the explicit-`--base` escape. The guard is **default-only**, as
this card specified — an explicit base stays ungated from any root.

**DRY note.** Rides `classify_worktree_role` instead of re-spelling the coord
shape test; coord and worker-fork branches share the `dispatch/` prefix, so a bare
prefix match misreads every fork as a coord. Extracted `coord_branch_suffix` in
`worktree::shared` as the single home for that rule (STD-001) and added
`coord_branch_slice` over it for the slice cross-check; `classify_worktree_role`
now delegates to the same helper, behaviour unchanged and its existing test green.

**One existing test modified, deliberately.**
`arm_spawn_defaults_base_to_head_when_omitted` asserted *"defaulted base is the
coord-root HEAD"* while running from a plain non-linked repo — i.e. it asserted the
exact permissiveness this item removes. Migrated to a real linked `dispatch/<NNN>`
fixture (`coord_repo`), so it now proves what its name always claimed. No
assertion was dropped; the explicit-base leg moved to its own test
(`arm_spawn_honours_an_explicit_base_off_the_coord_tree`) which pins the
default-only boundary from a non-coord root.

**Verification.** Pure truth table over all role × binding combinations; three
shell tests (off-coord refusal with nothing armed, wrong-slice refusal with
nothing armed, explicit-base admitted off-coord); `coord_branch_slice` unit test
including the documented digits-too-large corner where shape says coord but no id
parses (treated as "cannot compare", never as a mismatch). Full suite 4187 green,
`doctrine check gate` exit 0, zero clippy warnings.

Commit: `fe0ecf13`.
