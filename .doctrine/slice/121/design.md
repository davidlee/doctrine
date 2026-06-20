# SL-121 Design — dispatch sync --integrate: worktree-aware clean exit + legible outcome

> Governed by ADR-012 (dispatch integration topology), ADR-006 (worktree posture,
> pure/imperative split), ADR-001 (module layering). Bundles ISS-022, ISS-030,
> IMP-078, **IMP-075** (journal-cycle extraction — §2.6).

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
| not checked out | `update_ref_cas(target, planned, expected_old)` | today's path; **post-CAS re-probe** (below) |
| checked out + **fast-forward** advance (`expected_old` is an ancestor of `planned`) | `git -C <wt> merge --ff-only planned` | ref+index+worktree together; §2.5 race guard |
| checked out + **non-ff** advance (edge/force) | **refuse** `integrate-nonff-checkout` | can't safely non-ff a live ref without `reset --hard` (data loss); refuse, don't desync |

After either mechanism, **assert `target_ref == planned`** (post-condition); a
mismatch is a `Moved`/raced outcome, not success.

**None-leg post-CAS re-probe (round-2 MAJOR).** `update_ref_cas` has no worktree
awareness: if a ref is checked out *after* `worktree_for_ref` returned `None` but
*before/at* the CAS, the CAS advances a now-live branch behind its index — exactly
the ISS-022/030 desync this slice exists to kill. So after a `None`-leg `Applied`,
**re-probe** `worktree_for_ref(target)`: if it is now `Some(wt)`, the worktree just
desynced — resync it (clean → `reset --keep planned`) or, if dirty, surface a
`raced-checkout-desync` warning in the report (the advance already happened; we
cannot un-advance). This is best-effort under a genuine race; see §7 for the honest
concurrency boundary.

**Decision-time classification, not a transactional CAS on the merge leg
(round-2 BLOCKER).** The classification (`current == planned` / `== expected_old`)
is exact at *decision time*, but only the `update_ref_cas` leg re-checks
`expected_old` **atomically** at write (git.rs:892). The `merge --ff-only` leg has
no equivalent atomic expected-old guard — between classify and merge a concurrent
writer could move the branch to an ancestor-of-`planned` (or to `planned` itself),
after which the merge + post-assert still pass and the row is marked `Verified`,
whereas today's `replay_ref` would report `Moved`. **The landed content is always
`planned`** (`merge --ff-only` advances only along `planned`'s ancestry; anything
off it refuses → `Moved`), so this is a **labeling** divergence, not a corruption,
and it requires a concurrent *identical* advance — vanishingly rare. We therefore do
**not** claim transactional CAS equivalence for the checked-out leg (§7); we claim
content-correctness + an accepted, tested compatible-race window.

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

### 2.6 Journal-cycle extraction (IMP-075) — the thin bracket

`prepare_review` (dispatch.rs:934) and `integrate` (dispatch.rs:1044) share a
byte-for-byte journal cycle: `commit_journal` (intent, parent `tip`) → per-row
apply loop → `commit_journal` (applied status, parent `journal_commit`) → collect
failures → report-or-bail (RV-030 F-7). Only the per-row apply differs:
`prepare_review` does zero-oid creation CAS (`update_ref_cas(…, ZERO_OID)`,
collects `stale`); `integrate` does 3-way replay (`replay_ref`, collects `moved`)
— and that apply is exactly where §2.2's worktree-aware mechanism branch lands.
SL-121 rewrites the integrate apply *and* this slice extracts the shared cycle, so
both edit the same body; folding them means the extraction is shaped by the new
apply in one pass rather than re-touching the cycle twice.

**Shape — the bracket owns the boilerplate; the per-row move is an injected
closure.** OQ-6 settled **thin** (compose around, not a fat generic bracket):

```rust
/// Per-row result the apply closure hands back. Transient REPORT data — NOT a
/// `JournalRow` field (the row schema, ledger.rs, carries only the oids + status;
/// both NoOp and Applied persist as `Verified`, so disposition cannot be recovered
/// from the row after the fact — codex plan review BLOCKER). The bracket collects
/// these and returns them; the *caller* renders §4 and the bail from the vec.
enum RowOutcome {
    Done { disposition: Disposition },   // success — caller maps to the §4 line
    Refused { token: String },           // semantic refusal/moved — journaled Failed
}
// `Disposition` grows by phase: PHASE-04 has { Created, NoOp, Applied } (today's
// match arms, verbatim — behaviour-pure); PHASE-02 ADDS integrate's worktree
// variants { AdvancedResynced, AdvancedPureRef, RacedDesync } to integrate's
// closure ONLY — the bracket signature is stable across both phases.

/// Journal the planned intent onto `coord_ref` BEFORE any external ref mutation,
/// apply each row via `apply`, then re-journal applied status (recoverability).
/// `apply` mutates `row.status`/`applied_new_oid` in place and returns a
/// `RowOutcome`. `Err` is reserved for fatal command/invariant failure (B3): the
/// recovery commit only runs if `apply` returns `Ok`, so a semantic refusal MUST
/// be `Ok(Refused{token})`, never `?` — else applied status never persists.
fn with_journaled_projection(
    root: &Path, tip: &str, tip_tree: &str, journal_path: &str, coord_ref: &str,
    journal: &mut Journal, message: &str,
    mut apply: impl FnMut(&Path, &mut Row) -> anyhow::Result<RowOutcome>,
) -> anyhow::Result<Vec<RowOutcome>> {
    let journal_commit =
        commit_journal(root, tip_tree, tip, journal_path, coord_ref, journal, message)?;
    let mut outcomes = Vec::with_capacity(journal.rows.len());
    for row in &mut journal.rows {
        outcomes.push(apply(root, row)?);
    }
    commit_journal(root, tip_tree, &journal_commit, journal_path, coord_ref, journal, message)?;
    Ok(outcomes)
}
```

**`apply` contract (codex §2.6 review — enforced, not just documented).** Because
the recovery `commit_journal` runs **strictly after** the loop
(dispatch.rs:1009/1137), a `?`-`Err` out of `apply` aborts *before* applied status
is recorded. Therefore `apply` MUST return `Err` **only for fatal operational
failure** (a git command/invariant breaking). Every **semantic per-row refusal**
sets `row.status` (`Failed`) — and `applied_new_oid` where meaningful — and returns
`Ok(RowOutcome::Refused{token})`, so the post-loop commit still durably records it
(B3). This binds SL-121's per-row integrate refusals routed *through the closure*: a
**non-ff-checkout** refusal (§2.2) and a **raced `Moved`** (§2.5) are
`Ok(Refused{token})`, never `?`. (The **whole-integrate dirty refusal**, §2.3, is
different: it bails **caller-side before the bracket**, so nothing is journaled —
an `Err`/bail there is correct and the `apply` contract does not apply.) **This
contract is load-bearing and must be pinned by a test** that inspects the committed
`dispatch/<slice>` journal after a refusal and asserts the row persisted `Failed`
(not lost to an early `Err`) — §6.

**Seam placement — the three integrate-only worktree pieces stay caller-side, do
not enter the bracket:**

- **Dirty pre-gate (§2.3)** runs in `integrate` **before** the call — consistent
  with M4 (before the first `commit_journal`, which the bracket now owns). A dirty
  refusal returns before the bracket starts ⇒ zero refs moved, incl.
  `dispatch/<slice>`.
- **Per-row advance (§2.2)** — incl. the None-leg post-CAS re-probe/resync and the
  ff-merge leg — **is** integrate's injected `apply` closure. `prepare_review`'s
  closure is its existing zero-oid-CAS match body, verbatim.
- **Report (§4)** is rendered **after** the bracket returns, from the collected
  `Vec<RowOutcome>` it hands back (NOT from `journal.rows` — disposition is not a
  row field; codex BLOCKER) — integrate-side; `prepare_review` ignores dispositions
  and keeps its `"N ref(s) created"` line, bailing on any `Refused` token.

**Why thin (not fat).** `prepare_review`'s targets (`review/<slice>`,
`phase/<slice>-NN`) are created under zero-oid CAS and are **never checked out**, so
worktree-awareness is dead weight there. A fat bracket owning probe/gate/resync
would (a) push integrate-only concerns into `prepare_review`, (b) widen the
behaviour-preservation surface (§6), and (c) contradict §2.3's whole-integrate
(not per-row) refuse. Thin keeps `prepare_review` behaviourally identical — its
closure is unchanged code — so its suite is the proof.

**Behaviour-preservation gate (ADR-006/006).** `prepare_review`'s existing suite
(`e2e_dispatch_sync` prepare path) stays green **unchanged** — the proof the
extraction is behaviour-pure for the non-integrate caller. `integrate`'s behaviour
*does* change (worktree-aware), so its evidence is the §6 new VTs, not the
green-unchanged gate.

**Layering (ADR-001).** `with_journaled_projection` is an engine-internal
higher-order helper in `dispatch.rs` (same module as both callers); no new
cross-module edge, no leaf↔command cycle. The closure runs `git::*` plumbing at
the impurity altitude integrate already occupies — pure layer untouched.

## 3. Tree-true verify (ISS-030)

Close SKILL 3a today: `git diff --stat refs/heads/main~1..refs/heads/main -- src/`
— reads a **ref range** (misses the index/worktree desync) and `main~1` is wrong
under a merge or multi-commit advance.

Replace with two tree-true assertions:

```bash
# (a) no phantom reverse-diff — TRACKED working tree matches HEAD (not path-limited):
git diff --quiet HEAD     # nonzero exit ⇒ DESYNC, do not proceed   (untracked: ignored)

# (b) delta genuinely landed (not a silent dry-run) — read the journal's planned tip:
#     the integrate REPORT (§4) shows the trunk row "advanced"; belt-and-braces:
planned=$(doctrine dispatch sync --slice <N> --show-journal-trunk-oid)   # see read-surface note
git diff --quiet "$planned" refs/heads/main     # equal ⇒ trunk holds the projected tip
```

**(a) — F5/M8.** The ISS-030 detector: a phantom reverse-diff makes the tracked
working tree ≠ `HEAD`. It must **not** be path-limited (SL-097's reverse-diff spanned
implementation files beyond any one dir). Scope (M8): `git diff --quiet HEAD` covers
the **tracked** tree, not untracked files — correct here, since the phantom
reverse-diff is staged/unstaged tracked content.

**(b) — round-2 MAJOR (read surface).** The first revision said "diff against the
admitted `close_target` OID," but **that OID has no stable command at close 3a**:
`candidate admit` prints it to stdout once (dispatch.rs:652) — not captured by the
skill — and `candidate status` shows only an abbreviated tip (dispatch.rs:733). So
(b) rests on a **real** read surface: primarily the **integrate report** (§4), whose
trunk-row disposition must read `advanced` (a silent dry-run — `--trunk` omitted —
would show nothing); and, as a scriptable belt, the **trunk row's `planned_new_oid`
read from the committed `dispatch/<slice>` journal** (tree-read, the
`sync-tree-reads-ledger-not-worktree` invariant). **Plan-gate:** if no
CLI surface exposes that OID, this slice adds a minimal read (e.g. a `sync
--show-journal-trunk-oid` flag or documented `cat-file` of the journal) — the close
skill must not depend on capturing transient admit stdout. Either way the stale
`main~1..main` ref-boundary form is removed.

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
integrate: refs/heads/main 3a1f9c2..9c2e7b1 (advanced+resynced)
integrate: refs/heads/review/121 (no-op)
integrate: 2 ref(s) replayed
```

Disposition ∈ `{ advanced+resynced, advanced+pure-ref, no-op }` for success.
Refusals are not success lines: `integrate-dirty-worktree` (§2.3, whole-integrate),
`integrate-nonff-checkout` (§2.2, a checked-out target needing a non-ff advance),
and a raced `Moved` / `raced-checkout-desync` (§2.5/§2.2) all surface as the
named-token error / post-loop bail.

**Disposition home (codex BLOCKER).** These dispositions are **not** recoverable
from a `JournalRow` (ledger.rs: oids + status only — NoOp and Applied both persist
`Verified`). They live in the transient `RowOutcome` the §2.6 `apply` closure
returns; the bracket collects `Vec<RowOutcome>`; integrate renders this report from
that vec after the bracket. The named tokens above are the exact strings the
implementation MUST emit — pinned by §6 VTs, not paraphrased.

**OQ-3 resolved:** stderr human line + the existing stdout ref-list. No `--json`
(integrate has none; adding one is out of scope).

## 5. Code impact

| Path | Change |
|---|---|
| `src/git.rs` | NEW pure porcelain parser + `worktree_for_ref` (M9 fixtures); NEW status-capturing `ff_advance_in_worktree` (symbolic-ref + clean re-check + post-assert per §2.5; returns an outcome, never bare `?` — B3). |
| `src/dispatch.rs` | NEW `with_journaled_projection` thin bracket (§2.6, IMP-075): commit-pre / per-row `apply` closure / commit-post / collect failures. `prepare_review` + `integrate` both refactored onto it; `prepare_review`'s closure = its existing zero-oid-CAS body (behaviour-pure). `integrate` (≈1044–1161): dirty gate **before** the bracket (§2.3/M4); the injected `apply` closure carries per-row exact-CAS classify + mechanism branch (§2.2) + non-ff-checkout refusal + None-leg re-probe; report rendered from the returned `Vec<RowOutcome>` **after** the bracket (§4). `find_coordination_worktree` → wrapper over `worktree_for_ref` (`Err`→`"(removed)"`, F4). |
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
- **VT — `apply` contract persistence (codex plan review):** on a semantic refusal
  routed through the closure (a `Refused` outcome — e.g. integrate non-ff-checkout,
  or a prepare-review stale ref), read the **committed** `dispatch/<slice>` journal
  (`git show dispatch/<slice>:.doctrine/dispatch/<slice>/journal.toml`, tree-read)
  and assert the row persisted `status = Failed` — proving the post-loop recovery
  commit ran, not an early `Err` abort (B3). One test per caller.
- **Behaviour-preservation (IMP-075, §2.6):** `prepare_review` refactored onto the
  thin bracket with its apply closure = today's zero-oid-CAS body ⇒ the
  `e2e_dispatch_sync` **prepare** path stays green **unchanged** — the proof the
  extraction is behaviour-pure for the non-integrate caller. (`integrate`'s
  behaviour intentionally changes; its evidence is the worktree VTs above, not this
  gate.)
- **Behaviour-preservation:** existing `find_coordination_worktree` /
  `gather_fork_worktree` / `e2e_dispatch_sync` suites green **unchanged**.
- **VH — close 3a:** the revised verify block catches a deliberately-desynced tree
  (manual or scripted check that (a) fails on a phantom diff).

## 7. Invariants & boundaries

- **Dirty-atomicity (the guarantee we add):** the §2.3 pre-mutation pass, placed
  **before the first `commit_journal`** (M4), means a target dirty *at gate time*
  refuses with **zero refs moved** — including the coordination ref
  `dispatch/<slice>`. This is the new invariant this slice establishes. It covers
  **pre-existing** dirt only; see the concurrency boundary below.
- **Concurrency boundary (honest scope — round-2).** This slice does not introduce a
  worktree-placement lock; under genuine concurrent writers on the target, three
  residual races remain, all **content-safe** (the tree never lands on anything but
  `planned`), differing only in labeling/cleanliness reporting:
  1. *Dirt introduced during/after a merge* — caught only post-advance, so it is a
     **raced failure after advance**, not "zero refs moved." (Not the pre-existing
     case the gate guarantees.)
  2. *Checked-out merge leg vs a compatible concurrent advance* — can mark
     `Verified` where `replay_ref` would say `Moved` (decision-time classify, no
     atomic expected-old on the merge; §2.2). Content is still `planned`.
  3. *None-leg ref checked out between probe and CAS* — re-probe + best-effort
     resync / `raced-checkout-desync` warning (§2.2); cannot be un-advanced.
  These are documented and **tested**, not silently tolerated. Eliminating them
  needs a real placement lock — out of scope (a follow-up if close ever runs under
  true multi-writer contention).
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
- **F2 — leg asymmetry** (→ §7): **superseded/RESOLVED by the round-2 §2.2 rewrite.**
  Classification is now the exact `replay_ref` predicate on *both* legs (not
  ff-derived), so the rewound-target asymmetry is gone; the only residual is the
  decision-time-vs-atomic-write race, accepted under §7's concurrency boundary.
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

### Round 2 (codex GPT-5.5, on the revised design)

One real gap, three invariant-honesty corrections, one implementability fix, one
stale-text — all integrated:

- **R2-BLOCKER → §2.2/§7.** Merge leg has no atomic expected-old recheck (the CAS
  leg does, git.rs:892). Dropped the "exact CAS equivalence" claim for the
  checked-out leg; content is always `planned`, only labeling can race under a
  concurrent *identical* advance. Accepted + tested (§7 boundary).
- **R2-MAJOR → §2.2.** `None`-leg `update_ref_cas` could advance a ref checked out
  *after* the probe → recreates the original desync. Added a post-CAS re-probe +
  best-effort resync / `raced-checkout-desync` warning.
- **R2-MAJOR → §7.** Dirty-clean guarantee covers *pre-existing* dirt only;
  concurrent dirt is a raced-failure-after-advance. Invariant narrowed.
- **R2-MAJOR → §3(b).** The admitted `close_target` OID has **no stable close-3a read
  command** (`candidate admit` stdout uncaptured; `candidate status` abbreviated).
  Rebased (b) on the integrate report + the journal's `planned_new_oid`; added a
  plan-gate to expose that OID via a real CLI surface if none exists.
- **R2-MINOR → §9.** Stale F2 "not unified" text corrected to RESOLVED.

R2 confirmed sound on: non-ff-checkout refusal (no real close path needs a
checked-out non-ff edge advance — close only drives `--trunk refs/heads/main`);
status/recoverability parity across legs.

Residual: concurrency races are documented + tested, not eliminated (no placement
lock — out of scope).

### IMP-075 fold (2026-06-20, post-lock re-design)

Architect loop proposal 0018 flagged IMP-075 (`with_journaled_projection`
extraction) as refactoring the **same `integrate` body** SL-121 rewrites; folded
in to avoid double-touching the cycle (see slice scope + §2.6). OQ-6 (extraction
seam) settled **thin bracket + injected apply closure** — the worktree-aware
pieces stay caller-side (dirty gate before, resync inside integrate's closure,
report after), so `prepare_review` is behaviourally unchanged and its suite is the
behaviour-preservation proof. IMP-102 / IMP-103 remain out-of-scope followups
(`related: SL-121`); §2.6/§3 leave the close-step-3a seam and `--help` surface in a
state those can later bolt onto without re-cutting. Adversarial pass on §2.6 below.

**§2.6 self-review:**
- *Closure capture vs the dirty gate ordering.* The gate must observe the **same**
  checked-out targets the bracket will mutate. Both read `journal.rows` after row
  planning; the gate iterates targets before the call, the bracket iterates the
  identical `&mut journal.rows` inside — no divergence, gate strictly precedes the
  first `commit_journal` (M4 preserved). ✓
- *`apply` returning `Refused{token}` vs `Err`.* The bracket collects each
  `RowOutcome` (a `Refused{token}` is a journaled `Failed`) and still runs the
  post-commit (durability); only a real command failure `?`-aborts — identical to
  today's split (B3). prepare_review's `stale` and integrate's `moved` are derived
  caller-side from the returned `Vec<RowOutcome>`. ✓
- *Does the bracket hide the `fresh`/row-append or candidate logic?* No — journal
  construction (read, append trunk/edge rows, `pending_journal`) stays caller-side
  **before** the bracket; the bracket starts at the first `commit_journal`. The
  caller still owns what to journal; the bracket owns the commit/apply/commit
  dance. Clean boundary, no leakage. ✓

**Codex pass on §2.6 (2026-06-20, GPT-5.5, read-only vs src/).** Verdict
**sound-with-fixes**. Claims 1/2/3/5 confirmed against source (identical
`commit_journal` arg shape modulo message+parent — dispatch.rs:978/1009/1097/1137;
only loop bodies differ — :991/:1110; all journal construction strictly before the
first commit — :977/:1061/:1077). One MINOR: bind the `apply` contract explicitly —
per-row semantic refusals must be captured outcomes (later refined to
`Ok(Refused{token})` by the plan-review blocker), `Err` reserved for fatal failure,
else a `?` aborts before the recovery commit (B3). **Integrated** into §2.6 (the
`apply` contract paragraph), with the §2.3 whole-integrate dirty refusal explicitly
exempted (it bails before the bracket). No blocker/major.

**Codex pass on the PLAN (2026-06-20, GPT-5.5, read-only vs design+src).** Verdict
**blocker** — one real design gap surfaced and fixed here:
- **BLOCKER (disposition home) → §2.6/§4.** `JournalRow` (ledger.rs) carries only
  oids+status; NoOp and Applied both persist `Verified`, so §4's disposition
  vocabulary cannot be read back from `journal.rows`. **Fixed:** `apply` returns a
  transient `RowOutcome { Done{disposition} | Refused{token} }`; the bracket
  collects `Vec<RowOutcome>`; the caller renders §4 from it. No journal-schema
  change. `apply` signature changed from `Result<Option<String>>` accordingly.
- The remaining findings (apply-contract has no proving VT; report vocabulary too
  vague; PHASE-03 EN over-states the PHASE-02 dependency; dirty-refusal VT too weak;
  PHASE-03 VA-1 unfalsifiable) are **plan-level** — addressed in `plan.toml`, not
  here. The §6 `apply`-contract persistence VT above is the design-level half.

Residual: none blocking. Ready for `/plan` re-lock (plan fixes below).
