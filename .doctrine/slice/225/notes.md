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

## 2026-07-24 — Post-implementation audit (RV-294)

Both phases implemented and verified; ledger `done`, **no blockers**. Landed:
`ad1fff949` + `575757955` (PHASE-01, fix #1 + ii), `613af9c51` (PHASE-02, fix #2),
`837801a5` (lifecycle → audit).

**Evidence (all green).** `doctrine check gate` → exit 0 (fresh binary via
`build`→`validate`, full `test-all`); `e2e_worker_gate_skip` 7/7 (VT-1/1b/1c/1d);
`worker_marker_at` unit (VT-2); worker_commit suites green unchanged (VT-1e /
behaviour-preservation). Implementation matches design verbatim (validate body,
belt reorder, the one engine env line, governance.md reframe, helper + 27 guarded
goldens with `e2e_worker_guard` correctly unguarded). The IMP-306 handover-SKILL
foreign redness the phase-01 sheet flagged as blocking is **resolved** (IMP-306
closed) — gate is clean.

**Five findings, all verified terminal (2 minor, 3 nit; none blocking):**
- **F-1 (minor).** PHASE-01 drift: EX-5/design name CLAUDE.md as a second edit
  target, but the DOCTRINE_BIN rule is single-source in `.doctrine/governance.md`
  (zero hits in CLAUDE.md) and reaches the agent surface via the boot inline —
  editing CLAUDE.md directly would violate STD-001. EX-5 intent met. → reconcile
  design prose.
- **F-2 (minor).** PHASE-02 drift: §"Which goldens"/§"Code impact" name three
  illustrative goldens that are not the empirical target set (worker_guard exercises
  the guard → stays unguarded; dispatch_sync + doctor_golden don't false-red). Plan's
  "enumerated at implementation" is authoritative; delivered set correct. → reconcile
  design prose **+ `slice selector rm`** the three undelivered selectors.
- **F-3 (nit, tolerated).** Boundary registry polluted — PHASE-01 solo auto-binding
  `[886153a5f..30f2b82a8]` swallowed 8 interleaved IMP-306/chore commits (55 undeclared).
  Verified manually via per-commit `--against`. Runtime/disposable; non-contiguous
  two-commit landing admits no clean re-record. **Already tracked by IMP-175 (identical
  mechanism) + IMP-292 (audit-time signal degradation)** — fresh repro, no new mint.
- **F-4 (nit, aligned).** `e2e_check_regression` guard is in-charter (a false-red born
  of PHASE-01's own validate-skip fingerprint shift; marker-gated, no coverage lost).
- **F-5 (nit, tolerated).** ~95 explicit early-returns kept over a macro — greppability
  (VA-1's set-membership audit) + no hidden control-flow > cosmetic LOC.

Reconciliation brief written to `review-294.md` (§Per-slice: design prose; §Selector
registry: `slice selector rm` the 3; no REV — governance touch was authored in-phase
and is correct). Handed to `/reconcile`.
