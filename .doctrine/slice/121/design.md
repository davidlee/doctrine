# SL-121 Design — dispatch sync --integrate: worktree-aware clean exit + legible outcome

> Governed by ADR-012 (dispatch integration topology), ADR-006 (worktree posture,
> pure/imperative split). Bundles ISS-022, ISS-030, IMP-078.

## 1. Problem & root cause

`dispatch sync --integrate` (`src/dispatch.rs::integrate`, entry `run_integrate`)
is the **only** call site of `--integrate` — close step-3a, post-audit
(`plugins/doctrine/skills/close/SKILL.md`), run **inline in the user's main
session, which is checked out on `main`**. The coordination worktree is GC'd at
`/dispatch` conclude, so by close there is no checkout-free context.

Integrate advances trunk by **pure ref-CAS**: `replay_ref` → `git update-ref`,
for *every* journaled row, blind to checkout state. `update-ref` moves the `main`
ref but does **not** touch the live index/worktree that has `main` checked out.
HEAD then resolves to the new commit while index/worktree still hold the old tree
→ git renders **the inverse of the landed delta as staged changes** (the "phantom
reverse-diff").

Confirmed by the recording commits (the backlog bodies are empty):
- **ISS-022** (079955e0): *"trunk advanced correctly but staging area carried
  reverse-diff entries for SL-097 implementation files. Resolved with `git reset
  --hard`."* → stale **index**.
- **ISS-030** (3bf46b16): *"--integrate advances the main ref but not the live
  index/worktree, producing a phantom reverse-diff; step-3a verify reads the ref
  so it misses the desync."* → stale **worktree** + a verify that reads the ref.
- **IMP-078**: integrate emits only `N ref(s) replayed` — never reports what
  advanced or that it desynced the tree it stands in. (`after: ISS-022, ISS-030`.)

This is the `git-ref-vs-worktree-placement` hub (IMP-110 / ISS-029).

**One theme:** integrate must be **worktree-aware** — leave clean repository state
under every placement, and legibly report what it did.

### Decision record (design conversation)

- Considered: (A) refuse-on-checkout; (B) hand-rolled index resync
  (`read-tree`/`checkout-index`); (C) relocate close to a checkout-free context.
  **C was discarded** as incoherent: the *session* worktree owns `main` regardless
  of where the command runs, and the coordination worktree is already gone at
  close. **B-by-hand discarded** as a new corruption surface.
- **Chosen:** worktree-aware advance using git's own ff-only primitive — `merge
  --ff-only` for the checked-out leg (atomic, refuses on dirt), `update-ref` CAS
  for the not-checked-out leg, refuse-on-dirty as a pre-mutation gate. This is "B
  done through git's safe primitive," at the impurity altitude integrate already
  occupies (it already does `update-ref`).

## 2. Target behaviour — worktree-aware advance

The replay loop branches **per row** on the target ref's checkout state.

### 2.1 Shared probe (DRY)

`gather_fork_worktree`'s own doc states *"there is no existing branch→worktree-path
helper to reuse"* — yet `find_coordination_worktree` (dispatch.rs:1769) and
`gather_fork_worktree` (worktree.rs:947) are two copies of the same
`worktree list --porcelain` parse. This slice needs a third use ⇒ extract one
helper in `src/git.rs`:

```rust
/// The worktree path that has `refname` (e.g. "refs/heads/main") checked out,
/// or None if no live worktree does. Single parse of `worktree list --porcelain`
/// (blocks separated by blank lines; each has a `worktree <path>` line and, when a
/// branch is checked out, a `branch refs/heads/<name>` line).
pub(crate) fn worktree_for_ref(root: &Path, refname: &str) -> Result<Option<PathBuf>, CaptureError>
```

Refactor both callers onto it:
- `find_coordination_worktree` → wrapper mapping `None` → its `"(removed)"`
  sentinel. **F4:** the original folds *both* ref-absent **and git-command-failure**
  into `"(removed)"` (`let Ok(out) = … else return "(removed)"`); the new helper
  returns `Err` on git failure, so the wrapper must map `Err` → `"(removed)"` too,
  else the green-unchanged gate breaks. (`gather_fork_worktree` already propagates
  `anyhow::Error`, so it takes the `Err` straight through — no behavioural change.)
- `gather_fork_worktree` → delete the duplicate body, delegate.

**Parser divergence — pick deliberately (M9).** The two existing parsers are
**not** identical: `gather_fork_worktree` (worktree.rs:951) resets `current_path` on
a blank line; `find_coordination_worktree` (dispatch.rs:1774) does **not**. On a
well-formed `--porcelain` listing this never differs (a `branch` line always
precedes the next blank), but a shared helper must choose one rule. Adopt the
**block-reset** form (blank line clears state) — the more defensive parse — and
prove equivalence with fixtures.

**Behaviour-preservation gate (ADR-006):** extract a **pure** parser over the
porcelain text with fixtures — normal, ref-absent, **detached** (no `branch` line),
**blank-block**, and **command-failure** — and keep both callers' existing suites
green **unchanged**. That is the proof the extraction is behaviour-pure.

### 2.2 Per-row advance — classification preserved, mechanism swapped

**Corrected after external review (B1/B2/B3).** The first draft claimed
`merge --ff-only`'s own outcomes map onto `ReplayOutcome`. **That is wrong** and is
abandoned: `replay_ref` (git.rs:942) classifies on **exact** oids — `current ==
planned` → `NoOp`, `current == expected_old` → advance, else → `Moved` — while
`merge --ff-only` would (a) report "already up to date" when the target is *ahead*
of `planned` (CAS says `Moved`), and (b) fast-forward from a *rewound* ancestor that
isn't `expected_old` (CAS refuses). And edge/`review` rows are **explicitly not
ff-gated** (dispatch.rs `plan_edge_row` / `plan_candidate_edge_row`: *"Not ff-gated;
the CAS still guards"*) and creation rows carry `expected_old = ZERO_OID` — routing
those through `merge --ff-only` changes their semantics and could "repair" a
clobbered checked-out `review/NNN` that the suites require to refuse.

**The classification is therefore unchanged from `replay_ref`. The worktree-aware
leg swaps only the *mechanism* of the advance, never the verdict.** Per row:

```
# §2.3 dirty pre-gate has already run (before the first commit_journal).
for row in planned_rows:
    actual = resolve(target_ref)            # ZERO_OID if absent
    if actual == planned_new:   outcome = NoOp                      # exact, == replay_ref
    elif actual != expected_old: outcome = Moved{actual}           # exact, == replay_ref
    else:                       outcome = advance(row)             # actual == expected_old
    record(row, outcome)                    # status, applied_new_oid, disposition
# post-loop commit_journal (recoverability) unchanged; bail at end if any Moved
```

`advance(row)` — the only place the mechanism branches on checkout state:

| target checkout state | mechanism | notes |
|---|---|---|
| not checked out | `update_ref_cas(target, planned, expected_old)` | today's path, unchanged |
| checked out + **fast-forward** advance (`expected_old` is an ancestor of `planned`) | `git -C <wt> merge --ff-only planned` | atomic ref+index+worktree; §2.5 race guard |
| checked out + **non-ff** advance (edge/force) | **refuse** `integrate-nonff-checkout` | can't safely non-ff a live ref without `reset --hard` (data loss); refuse, don't desync |

After either mechanism, **assert `target_ref == planned`** (post-condition); a
mismatch is a `Moved`/raced outcome, not success.

**Status-capturing, not `?`-erroring (B3).** `advance` returns an *outcome*
(`Applied`/`Moved`/refusal), so a merge refusal or non-ff case sets the row to
`Failed` and is journaled by the existing post-loop `commit_journal` — exactly as
the CAS `Moved` path does today (dispatch.rs:1125/1137). `Err` is reserved for real
command/invariant failures. A naive `git merge … ?` (which `git_text` turns into an
`Err` on nonzero, git.rs:520) would abort *before* the row status is made durable —
forbidden.

Net: exact CAS semantics (B1) and edge/creation refusal semantics (B2) are
preserved; only a checked-out **clean fast-forward** row additionally syncs its
worktree, via git's own atomic primitive.

### 2.3 Atomicity gate (refuse-on-dirty)

One pass over the planned rows: for each whose target is checked out
(`worktree_for_ref` → `Some(wt)`), assert `gather_tree_clean(wt)`. Any dirty →
**refuse the whole integrate** with `integrate-dirty-worktree`, zero refs moved.

**Gate placement (M4).** "Before any ref mutation" must mean **before the first
`commit_journal`** (dispatch.rs:1097) — that call advances the coordination ref
`dispatch/<slice>` *before* any external replay. The gate runs right after row
planning (≈dispatch.rs:1093, where all targets are known) and before that journal
commit. Placed later, a dirty refusal would still leave pending intent committed on
`dispatch/<slice>` — violating "zero refs moved." `dispatch/<slice>` has no live
worktree at close (GC'd at conclude), so advancing it doesn't desync anything, but
it is still a ref mutation the gate must precede.

**The pre-gate is the dirty guarantee — not `merge --ff-only` (M5).** `merge
--ff-only` does *not* refuse on arbitrary dirt: git aborts only changes it would
**overwrite**; unrelated tracked dirt survives into a "dirty success." So the
cleanliness guarantee rests on this pre-gate (`git status --porcelain
--untracked-files=no` empty), re-checked immediately before/after the merge (§2.5),
**not** on any ff-only invariant.

`gather_tree_clean` (worktree.rs:711) currently takes `root`; it already shells
`git status` at that path, so it generalises to any worktree path with no change —
pass `wt`.

**Untracked-collision caveat (F1).** `gather_tree_clean` uses
`--untracked-files=no`, so an **untracked** file colliding with a projected path
passes this gate, then `merge --ff-only` aborts on its own ("untracked working
tree files would be overwritten"). That abort is a per-row failure, handled like
`Moved` (§2.2) — *not* silently swallowed, and surfaced with git's own message. We
deliberately do **not** widen the gate to `--untracked-files=normal`: untracked
noise in the close session is common and must not block a clean ff. Consequence:
an untracked collision on row N>1 can leave rows<N applied — folded into the honest
atomicity statement (§7), not the dirty pre-gate's guarantee.

### 2.4 Multi-row generality

No trunk special-casing. The probe is per-row; `edge` (`review/<slice>`) is rarely
checked out → `None` → CAS path. Generality is free once the probe exists.

### 2.5 Probe→merge race guard (M6)

`worktree_for_ref` is read before `advance` runs; between them the worktree could
detach or switch branches, after which `git -C <wt> merge --ff-only <oid>` would
advance a *detached HEAD* or a *different branch* — not `target_ref`. Guard inside
`advance`, in the target worktree, immediately before and after the merge:

- `git -C <wt> symbolic-ref --quiet HEAD` == `target_ref` (still attached to it),
- re-assert `gather_tree_clean(wt)` (the M5 window),
- post-merge: `target_ref` resolves to `planned` (the §2.2 post-condition).

Any mismatch → a raced `Moved` outcome (journaled `Failed`), never a silent
wrong-ref advance. (Single-writer orchestration narrows this, but close runs in a
live human/agent session, so the guard is not optional.)

## 3. Tree-true verify (ISS-030)

Close SKILL 3a today: `git diff --stat refs/heads/main~1..refs/heads/main -- src/`
— reads a **ref range** (misses the index/worktree desync) and `main~1` is wrong
under a merge or multi-commit advance.

Replace with two tree-true assertions:

```bash
# (a) no phantom reverse-diff — TRACKED working tree matches HEAD (not path-limited):
git diff --quiet HEAD     # nonzero exit ⇒ DESYNC, do not proceed   (untracked: ignored)

# (b) delta genuinely landed — trunk tip's tree == the ADMITTED close_target tree:
git diff --quiet <admitted_close_target_oid> refs/heads/main   # equal ⇒ landed
#   (display only: git diff --stat <admitted_close_target_oid> refs/heads/main)
```

**(a) — F5/M8.** The ISS-030 detector: a phantom reverse-diff makes the tracked
working tree ≠ `HEAD`. It must **not** be path-limited (SL-097's reverse-diff spanned
implementation files beyond any one dir). Scope note (M8): `git diff --quiet HEAD`
covers the **tracked** tree, not untracked files — correct here, since the phantom
reverse-diff is staged/unstaged tracked content; untracked noise is deliberately out
of the desync signal.

**(b) — M7.** Compare against the **admitted `close_target` OID**, *not* the mutable
`refs/heads/candidate/<N>/close-001` ref. Integrate targets the admitted OID
(`plan_candidate_trunk_row`, dispatch.rs:1234); the candidate ref can move, so a
diff against it can pass while the wrong tree landed. Diff trunk's tree to the
admitted OID's tree (no pathspec, or the actual payload paths) with `--quiet`;
`--stat` is display only. With §2 the desync cannot arise, but the verify now
*proves* it against the **tree** rather than the `main~1..main` ref boundary (which
was also wrong under a merge/multi-commit advance). Remove the stale form and its
TODO's reliance on it.

**Scope of this step.** §3 lives in the close SKILL and is therefore
**close-to-main-specific** (`refs/heads/main`, the `close_target` candidate).
The §2 *engine* is target-agnostic (see §7), so a future PR-style integrate flow
(integrate to a topic branch, push as a PR against main) reuses §2 unchanged but
would supply its **own** tree-true verify against its topic ref — it is not this
close step. SL-121 does not build that flow; it only keeps the engine general.

## 4. Legible outcome (IMP-078)

Today: success → `integrate: {N} ref(s) replayed` on stderr; Applied rows print
`row.target_ref` to **stdout** (the machine-readable changed-ref list).

**Preserve the stdout ref-list contract** (scripts may consume it). **Add**
per-row human detail on **stderr**, carrying a disposition derived from §2:

```
integrate: refs/heads/main 3a1f9c2..9c2e7b1 (advanced, worktree resynced)
integrate: refs/heads/review/121 (no-op, already at tip)
integrate: 2 ref(s) replayed
```

Disposition ∈ `{ advanced+resynced, advanced+pure-ref, no-op }` for success.
Refusals are not success lines: `integrate-dirty-worktree` (§2.3, whole-integrate),
`integrate-nonff-checkout` (§2.2, a checked-out target needing a non-ff advance),
and a raced `Moved` (§2.5) all surface as the named-token error / post-loop bail.

**OQ-3 resolved:** stderr human line + the existing stdout ref-list. No `--json`
(integrate has none; adding one is out of scope).

## 5. Code impact

| Path | Change |
|---|---|
| `src/git.rs` | NEW pure porcelain parser + `worktree_for_ref` (M9 fixtures); NEW status-capturing `ff_advance_in_worktree` (symbolic-ref + clean re-check + post-assert per §2.5; returns an outcome, never bare `?` — B3). |
| `src/dispatch.rs` | `integrate` (≈1044–1161): dirty gate **before the first `commit_journal`** (§2.3/M4); per-row exact-CAS classify + mechanism branch (§2.2); non-ff-checkout refusal; per-row disposition + report (§4). `find_coordination_worktree` → wrapper over `worktree_for_ref` (`Err`→`"(removed)"`, F4). |
| `src/worktree.rs` | `gather_fork_worktree` → delegate to `worktree_for_ref`; `gather_tree_clean` reused at a worktree path (no signature change). |
| `plugins/doctrine/skills/close/SKILL.md` | step-3a verify → tree-true (§3). |

## 6. Verification

New/changed evidence:

- **VT — pure probe parse (M9):** the extracted pure parser over fixed `worktree
  list --porcelain` text: ref present (→ path), absent (→ None), detached block (no
  `branch` line → skipped), **blank-block** (state reset), **command-failure**
  (`Err` → wrapper maps to `"(removed)"`). Drives the extraction; both callers'
  suites stay green unchanged.
- **VT — advance dispatch:** unit/e2e — target **not** checked out → CAS path,
  ref moves, index untouched (no phantom diff). Target checked out + clean ff → ref +
  index + worktree all at new tip; `git status` empty (the ISS-022/030 regression
  test). Target checked out + **dirty** → `integrate-dirty-worktree`, **zero refs
  moved incl. `dispatch/<slice>`** (gate before first `commit_journal`, M4). Target
  checked out + **non-ff** advance → `integrate-nonff-checkout`, ref untouched (B2).
- **VT — exact-CAS classification (B1):** target *ahead* of `planned` → `Moved`
  (not the ff "already up to date"); target == `planned` → `NoOp`; foreign advance
  off `expected_old` → `Moved`/Failed, bail *after the loop* (earlier rows persist —
  F3), re-run resumes idempotently (skips `Verified`, `NoOp`s at-target).
- **VT — race guard (M6):** probe says checked-out, HEAD detached/switched before
  merge → raced `Moved`, never a wrong-ref advance.
- **VT — report:** stderr carries per-row `old..new (disposition)`; stdout
  ref-list contract preserved (regression).
- **Behaviour-preservation:** existing `find_coordination_worktree` /
  `gather_fork_worktree` / `e2e_dispatch_sync` suites green **unchanged**.
- **VH — close 3a:** the revised verify block catches a deliberately-desynced tree
  (manual or scripted check that (a) fails on a phantom diff).

## 7. Invariants & boundaries

- **Dirty-atomicity (the guarantee we add):** the §2.3 pre-mutation pass, placed
  **before the first `commit_journal`** (M4), means a dirty checked-out target
  refuses with **zero refs moved** — including the coordination ref
  `dispatch/<slice>`. This is the new invariant this slice establishes.
- **NOT fully atomic on `Moved`/collision (F3 — honest scope).** The replay loop
  applies rows sequentially and bails *after* the loop on any `Moved` row (the
  existing `moved` vec), so a later `Moved` — or an untracked-collision ff abort
  (F1) — can leave **earlier rows applied**. This is **pre-existing** integrate
  behaviour, unchanged here, and is safe because replay is **idempotent**: a re-run
  skips already-`Verified` rows (`fresh` guard) and `NoOp`s a row already at its
  target. The recovery model is "re-run resumes," not "all-or-nothing." We do *not*
  claim transactional atomicity across rows; only the dirty pre-gate is atomic.
- **Classification asymmetry RESOLVED (was F2).** The §2.2 rewrite **removes** the
  asymmetry: rows are classified by the *exact* `replay_ref` predicate (`current ==
  planned` / `current == expected_old`) regardless of leg; `merge --ff-only` is only
  the *mechanism* of an already-classified advance, plus a post-condition assert
  (`target == planned`). The rewound-target case the first draft tolerated is now
  caught identically to CAS — `current != expected_old` → `Moved` — and a non-ff
  checked-out advance refuses rather than fast-forwarding.
- **No new impurity altitude:** the advance was already an imperative side effect
  (`update-ref`); `merge --ff-only` is the same altitude, the pure layer is
  untouched (ADR-006).
- **Idempotent replay preserved:** a re-run of a partially-journaled integrate
  still skips already-`Verified` rows (the `fresh` guard) and `NoOp`s a row at its
  target — true through both the CAS and the worktree-resync mechanisms.
- **stdout ref-list is a contract:** machine-readable; only additive stderr detail.
- **Target-ref-agnostic:** the advance keys on the target's *checkout state*
  (`worktree_for_ref`), never its *name*. `--trunk refs/heads/main` (close) and
  `--trunk refs/heads/feature/x` (integrate-to-topic-branch for a PR against main)
  traverse the **same** code: not-checked-out → CAS; checked-out-clean → ff-only
  resync (in whichever worktree holds it); checked-out-dirty → refuse. The ff-only
  precondition on the target is inherent integrate semantics (a *divergent* topic
  branch refuses as `Moved`), **pre-existing** and unchanged by this slice.

## 8. Out of scope / follow-ups

- IMP-103 (`--help` --trunk dry-run wording; gated on IMP-101 `deliver_to`).
- IMP-102 (close structural gate refusing `done` when un-integrated).
- ISS-024 (`candidate create` stray `.doctrine/slice/` dirs).
- `--json` integrate output; deriving trunk ref from `doctrine.toml deliver_to`
  (IMP-101).

## 9. Adversarial pass (internal)

Self-review before external challenge; findings integrated above.

- **F1 — untracked-collision** (→ §2.3): `gather_tree_clean`'s
  `--untracked-files=no` lets an untracked file collide; `merge --ff-only` aborts
  per-row. Accepted: surfaced as a `Moved`-class failure, gate deliberately not
  widened. Folded into §7 honest atomicity.
- **F2 — leg asymmetry** (→ §7): CAS exact-old vs ff-only descendant differ only on
  a rewound target. Accepted, documented, not unified.
- **F3 — atomicity overclaim** (→ §7, §6): original "atomic-or-nothing" was false
  for mid-loop `Moved`. Rewritten to the honest guarantee (dirty pre-gate atomic;
  `Moved`/collision idempotent-resume, pre-existing).
- **F4 — refactor behaviour drift** (→ §2.1): `find_coordination_worktree` swallows
  git failure into `"(removed)"`; wrapper must map `Err` → sentinel to keep the
  green-unchanged gate.
- **F5 — path-limited desync detector** (→ §3): `(a)` changed from
  `git diff --quiet refs/heads/main -- src/` to `git diff --quiet HEAD` (whole
  tracked tree) — the SL-097 reverse-diff was not src-only.

Residual: none blocking.

## 10. External adversarial pass (codex GPT-5.5)

Hostile review against the source; all nine findings confirmed against `src/` and
integrated. Summary:

- **B1 (blocker) → §2.2 rewrite.** `merge --ff-only`'s outcomes do **not** map onto
  `ReplayOutcome` (git.rs:942 classifies on exact oids). Abandoned the mapping;
  classification is now exact-CAS, merge is mechanism-only with a post-assert.
- **B2 (blocker) → §2.2.** Edge/`review` rows are *not ff-gated* and creation rows
  use `expected_old = ZERO_OID`; routing them through ff-only changed semantics.
  Now: exact classify; non-ff checked-out advance refuses (`integrate-nonff-checkout`).
- **B3 (blocker) → §2.2.** A merge refusal must be a captured *outcome* (journaled
  `Failed`), not a bare `?` `Err` (git.rs:520) that aborts before status is durable.
- **M4 → §2.3/§7.** Dirty gate must run **before the first `commit_journal`**
  (dispatch.rs:1097), which advances `dispatch/<slice>`.
- **M5 → §2.3.** `merge --ff-only` does *not* refuse on arbitrary dirt; the pre-gate
  is the dirty guarantee, re-checked around the merge.
- **M6 → §2.5.** Probe→merge TOCTOU; guard `symbolic-ref HEAD == target` + clean +
  post-assert in the target worktree.
- **M7 → §3(b).** Verify against the **admitted `close_target` OID**, not the mutable
  candidate ref (dispatch.rs:1234).
- **M8 → §3(a).** `git diff --quiet HEAD` is the **tracked** tree (untracked
  ignored) — correct for the phantom reverse-diff; wording fixed.
- **M9 → §2.1/§6.** The two existing parsers differ on blank-line reset; adopt the
  block-reset form; extract a pure parser with fixtures.

Ready for `/plan`.
