# dispatch sync --integrate: clean exit state and legible outcome

## Context

`doctrine dispatch sync --integrate` (`src/dispatch.rs::integrate`, entry
`run_integrate`) projects a slice's journaled refs onto trunk under a
fast-forward CAS. It is the **only** place `--integrate` runs — close step-3a,
post-audit (`plugins/doctrine/skills/close/SKILL.md`), invoked **inline in the
user's main session, which is checked out on `main`**. The coordination worktree
is already GC'd at `/dispatch` conclude, so by close there is no separate
checkout-free context. Three reported defects share this one operation.

**Root cause (one bug, three faces).** Integrate advances trunk by **pure ref-CAS**
(`replay_ref` → `git update-ref`). That is correct when nothing is checked out on
the target. But close runs it while the session worktree owns `main`: moving the
`main` ref out from under that live index+worktree leaves HEAD at the new commit
while the index/worktree still hold the old tree → git renders **the inverse of
the landed delta as staged changes** (the "phantom reverse-diff"). Confirmed by
the recording commits: ISS-022 (079955e0) *"trunk advanced correctly but staging
area carried reverse-diff entries … resolved with `git reset --hard`"*; ISS-030
(3bf46b16) *"--integrate advances the main ref but not the live index/worktree …
step-3a verify reads the ref so it misses the desync."*

- **ISS-022** — stale **index** (phantom reverse-diff) after the advance.
- **ISS-030** — stale **worktree** (same desync); and close step-3a's verify reads
  a **ref** (`git diff --stat main~1..main`) so it passes blind to the desync.
- **IMP-078** — integrate is **silent**: emits only `N ref(s) replayed`, never
  reports what advanced or that it just desynced the tree it stands in. IMP-078
  declares `after: ISS-022, ISS-030` — it reports on the very state they fix.

This is the `git-ref-vs-worktree-placement` hub (IMP-110 / ISS-029).

One theme: **`sync --integrate` must be worktree-aware — leave clean repository
state (index + worktree) under every placement, and legibly report what it did.**

## Scope & Objectives

The fix is a **worktree-aware advance** inside integrate. Per planned row, by the
target ref's checkout state:

- **not checked out anywhere** → raw `update-ref` CAS (today's path, unchanged).
- **checked out + tree clean** → advance the ref *and* resync that worktree's
  index+worktree to the new tip via git's own fast-forward primitive
  (`merge --ff-only` / `reset --keep`), atomically and refusing on conflict.
- **checked out + tree dirty** → **refuse the whole integrate before any mutation**
  with a named token (fail closed, atomic — never half-advance).

In scope:

1. **Clean index + worktree (ISS-022 + ISS-030).** Integrate never leaves a
   phantom reverse-diff: when the target is checked out, ref and tree move together
   (clean) or it refuses (dirty). No manual `reset --hard` ritual.
2. **Tree-true verify (ISS-030).** Close step-3a verification reads the projected
   **tree** (compare trunk tip tree to the admitted close_target tree), not a ref
   boundary, so a green verify genuinely proves the delta landed *and* the tree is
   in sync.
3. **Legible outcome (IMP-078).** On success, integrate reports per row: ref
   `old..new`, and the worktree disposition (`resynced` / `pure-ref` /
   `refused-dirty`) — not just a replay count.

Affected surface (concrete):

- `src/dispatch.rs` — `integrate` (≈1044–1161): a pre-mutation checkout/dirty gate,
  the replay loop, a post-advance worktree resync, and the stderr report
  (≈1108–1160). `run_integrate` (≈131).
- `src/git.rs` / `src/worktree.rs` — a "which worktree has ref R checked out + is it
  clean" probe (`git worktree list --porcelain` + dirty check), and the ff-only
  resync primitive. Reuse existing helpers where they exist (`gather_tree_clean`,
  `find_coordination_worktree`).
- `plugins/doctrine/skills/close/SKILL.md` — step-3a verify (≈85): tree-true
  comparison; drop reliance on `main~1..main`.

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

- **OQ-1** ~~Root-cause of the stale index~~ — **RESOLVED.** Pure ref-CAS advance
  of a checked-out `main` desyncs the live index+worktree (phantom reverse-diff).
  Confirmed from the recording commits. Not a `replay_ref`/projection index leak.
- **OQ-2** ~~Resync primitive~~ — **RESOLVED (design §2.2).** Exact `replay_ref`
  classification preserved on both legs; `merge --ff-only` is the *mechanism* for a
  checked-out clean fast-forward only; `update_ref_cas` otherwise; non-ff checked-out
  refuses. Dirty gate before the first `commit_journal`.
- **OQ-3** ~~Report shape~~ — **RESOLVED (§4):** stderr per-row human line + the
  existing stdout ref-list; no `--json`.
- **OQ-4** ~~Multi-row generality~~ — **RESOLVED (§2.4):** general per-row probe, no
  special-casing.
- **OQ-5** (plan-gate, design §3(b)): the close verify needs the trunk row's
  `planned_new_oid` / admitted `close_target` OID, which has **no stable CLI read** at
  close 3a today. `/plan` to decide the minimal read surface (a `sync` flag exposing
  the journal trunk OID, or a documented journal `cat-file`) — the close skill must
  not depend on capturing transient `candidate admit` stdout.
