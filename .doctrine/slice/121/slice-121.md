# dispatch sync --integrate: clean exit state and legible outcome

## Context

`doctrine dispatch sync --integrate` (`src/dispatch.rs::integrate`, entry
`run_integrate`) projects a slice's journaled refs onto trunk under a
fast-forward CAS. It is the *only* place `--integrate` runs — close step-3a,
post-audit (see `plugins/doctrine/skills/close/SKILL.md`). Three reported defects
share this one operation; fixing them piecemeal would re-touch the same exit path
repeatedly, so they bundle.

This slice bundles three backlog items, all on the `sync --integrate` exit path:

- **ISS-022** — after advancing trunk, the **staging area (index)** is left in a
  stale reverse-diff state. The integrate's ref-CAS replay (`git::replay_ref`)
  and/or the surrounding sync projection mutates the index and does not restore
  it, leaving the working checkout dirty post-integrate.
- **ISS-030** — integrate leaves a **stale worktree**; relatedly, close step-3a's
  verify reads a **ref** (`git diff --stat refs/heads/main~1..main`) rather than
  the **tree**, so it can pass on a ref that does not reflect the actual projected
  tree. Aligns with the established invariant that sync sources the ledger from
  the branch tip tree, not the working filesystem
  (`mem.pattern.dispatch.sync-tree-reads-ledger-not-worktree`).
- **IMP-078** — integrate is **silent about its trunk/worktree outcome**. Success
  emits only `integrate: N ref(s) replayed` on stderr; the user cannot see *what*
  moved (trunk advanced? which OID? worktree state?) without inspecting refs by
  hand. IMP-078 declares `after: ISS-022, ISS-030` — it reports on the very state
  the other two fix.

One theme: **`sync --integrate` must leave clean repository state (index +
worktree) and legibly report what it did.**

## Scope & Objectives

In scope:

1. **Clean index (ISS-022).** Integrate leaves the staging area in the same state
   it found it — no residual reverse-diff. Root-cause whether `replay_ref` or the
   sync projection step touches the index, and restore/avoid it.
2. **Clean / consistent worktree + tree-true verify (ISS-030).** Integrate does
   not leave a stale worktree; close step-3a verification reads the projected
   **tree**, not just a ref boundary, so a green verify genuinely proves the code
   delta landed.
3. **Legible outcome (IMP-078).** On success, integrate reports the concrete
   trunk/worktree outcome — what ref(s) advanced and to which OID, and the
   resulting worktree disposition — not just a replay count.

Affected surface (concrete):

- `src/dispatch.rs` — `integrate` (≈1044–1161), `run_integrate` (≈131), the
  replay loop and stderr reporting (≈1108–1160), worktree-cleanup detection
  (`find_coordination_worktree` ≈1769).
- `src/worktree.rs` — `git worktree remove` path (≈1408), if worktree hygiene is
  implicated.
- `src/git.rs` — `replay_ref` / any index-touching plumbing.
- `plugins/doctrine/skills/close/SKILL.md` — step-3a verify (≈85), to read the
  tree not the ref.

## Non-Goals

- **IMP-103** (`--integrate --help` --trunk dry-run wording) — gated `after
  IMP-101` (`deliver_to` config, not landed); --trunk semantics may shift. Defer.
- **IMP-102** (close structural gate: refuse `done` when un-integrated) —
  close-side lifecycle gate, a different surface from the sync exit path.
- **ISS-024** (candidate create stray `.doctrine/slice/` dirs break
  corpus-scanner) — a different verb (`candidate create`), different failure.
- Re-deriving trunk ref / verify from `doctrine.toml [dispatch] deliver_to` — that
  is IMP-101's job; this slice keeps the mandatory `--trunk` stopgap.

## Summary

_(to be completed at close)_

## Follow-Ups

- IMP-103, IMP-102, ISS-024 remain open and out of scope (see Non-Goals).

## Open Questions

- **OQ-1** Is the stale index (ISS-022) caused inside `replay_ref` (ref-CAS
  shouldn't touch the index) or by the surrounding sync projection's
  index-application for orthogonal/reverse-diff classes? `/design` to root-cause.
- **OQ-2** Worktree hygiene (ISS-030): does integrate own worktree removal, or
  only report disposition? ADR-012 says stage-2 `--integrate` runs *after* the
  coordination worktree is removed — clarify what "stale worktree" refers to.
- **OQ-3** Outcome-report shape (IMP-078): stderr human line vs structured
  (`--json`) — match existing dispatch reporting conventions.
