# Implementation Plan SL-199: Confined subagent orchestrator drive-loop (Mode B)

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Five phases move dispatch orchestration off the main thread onto a fully-confined
`dispatch-orchestrator` subagent that drives the funnel via doctrine MCP tools. The
work is authored against the **post-SL-198 world**: SL-199 `needs` SL-198 (the
`worker_commit` keystone, the per-worktree dispatch record, the coord enumerate
seam, the agent-def tool-surface lint, the import-a-commit switch), which is `ready`
but **not yet executed** — verified absent from the execution base 2026-07-04. That
is the slice-level entrance precondition on every phase; execution is gated on
SL-198 landing.

The design was locked and then externally reviewed (codex GPT-5.5, 2026-07-04). The
review moved the centre of gravity of the plan: the confined arm needs **more
server-side transactionality** than a set of thin MCP wrappers. Three verified code
facts drove the phase shape — `run_record_boundary` is non-committing and
prepare-review tree-reads *committed* history (F1); `classify_import` hard-refuses
undeclared scope, so undeclared cannot be "advisory" (F2); and `set_phase_status`
suppresses solo-binding in coord context, so the real crash mode is a *missing*
boundary, not a clobbered one (F3). The plan is built so the risky transactional
core is isolated, built, and tested **before** any tool surface rides it.

## Sequencing & Rationale

A hard serial spine 01 → 02 → 03 → 05, with 04 hanging off 03:

- **PHASE-01 (create-fork confined discriminator) is first and independent.** It is
  the §A linchpin the feasibility probe forced: a confined subagent's Bash cwd resets
  every tool call, so positional-cwd arming produces Passthrough (no branch, no jail
  record) and SL-198's `worker_commit` can't resolve the worker. The fix is pure and
  self-contained — an additive Fork trigger in `classify_create` plus a one-shot
  arm the hook consumes — and it depends on nothing else in the slice. It gates the
  end-to-end (PHASE-05) but not the tool-surface phases, so it can land early against
  the current tree while SL-198 is still in flight.

- **PHASE-02 (commit-on-behalf + coord-by-slice resolver) is the transactional
  trust anchor, isolated deliberately.** The external review showed the load-bearing
  risk is not the tool signatures but the *server-side commit the confined
  orchestrator cannot perform itself*. Building the commit-on-behalf primitive and
  the coord resolver as one phase — with the commit-provenance contract (R4/F6)
  defined and asserted here — means the tool surface in PHASE-03 is thin wrappers
  over a primitive already proven, rather than three tools each re-litigating how the
  server commits on the orchestrator's behalf. This mirrors SL-198's own "trust
  anchor first" sequencing (its record+resolver preceded `worker_commit`).

- **PHASE-03 (the three funnel tools) rides PHASE-02.** `dispatch_import`,
  `dispatch_conclude_phase`, and `dispatch_reap` each resolve the coord tree
  server-side and ride an existing `run_*` belt — one seam, two doors. Two review
  corrections live here and are the phase's sharpest exit criteria: undeclared scope
  is a **hard pre-commit refusal** (nothing lands, report-and-halt — not an advisory
  the orchestrator blesses, which it could not, having no `git reset` inside the
  jail); and the phase-conclusion is **atomic** (flip + boundary + one commit), so a
  fault never leaves a phase `completed` in committed history without its boundary.
  The atomic `conclude_phase` is a partial reversal of the design's earlier
  "all-discrete tools" position — earned by the review, because two independent
  commits can split across a crash.

- **PHASE-04 (agent-def + lint allowlist row) hangs off PHASE-03** and SL-198's lint
  mechanism. It is deliberately small: the def is authored and the lint gains one
  data row (the orchestrator allowlist), referencing the same named tool-name
  constants PHASE-03 defines (STD-001, no second literal). It can land any time after
  PHASE-03 and SL-198's lint, in parallel with nothing depending on it but PHASE-05.

- **PHASE-05 (drive-loop + docs + end-to-end) is last** because it composes
  everything: the orchestrator's operating guidance (the cadence, the one-shot arm,
  the report-and-halt boundary), the shipped `dispatch-mechanics.md` Mode B section,
  and the live human-witnessed proof that a confined orchestrator drives a real phase
  fork→land→conclude→reap with its raw `.git` writes walled. The VH here is the
  slice's closure evidence — the empirical mirror of the §6 feasibility probe, now
  showing the §A discriminator produces the branch+record positional arming could not.

## Notes

- **The confined arm's undeclared response is report-and-halt, by construction.**
  SL-198's `worker_commit` lets an undeclared src path *soft-commit* to the worker
  branch (its EX-6), but `classify_import` hard-refuses undeclared at the import
  boundary. The *main-thread* orchestrator resolves that by amending selectors and
  committing; the *confined* orchestrator cannot commit from inside the jail, so its
  only response is to surface the refusal to the main thread. This is a clean, intended
  boundary — not a gap — captured in PHASE-03 EX-4.

- **OS1 resolved at plan** (design R1 closed): `coordinate()` always checks the coord
  worktree out on `dispatch/<NNN>` (`worktree add -b …` / `add <dir> <branch>`), never
  `--detach` (coordinate.rs:212-227) — the `coord_in_dispatch` guard is sound.

- **Open questions carried into phase-plan** (resolve just before the owning phase):
  commit provenance — the exact author/message contract (PHASE-02 EX-3); start-
  `in_progress` flip — does the funnel need a separate tool or does the flip ride
  `conclude` only (PHASE-03)?; lint ceiling-vs-floor and iterate-all (PHASE-04 EX-2).

- **Behaviour-preservation gate** applies throughout: every funnel tool composes the
  existing `run_*` seams rather than forking them; the main-thread funnel and the
  subprocess arm stay green unchanged (PHASE-03 EX-5, PHASE-05 EX-4).

- **Test-file paths in `plan.toml` name post-SL-198 / new modules** (e.g.
  `src/mcp_server/dispatch.rs`) that do not exist on the current base; they are the
  intended homes, checked at the dispatch handover after implementation, not at plan
  time.
