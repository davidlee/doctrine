# SL-031 — Implementation notes

Durable notes for the dispatch orchestrator funnel slice. Runtime progress lives
under `.doctrine/state/`; this file carries findings that must survive close-out.

## PHASE-03 — `/dispatch` funnel skill + reconciliation tail

- **`/dispatch` authored** at `plugins/doctrine/skills/dispatch/SKILL.md`
  (placeholder overwritten). Orchestrator-sole-writer half only; it **composes**
  `/worktree mode=worker` (PHASE-02) rather than restating the worker loop or the
  spawn guards (C-II no-parallel-implementation). Funnel cadence is the strict
  per-batch D7 order; the R-5 `.doctrine/`-reject belt and the `branch-point-check`
  guard are marked non-droppable; `DOCTRINE_WORKER=1` documented as a self-armed,
  fail-open contract (C-I) with R-5 named as the real protection.
- **No attribution footer.** Unlike `/worktree` (adapted from
  `superpowers:using-git-worktrees`), the funnel prose is authored wholesale from
  this slice's design (D7 / R-5 / X-1..3 / file-disjoint batching); no prior-art
  prose was reused, so no NOTICE.md and no attribution comment — claiming a
  derivation would be false.
- **Embed refresh.** Touched `src/skills.rs` to force the RustEmbed re-embed, then
  `doctrine skills install --skill dispatch` refreshed the gitignored install copy
  (`.doctrine/skills/dispatch`) + relinked `.claude/skills/dispatch`. Source of
  truth stays `plugins/`.

## Reconciliation (for `/close`)

- **IMP-002 — DONE in-phase.** `backlog edit IMP-002 --status resolved --resolution
  done`. Substance shipped under **SL-032** (D2a worker-mode guard `DOCTRINE_WORKER=1`
  + `tests/e2e_worker_guard.rs`; D3 trunk-ref minting; validate + reseat); the only
  SL-031 residue was the 5 `&[]` minting placeholders, wired in **PHASE-01**. The
  backlog item was stale-open; this closes the wiring tail. The original A-1
  ("blocked until IMP-002 lands") is retired — SL-031 was never execution-blocked.
- **IMP-003 — `/close` lands the flip.** "Dispatch worktree creation: detection and
  creation paths with guards" is realised across **SL-029** (the worktree *lifecycle*:
  detection D1, creation ladder D5/D9, `provision`/`check-allowlist`, solo
  `/worktree`, commit-before-spawn + single-tree branch-point) and **SL-031** (the
  worker-mode funnel built on it). Proposed transition: `backlog edit IMP-003
  --status resolved --resolution done` at close.
  - **Backlog→slice graph edge DEFERRED (C-VII).** Relations are v1-empty; there is
    no command to link a backlog item to its realising slice(s). The realisation is
    recorded here in prose, not as a stored edge.
  - **OQ-1 — resolved-defer.** The WorktreeCreate-hook half of the OQ-1 split is
    deferred (design §6): in the funnel the orchestrator provisions before the worker
    exists (D9), so the gap the hook would close is unreachable; the hook is
    Claude-only and never dependable. Recorded resolved-defer.
