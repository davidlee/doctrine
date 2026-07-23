# Funnel false-red elimination

## Context

Descends from RFC-016 (zero-rescue dispatch), **Cluster 1 / move B** —
absorb mechanically-decidable recovery into the verbs and stop the funnel
showing the worker or orchestrator *environmental* red. Three of the RFC-011
case-notes' top burners share the theme "the funnel's gates and tree-state lie":

- **worker_commit gate false-red (6+ repros, ISS-218).** The gate shells
  `doctrine` from `$PATH` (stale nix-store binary), so any phase that changes
  conformance rules / allowlists / roles — even a *pure-JS* delta — gets a
  false-red the worker cannot distinguish from its own damage.
- **e2e authored-write goldens fail under the worker marker (7+ repros,
  CHR-044).** ~30+ e2e binaries spawn the CLI for authored writes, which the
  worker-mode guard correctly refuses under the marker; the worker reads it as
  own-delta red and burns tokens diagnosing.
- **coord worktree stale post-import (6 repros, GAP).** `dispatch_import` /
  `dispatch_conclude_phase` advance the coord branch ref via object-db only; the
  working tree and index stay at B, so every landed file shows as a staged
  deletion (reverse-diff). A pathless `git commit` here would commit mass
  reversions — the orchestrator must `git restore` after every funnel write.

Each burns tokens as diagnosis of noise, and #3 fixing the coord-tree footgun
**retires the checkout-import recovery memories** — the first move-E / OQ-6
memory-retirement, and the baseline the Cluster 2 memory-blind benchmark is
measured against.

## Scope & Objectives

1. **Gate runs the in-process / workspace binary, not `$PATH`.** worker_commit's
   `check commit` belt resolves doctrine to the server's own binary so gate
   truth matches the fork's actual rules (ISS-218; IMP-270 dup-closed here).
2. **Marker-aware skip for authored-write e2e goldens.** The e2e goldens that
   drive authored writes skip (or route around the guard) under the worker
   marker / `DOCTRINE_WORKER`, so a marked fork's suite reflects delta health,
   not environmental refusals (CHR-044).
3. **Funnel writes leave the coord tree honest.** `dispatch_import` /
   `dispatch_conclude_phase` refresh the checked-out coord tree + index to the
   advanced ref (auto-sync, precondition: belt-verified clean pre-import), so
   `git status` reads forward-diff and the reverse-diff footgun is gone.

Closure intent: a worker in a marked fork, and the orchestrator after a funnel
write, both see a suite/tree that reflects real delta health with **no
environmental red and no reverse-diff** — no `git restore` ritual, no recalled
recovery idiom.

## Non-Goals

- Refusal legibility / plan-time selector lint (move C) — that is **SL-224**.
- The `dispatch next` state machine, no-shell-git-in-funnel prohibition, and the
  memory-blind benchmark — RFC-016 Cluster 2 / move A (this slice makes #3's
  memories *retirable*; formal retirement + benchmark is the later slice).
- Reworking the worker-mode guard's semantics — the guard is correct; we stop
  *conflating* its refusal with delta damage.
- ISS-219 (295k-char transcript in the refusal) and the architecture-layering
  ratchet-red handoff (IMP-293) — adjacent false-red ergonomics, not in this cut.

## Affected surface (coarse — `/design` refines)

- `src/mcp_server/worker_commit.rs` — gate binary resolution (#1).
- `src/mcp_server/dispatch.rs`, `src/worktree/import.rs`, `src/worktree/mod.rs`
  — funnel ref advance + coord-tree sync (#3).
- `src/worktree/marker.rs`, `src/commands/guard.rs` — worker-mode guard / marker.
- `tests/e2e_*.rs` authored-write goldens (`e2e_worker_guard.rs`,
  `e2e_dispatch_sync.rs`, `e2e_doctor_golden.rs`, and the ~30 marker-poisoned
  suites) — marker-aware skip (#2).

## Risks / Assumptions / Open questions

- **A:** the server knows its own binary path (or can build/locate the coord
  binary) for #1 — confirm the resolution seam in `/design`.
- **A:** the coord tree is guaranteed belt-clean pre-import, so auto-sync is a
  safe fast-forward of the checkout — verify the precondition.
- **OQ:** #2 — skip the goldens under marker, or route worker_commit's gate to
  *exclude* authored-write goldens inside a marked fork? (Two places to cut.)
- **OQ:** #3 auto-sync default-on for all funnel writes, or an opt-in flag?
  (case-note recommendation: auto-refresh, safe under the clean precondition.)
- **Risk:** #2 marker-skips must not mask a *real* authored-write regression on
  the main arm — the skip must be strictly marker-gated.

## Verification / closure intent

VT per fix: a gate test showing green on a conformance-rule-changing fork under
the in-process binary; a marked-fork e2e run that skips the authored-write
goldens and goes green on a clean delta; a post-import `git status` golden
reading forward-diff. Closes when all three land green and the checkout-import
recovery memories are marked retirable.

## Follow-Ups

- Move-E / OQ-6: mark the checkout-import recovery memories retirable once #3
  lands (formal retirement rides the Cluster 2 benchmark slice).
- Adjacent, not in scope: ISS-219 (refusal transcript cap), IMP-293
  (ratchet-red handoff signal), IDE-028 (phase-status push).
