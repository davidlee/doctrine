# IMP-518: Capsule contracts: unnamed orchestrator, early planner sheet, commit hygiene

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

Cross-repo friction survey (2026-10-17) of the observation corpora in
`~/dev/satan` (31 records, SL-016/017/021) and `~/flakes` (41 records,
SL-006..011). Most doctrine-side friction sits in the `/capsule-driver` path.
Three prose-only contract fixes cover roughly two thirds of it.

## 1. Driver spawns the orchestrator unnamed

`.agents/skills/capsule-driver/SKILL.md` loop step 1 does not forbid passing
`name` to the `capsule-orchestrator` spawn. A named spawn becomes an
in-process teammate; its children's `SubagentHandback` then fails ("the agent
that spawned you is no longer running") and the report surfaces at the driver
seat, or is lost. Downward `SendMessage` to a named orchestrator also never
arrived (satan SL-016).

- Evidence: ~12 records, every capsule spawn in both repos. Root cause
  confirmed by a flakes probe (2026-10-16): unnamed parent receives the child
  hand-back; named parent gets "ended without delivering a report".
- `install/agents/claude/capsule-orchestrator.md:46` already forbids `name` on
  the orchestrator's own children; the driver needs the same rule (and the
  reason) for the orchestrator.

## 2. Planner writes its sheet early

`install/agents/claude/capsule-phase-planner.md` has `maxTurns: 40` and no
budget guidance. Planners read until the cap and write nothing, or write the
sheet and never hand back.

- Evidence: 7 records (satan SL-016 P04/P12, SL-021 P03; flakes SL-006 P03,
  SL-007 P03, SL-008 P07). One burned ~180k tokens for no output. Every retry
  told to bound reading and write early finished in ~20 calls.
- Fix: write a skeleton sheet early (by ~call 15), then refine; reserve the
  last turns for the hand-back.

## 3. Commit hygiene in dirty shared trees

`capsule-orchestrator.md` says "Commit the phase" with no hygiene rules.
Capsule runs happen in trees with the user's uncommitted work (`$HOME`
worktree, satan-corpus).

- Evidence (flakes SL-010, SL-011, SL-008; satan SL-021):
  `git commit … -- .` swept the user's unstaged justfile; a plan prescribed
  `just commit` in satan-corpus (`git reset` + `git add .`), which would have
  swept unrelated state; tree-wide `nix fmt` would reformat the user's dirty
  files; staging a gitignored `.doctrine/state` sheet aborted the whole
  `git add`.
- Fix: commit with an explicit pathspec of the files the phase changed; never
  `-- .`, a project commit recipe, or a tree-wide formatter; never stage
  runtime state; `git status --porcelain` first and leave foreign changes
  alone.

## Out of scope (neighbours)

- ISS-519 — planner vs `/phase-plan` on who flips phase status. Same file
  cluster; also the sheet-asks-worker-to-write-`notes.md` conflict (flakes
  SL-011). Natural follow-on, kept separate.
- Capsule phase-delta binding (`verify-vt` UNATTRIBUTABLE after amend or
  without `record-delta`; IMP-192 neighbour) — needs investigation; slice it.

## Acceptance

- Driver skill step 1 forbids `name` on the orchestrator spawn, with reason.
- Planner contract carries the write-early / hand-back-before-cap rule.
- Orchestrator contract carries the commit-hygiene rules.
- Published copies (`plugins/`, installed agent defs) regenerated; `doctrine
  check gate` green.
