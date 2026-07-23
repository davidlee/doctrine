# Notes SL-225: Funnel false-red elimination

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## 2026-07-24 — DEC-003 design inquisition

RV-291 tried DEC-003's zero-engine reversal and sustained three findings:

- F-1 blocker: `DOCTRINE_BIN` is resolved when the MCP server loads, but the
  coord build it must name is normally created later by `dispatch setup`; the
  design needs an explicit restart/rebind lifecycle or a distinct stable
  project-local indirection before `/plan`.
- F-2 major: `design.md` still asserts the rejected `current_exe()` publication
  at lines 152-153.
- F-3 major: VT-1 proves recipe argv precedence with a stub, not elimination of
  ISS-218 through the actual `worker_commit` gate.

All findings are terminal with `design-wrong`; no follow-up or tolerated drift
was sanctioned. RV-291, this note, and the RFC-011 CLI-friction case-note form a
dedicated authored review commit. `just gate` passed after the review (existing
doctor warnings only).

## 2026-07-24 — DEC-003 second-pass inquisition

RV-292 re-tried the `d8cf94d6` self-locating remediation and found DEC-003 still
unfit for `/plan`:

- RV-291 F-1 remains standing. Every linked worktree shares the primary
  repository's `--git-common-dir`; its parent is the primary tree, not the
  coordination worktree. Rung 3 therefore selects the primary/edge binary or
  refuses. Source already warns that primary-worktree discovery via
  `git worktree list` is correct “unlike parent(--git-common-dir)”
  (`src/git.rs`).
- RV-291 F-2 is resolved: `current_exe()` survives only as a rejected
  alternative, and the engine publishes no environment.
- RV-291 F-3 is resolved: VT-1/1b now name a rule-divergent real
  `worker_commit` gate proof plus the missing-build negative; VT-1c is only the
  precedence unit.
- New F-2 major: `just check` validates before its build leg, so executable
  existence does not prove freshness for a current-phase rule change.
- New F-3 major: dispatch setup does not establish the claimed coord build, and
  live governance still mandates the rejected frozen `DOCTRINE_BIN` rule despite
  DEC-003 saying the softened note already lives there.
- New F-4 minor: the generic-host path assumes undeclared Git 2.31+
  `--path-format=absolute` support and exits before fallback on older Git.

All four RV-292 findings are terminal `design-wrong` (F-1 blocker, F-2/F-3
major, F-4 minor). The reusable Git invariant is recorded as
`mem.fact.git.common-dir-not-coord-root`, related to the earlier
`mem.fact.dispatch.coord-root-not-git-common-dir`. `just gate` passed after the
review (existing doctor warnings only). RV-292, the memory, this note, and the
RFC-011 case note landed together in `379d64c7`.
