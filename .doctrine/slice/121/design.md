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

**Behaviour-preservation gate (ADR-006):** the existing suites for both callers
must stay green **unchanged** — this is the proof the extraction is behaviour-pure.

### 2.2 Per-row advance

```
# pre-mutation gate (§2.3) runs first, then:
for row in planned_rows:
    match git::worktree_for_ref(root, &row.target_ref)? {
        None      => replay_ref(...)                              # today's CAS, unchanged
        Some(wt)  => ff_advance_in_worktree(root, &wt, &row.planned_new_oid)?
    }
    # set row.status + applied_new_oid + disposition from the outcome
```

`ff_advance_in_worktree` = `git -C <wt> merge --ff-only <planned_new>`. Its results
map **exactly** onto the existing `ReplayOutcome` — no new result type:

| `git merge --ff-only` result          | ReplayOutcome | journal status |
|----------------------------------------|---------------|----------------|
| "Already up to date."                  | `NoOp`        | Verified       |
| fast-forwarded                         | `Applied`     | Verified       |
| refused — `<new>` not a descendant     | `Moved`       | Failed         |

Why this is safe and sufficient:
- **Atomic:** `merge --ff-only` moves ref + index + worktree together — no desync
  window.
- **Refuses on dirt** on its own (belt to §2.3's suspenders).
- **Moved-target protection preserved:** integrate's `planned_new` is
  `expected_old`+delta, so a foreign trunk advance is not an ancestor of
  `planned_new` → ff-only refuses = `Moved`. Equivalent to the CAS exact-old guard.

### 2.3 Atomicity gate (refuse-on-dirty)

Before **any** ref mutation, one pass over the planned rows: for each whose target
is checked out (`worktree_for_ref` → `Some(wt)`), assert `gather_tree_clean(wt)`.
Any dirty → **refuse the whole integrate** with a named token
`integrate-dirty-worktree`, zero refs moved. Fail closed; never advance row 1 then
choke on row 2. (`merge --ff-only` would refuse per-row anyway; the pre-pass makes
the refusal atomic across rows.)

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

## 3. Tree-true verify (ISS-030)

Close SKILL 3a today: `git diff --stat refs/heads/main~1..refs/heads/main -- src/`
— reads a **ref range** (misses the index/worktree desync) and `main~1` is wrong
under a merge or multi-commit advance.

Replace with two tree-true assertions:

```bash
# (a) no phantom reverse-diff — WHOLE working tree matches trunk tip (NOT path-limited):
git diff --quiet HEAD     # nonzero exit ⇒ DESYNC, do not proceed

# (b) delta genuinely landed — projected code == trunk code:
git diff --stat refs/heads/candidate/<N>/close-001 refs/heads/main -- src/   # empty ⇒ landed
```

(a) is the ISS-030 detector (a phantom reverse-diff makes the working tree ≠ HEAD).
**F5:** it must **not** be path-limited — the SL-097 report showed reverse-diff
entries across implementation files, and a `-- src/` filter would miss a desync in
any other dir; `git diff --quiet HEAD` covers the whole tracked tree. (b) keeps the
`-- src/` filter because it is asserting the *code* projection specifically. With §2
the desync cannot arise, but the verify now *proves* it instead of reading a ref
blind, and (b) proves projection against the **tree**, not a `~1` boundary. Remove
the stale `main~1..main` form and its TODO's reliance on it.

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

Disposition ∈ `{ advanced+resynced, advanced+pure-ref, no-op }` for success;
`refused-dirty` surfaces as the §2.3 named-token error, not a success line.

**OQ-3 resolved:** stderr human line + the existing stdout ref-list. No `--json`
(integrate has none; adding one is out of scope).

## 5. Code impact

| Path | Change |
|---|---|
| `src/git.rs` | NEW `worktree_for_ref`; NEW `ff_advance_in_worktree` (or inline `merge --ff-only` mapping to `ReplayOutcome`). |
| `src/dispatch.rs` | `integrate` (≈1044–1161): pre-mutation dirty gate (§2.3); per-row branch in the replay loop (§2.2); per-row disposition + report (§4). `find_coordination_worktree` → delegate to `worktree_for_ref`. |
| `src/worktree.rs` | `gather_fork_worktree` → delegate to `worktree_for_ref`; `gather_tree_clean` reused at a worktree path (no signature change). |
| `plugins/doctrine/skills/close/SKILL.md` | step-3a verify → tree-true (§3). |

## 6. Verification

New/changed evidence:

- **VT — pure probe parse:** `worktree_for_ref` parses a known `worktree list
  --porcelain` fixture: ref present (→ path), absent (→ None), detached block (no
  `branch` line → skipped). Drives the extraction.
- **VT — advance dispatch:** unit/e2e — target **not** checked out → CAS path,
  ref moves, index untouched (no phantom diff). Target checked out + clean → ref +
  index + worktree all at new tip; `git status` empty (the ISS-022/030
  regression test). Target checked out + **dirty** → `integrate-dirty-worktree`
  refusal, **zero refs moved** (atomicity).
- **VT — outcome map:** ff-only already-up-to-date → `NoOp`; foreign trunk advance
  → `Moved`/Failed, integrate bails *after the loop* (earlier applied rows persist
  — F3); a re-run resumes idempotently (skips `Verified`, `NoOp`s at-target).
- **VT — report:** stderr carries per-row `old..new (disposition)`; stdout
  ref-list contract preserved (regression).
- **Behaviour-preservation:** existing `find_coordination_worktree` /
  `gather_fork_worktree` / `e2e_dispatch_sync` suites green **unchanged**.
- **VH — close 3a:** the revised verify block catches a deliberately-desynced tree
  (manual or scripted check that (a) fails on a phantom diff).

## 7. Invariants & boundaries

- **Dirty-atomicity (the guarantee we add):** the §2.3 pre-mutation pass means a
  dirty checked-out target refuses with **zero refs moved**. This is the new
  invariant this slice establishes.
- **NOT fully atomic on `Moved`/collision (F3 — honest scope).** The replay loop
  applies rows sequentially and bails *after* the loop on any `Moved` row (the
  existing `moved` vec), so a later `Moved` — or an untracked-collision ff abort
  (F1) — can leave **earlier rows applied**. This is **pre-existing** integrate
  behaviour, unchanged here, and is safe because replay is **idempotent**: a re-run
  skips already-`Verified` rows (`fresh` guard) and `NoOp`s a row already at its
  target. The recovery model is "re-run resumes," not "all-or-nothing." We do *not*
  claim transactional atomicity across rows; only the dirty pre-gate is atomic.
- **Leg asymmetry (F2 — accepted, documented).** The CAS leg refuses on
  `current != expected_old` (exact); the ff-only leg refuses on
  `planned_new not a descendant of current`. They diverge only on a **rewound**
  target (current is a strict ancestor of `planned_new` but ≠ `expected_old`):
  CAS refuses, ff-only would advance. Trunk rewind under a close is not a real
  scenario; accepted, not unified.
- **No new impurity altitude:** the advance was already an imperative side effect
  (`update-ref`); `merge --ff-only` is the same altitude, the pure layer is
  untouched (ADR-006).
- **Idempotent replay preserved:** a re-run of a partially-journaled integrate
  still skips already-`Verified` rows (the `fresh` guard) and `NoOp`s a row at its
  target — now true through both the CAS and ff-only legs.
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

Residual: none blocking. Ready for external adversarial pass or `/plan`.
