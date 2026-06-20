# SL-127 — implementation notes

Durable record for audit/close harvest. (Runtime scratch lives in
`state/.../phases/`; this file survives.)

## Dispatch drive (claude arm, serial, 2026-06-20)

Five phases implemented via `/dispatch` claude-arm workers (user override of
`claude-force-subprocess-dispatch=true` — correctness-critical, accepted the
bootstrap hazard knowingly). Each funneled green (import → verify → branch-point
→ one commit → record), R-5 clean, single-commit deltas.

| Phase | Change | Files | Coord commit |
|---|---|---|---|
| 01 | freshest-descendant trunk ladder | `git.rs` | 562c059a |
| 02 | plan-presence refuse-gate at coordinate Create | `worktree.rs` | 1185b5a7 |
| 03 | `trunk_drift` extract + `refresh-base` verb | `dispatch.rs`,`main.rs` | fb067cec |
| 04 | drift diagnostics — conflict hint + RefreshBase guidance | `dispatch.rs` | 11c71147 |
| 05 | dispatch skill routes to `refresh-base` | `plugins/.../dispatch/SKILL.md` | f6e4b315 |

Refs at park: `dispatch/127` @ 47ebbe57 (= 5 phases + a refresh-base merge of 8
trunk commits + prepare-review journal commits); `review/127` @ a325174e (bundle
on fresh base); `main` @ ca6228fe.

## Live dogfood (the validation that matters)

Mid-drive, 8 concurrent trunk commits landed on `main` (SL-126 close, SL-104,
ISS-011), touching `git.rs` + `dispatch.rs` — the exact RSK-010 base-divergence
this slice fixes. Drove the **new** binary's own fixes against it:

- **PHASE-01** — new ladder resolved `trunk=main` and reported the 8-commit drift
  with **no `DOCTRINE_TRUNK_REF` prefix** (the old binary requires it). ISS-036
  resolved in practice.
- **PHASE-03** — `dispatch refresh-base --slice 127` merged the 8 commits
  **cleanly** (drift hunks disjoint from ours within git.rs/dispatch.rs),
  advanced `dispatch/127`, `merge-base(dispatch,main) == main`, no trunk write,
  exit 0. RSK-010 remedy proven.
- **PHASE-04** — post-refresh, `next` guidance correctly dropped `RefreshBase`
  and pointed at prepare-review (drift==0).
- Merged tree (5 phases + 8 trunk commits): **2116 tests pass, clippy clean**.
- `review/127` delta = SL-127's 5 files only; trunk's 8 commits sit in the
  merge-base, not the bundle.

## Audit fodder (open items for /audit → reconcile)

1. **`prepare-review` cut `review/127` but no `phase/127-NN` refs** despite 5
   well-formed boundaries in `.doctrine/dispatch/127/boundaries.toml`. Confirm
   whether claude-arm per-phase deliverable cuts are expected here or a defect.
2. **EX-2 / VT-1 (retire `DOCTRINE_TRUNK_REF=main`) was vacuous** — the env-prefix
   ritual was never in the skill masters (only in backlog/memory/other-slice
   notes). PHASE-05 reframed it as confirmed-absent. Reconcile: the
   `DOCTRINE_TRUNK_REF=main` **memories** (mem_019ee083…, mem_019ee3c4…,
   mem_019ec912…) want updating once SL-127 integrates to trunk — until then the
   installed binary still has the bug, so they stay valid. Do NOT prune pre-close.
3. **PHASE-04 `select_guidance` refactor** wider than its task sketch — worker
   extracted the inline `next_guidance` chain into a pure fn taking precomputed
   git facts. Ordering/outcomes preserved (existing status tests green). Flagged
   by the worker; confirm at audit.

## Conclude-resume procedure (when main settles)

Coordination worktree LEFT UP at `.dispatch/SL-127` (no teardown). Use the
**jail-built** binary `/home/david/.cargo/doctrine-target-jail/debug/doctrine`
(has `refresh-base`; the PATH `doctrine` is the OLD binary without it). Steps:

1. `<newbin> dispatch status --slice 127` (no env prefix — PHASE-01 ladder).
2. If `trunk: moved` → ensure coord tree clean, then `<newbin> dispatch
   refresh-base --slice 127`, then **re-run** `<newbin> dispatch sync
   --prepare-review --slice 127` (review/127 re-pins to the fresh base). If
   refresh-base conflicts → resolve in the coord tree, commit, re-prepare.
3. If `trunk: stable` → review/127 is current; proceed.
4. `slice status 127 audit` → `/audit` from session root.
5. Stage-2 integrate is `/close`'s job, post-audit — never pre-audit.

Caveat: `refresh-base` refuses a dirty coord tree; the park state has an
uncommitted `.doctrine/dispatch/127/journal.toml` change (prepare-review
bookkeeping) — clean/settle it before re-refreshing.

## Conclude-resume EXECUTED (2026-06-20, post SL-104/126 close)

Trunk settled: `main` advanced `ca6228f → 1b00c46` (+4: notes-SL-127 `af88ce84`,
SL-104 trio `9e64cb3a`/`b8c303fa`/`1b00c46`). Source-disjoint from SL-127's delta
(`estimate.rs`/`value.rs`/`e2e_estimate` vs `dispatch`/`git`/`main`/`worktree.rs`)
— clean merge, no conflict.

Steps run (all from coord tree `.dispatch/SL-127`, jail binary):
1. `refresh-base --slice 127` → merged 4 trunk commits, dispatch tip `608d4cb`.
2. `sync --prepare-review` refused first: stale `review/127 @ a325174e` (old base)
   — the never-clobber guard. Resolved by `git branch -D review/127` then re-ran.
3. Re-prepared: `review/127 @ e25adbd`, base = `1b00c46` = main; dispatch `968b984`;
   `trunk: stable`. Bundle now reflects current trunk.

**GOTCHA — stale shared jail-target binary.** First drift check reported
`trunk: stable` falsely. Root cause: the shared build target
`/home/david/.cargo/doctrine-target-jail/debug/doctrine` had been overwritten by a
`cargo build` on `main` (OLD code, pre-PHASE-01 ladder), so it ran the plain
origin/HEAD-first ladder and measured drift against stale `origin/HEAD=ca6228f`
instead of the freshest-descendant pick (local `main`). **Always rebuild the binary
from the coord tree (`cd .dispatch/SL-127 && cargo build`) before trusting dispatch
drift** — the handover's "reuse the jail-target binary" is unsafe when concurrent
agents build on main. Post-rebuild: correctly `moved (4 ahead)`. The fix itself
(`freshest_descendant`, `git.rs`) is sound — it overtakes an ancestor `origin/HEAD`
with local `main`, verified live here.

### Audit fodder (added)
4. **`0 phase cut(s)` confirmed on the fresh re-prepare too** — reinforces fodder
   #1 (no `phase/127-NN` despite 5 boundaries). Persistent, not a base artifact.
   Audit must decide expected-vs-defect for the claude arm.
