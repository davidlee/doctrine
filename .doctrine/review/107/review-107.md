# Review RV-107 — reconciliation of SL-121

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject & surface reviewed.** SL-121 — worktree-aware `dispatch sync --integrate`
clean-exit + legible outcome (bundles ISS-022, ISS-030, IMP-078, IMP-075). The
dispatch arm was **abandoned mid-slice** (ISS-034: `isolation: worktree` forked a
wrong/moving base under shared-clone lock contention); PHASE-02/03 were finished
**inline** in the coordination worktree `.dispatch/SL-121` on branch `dispatch/121`.
The reviewed code surface is therefore the **source delta `587d4403..21d0a691`**
(the 6 SL-121 commits), linear on the fork base. Tests were run on that tree in
`.dispatch/SL-121`. This ledger is driven from the **parent tree on `main`**
(governance home; review baton-verbs are refused on the fork — IMP-024). Slice
authored governance (design/plan/scope) already lives on `main`.

**Probes (lines of attack).**

1. **Exact-CAS classification preserved (B1/B2).** The worktree-aware leg must swap
   only the *mechanism* of an advance, never the verdict. Confirm `advance_row`
   classifies on the exact `replay_ref` predicate (`current==planned`→NoOp;
   `current!=expected_old`→Moved/Failed; else advance) and that `merge --ff-only`
   never re-verdicts (no "already up to date" masking a `Moved`; edge/creation rows
   not ff-routed).
2. **Captured outcomes, never bare `?` (B3).** Every semantic refusal (non-ff
   checked-out, raced merge, moved CAS) sets `row.status=Failed` and returns a
   captured `RowOutcome`/`FfAdvance` — never a `?`-`Err` that aborts before the
   post-loop recovery `commit_journal` makes status durable.
3. **Dirty pre-gate placement (M4).** The dirty pass runs **before** the first
   `commit_journal` (which the bracket owns and which advances `dispatch/<slice>`),
   so a dirty target refuses with **zero refs moved incl. the coordination ref**.
4. **None-leg post-CAS re-probe (§2.2).** After a pure-ref advance, re-probe; a ref
   checked out in the probe→CAS window resyncs (clean) or warns `raced-checkout-desync`
   (dirty) — never silently desynced.
5. **Behaviour-preservation of `prepare_review` (IMP-075/ADR-006).** The thin
   `with_journaled_projection` bracket must leave `prepare_review` behaviourally
   identical; its existing `e2e_dispatch_sync` suite is the proof (green unchanged).
6. **Report contract (§4).** stdout ref-list byte-for-byte preserved; stderr
   per-row disposition additive.
7. **Tree-true verify + read surface (§3/OQ-5).** close SKILL 3a is whole-tracked-tree
   (`git diff --quiet HEAD`, not path-limited) + a stable journal-tree read
   (`--show-journal-trunk-oid`, not transient admit stdout).

**Known design-vs-impl deltas to disposition (carried from execution).**
- §2.2 names `reset --keep`; the impl uses `reset --hard` (`resync_worktree_hard`) —
  empirically forced (HEAD already moved under the ref), documented in code; design
  fold owed at reconcile.
- §5 said "reuse `gather_tree_clean`"; the impl extracted a leaf-level
  `git::tree_clean` predicate and `gather_tree_clean` delegates — a layering
  improvement, design text diverges.

**Standing closure risk (not a code defect).** `dispatch/121` is based on the old
fork base `587d4403`; `main` has since diverged on **orthogonal** files
(`boot.rs`/`.pi`/specs — none on the SL-121 surface). `close --integrate --trunk`
will **correctly refuse** the non-ff trunk advance (the engine reports, never
clobbers); close must rebase the `close_target` onto current `main` and re-admit
before integrate accepts. Flagged for the reconciliation brief.

## Synthesis

**Verdict: conformant and closeable.** The implementation faithfully realises the
twice-codex-hardened design. The worktree-aware advance branches *mechanism* only —
`advance_row` classifies on the exact `replay_ref` predicate (NoOp / Moved / advance)
and `merge --ff-only` never re-verdicts (B1/B2); every semantic refusal is a captured
`RowOutcome`/`FfAdvance`, never a bare `?`-`Err` (B3); the dirty pre-gate sits before
the first `commit_journal` so a dirty target refuses with zero refs moved incl.
`dispatch/<slice>` (M4); the report preserves the stdout ref-list byte-for-byte and
adds stderr disposition detail (§4); close SKILL 3a is whole-tracked-tree + a stable
journal-tree read (§3/OQ-5). `prepare_review` rides the extracted thin bracket
behaviour-pure (IMP-075). Evidence is strong: clippy clean, **2005 bin units green**,
**24/24 `e2e_dispatch_sync`** covering every phase EX/VT.

**Closure story.** Six findings, all verified terminal, **no blockers**. Three are
the codex external pass on the residual-concurrency paths; three are
execution-surfaced deltas. Of the six, four route per-slice design/text corrections
to reconcile (F-1, F-2, F-4, F-5), one is a tolerated documented boundary (F-3), and
one is a close-mechanics prerequisite (F-6).

**Standing risks consciously carried.**
1. **§7 "content-safe" was overstated for the None-leg `reset --hard` resync**
   (F-1/F-2). The slice's *single-writer close purpose* — kill the phantom
   reverse-diff — is fully met and tested; the gaps live only on a vanishing race
   (a ref becoming checked out *after* the CAS) and are owned as IMP-122 + a §7
   honesty correction. Accepting them is a conscious tradeoff, not an oversight: the
   broad fix is a worktree/placement lock the locked design already scoped out.
2. **The ff-merge leg's check-then-act guard** (F-3) is the same documented §7
   boundary; the post-merge assert keeps the *target* row honest.
3. **Integration topology** (F-6): the slice lives on a fork off the old base
   `587d4403`; the engine will *correctly refuse* a non-ff trunk at close — a
   procedural rebase prerequisite, not corruption.

**Tradeoffs accepted.** `reset --hard` over the design's named `reset --keep` (F-4 —
the only primitive that fixes an already-HEAD-moved desync); a leaf-level
`git::tree_clean` over "reuse `gather_tree_clean`" (F-5 — a layering improvement).
Both are the *code* being right and the *design text* owing a fold.

## Reconciliation Brief

### Per-slice (direct edit)
- **design.md §7** (from F-1, F-2): the "all residual races are content-safe (the
  tree never lands on anything but `planned`)" claim is **overstated for the None-leg
  `reset --hard` resync**. Correct it: (a) the resync can force a live branch back to
  `planned`, *clobbering a concurrent post-CAS advance*, while reporting success
  (F-1); (b) `reset --hard` overwrites an untracked file colliding with a tracked
  path in the target tree — silent data loss, *asymmetric* with the ff-merge leg's
  safe abort (F-2). State both as known, owned (IMP-122), vanishing-likelihood race
  gaps under single-writer close — not "content-safe."
- **design.md §2.2 + OQ-2** (from F-4): replace the None-leg `reset --keep` with
  `reset --hard`, with the rationale: after `update_ref_cas` the live HEAD already
  resolves to `<oid>`, so `reset --keep`/`--merge` see no diff and leave the desync;
  `reset --hard` under the prior `tree_clean` precondition is required. (Already in
  the `resync_worktree_hard` doc-comment; fold into the design.)
- **design.md §5** (from F-5): the code-impact table should record the extracted
  leaf-level `git::tree_clean` predicate (shared by the dirty pre-gate, the §2.5
  re-check, and `worktree::gather_tree_clean` as a delegating wrapper) rather than
  "reuse `gather_tree_clean` … no signature change."

### Close procedure (not a governance edit — a /close prerequisite)
- **F-6 — RESOLVED by consolidation (post-audit).** The abandoned-dispatch fork was
  consolidated by **cherry-picking the 7 SL-121 code commits directly onto current
  `main`** (clean — main never touched `src/{dispatch,git,worktree,main}.rs`,
  `close/SKILL.md`, `e2e_dispatch_sync.rs` since `587d4403`; gate green on main:
  clippy + 24/24 e2e_dispatch_sync + 2037 bin units). **The code is therefore
  already integrated on `main`.** `/close` must **NOT** run `dispatch sync
  --integrate` (the dispatch path is moot — dispatch was abandoned per ISS-034); the
  non-ff trunk refusal F-6 warned about no longer applies. Close = confirm the
  rollup (`⚠` clears once authored status agrees), spec-coherence, harvest, final
  commit. The `dispatch/121` branch is retained as immutable evidence.

### Governance/spec (REV)
- _None._ No ADR / tech-spec / requirement change is owed; all corrections are
  per-slice design-text folds.

### Follow-up work (captured)
- **IMP-122** (related SL-121): harden the None-leg post-CAS resync — re-resolve
  `target_ref` before `reset --hard`; guard untracked collisions. Closes the F-1/F-2
  code gaps; the broad §7 placement-lock remains the larger out-of-scope follow-up.

## Reconciliation Outcome

Reconcile pass complete (RV-107 consumed). All 6 findings terminal at entry
(F-1/F-2/F-4/F-5/F-6 `verified`, F-3 `tolerated`). 3 per-slice design-text folds
written to `slice/121/design.md`; 0 REV (no governance/spec edit owed).

### Direct edits applied (design.md, SL-121)
- **§2.2** (RV-107 F-4): None-leg resync `reset --keep planned` → `reset --hard
  planned`, with rationale (post-`update_ref_cas` HEAD already resolves to `planned`,
  so `--keep`/`--merge` see no diff and leave the desync; `--hard` under the §2.3
  `tree_clean` precondition is the only re-syncing primitive). Code was already
  correct; this folds design text to match.
- **§5 code-impact table** (RV-107 F-5): `gather_tree_clean` row corrected from
  "reused at a worktree path (no signature change)" to record the extracted leaf-level
  `git::tree_clean` predicate that the dirty pre-gate, §2.5 re-check, and
  `gather_tree_clean` (now a thin wrapper) all delegate to — a layering extraction.
- **§7 concurrency boundary** (RV-107 F-1/F-2): removed the overstated blanket
  "three residual races … all content-safe." Items 1–2 remain content-safe; item 3
  (None-leg `reset --hard` resync) reframed as **NOT content-safe** — (a) can clobber
  a concurrent post-CAS advance while reporting success, (b) overwrites an untracked
  file colliding with a tracked path (silent data loss, asymmetric with the ff-merge
  safe abort). Stated as known, owned, vanishing-race gaps under single-writer close;
  code remediation owned by IMP-122.

### REVs completed
- _None._ Brief carried no governance/spec item; all corrections were per-slice
  design-text folds.

### Withdrawn / tolerated
- RV-107 F-3: `tolerated` at audit — ff-merge §2.5 check-then-act is the same
  documented §7 boundary; rationale in finding disposition. No reconcile write owed.

### Close prerequisite (carried from brief, not a reconcile write)
- F-6 resolved by consolidation: the 7 SL-121 code commits were cherry-picked onto
  `main` (clean). **Code is already integrated on `main`** ⇒ `/close` must **NOT**
  run `dispatch sync --integrate`. `dispatch/121` retained as immutable evidence.

Reconcile pass complete — handoff to /close.
