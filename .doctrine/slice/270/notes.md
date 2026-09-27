# Notes SL-270: Routing trial two

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-01 VA-1 sweep (2026-09-27)

Re-runnable:

    rg -l --hidden "owner-fix|OwnerFix" -g '!target' -g '!.git' .
    rg -l --hidden -i "five routes|closed five|exactly five|5-set|five-route" -g '!target' -g '!.git' .

Every hit is excluded, for one of these reasons:

- **Historical evidence**: closed-slice docs (SL-260, SL-261, SL-268, older),
  review ledgers, knowledge records, RFC-026 (E11-E14) and RFC-032 material,
  REV-064, and observation records. They record what was true when written.
- **Via REV-065, at reconcile**: ADR-007, SPEC-032, and REV-065's own before
  column.
- **This slice's own authored docs**: slice-270.md, design.md, plan.*, notes.md.
- **Legacy or refusal tests**: `vocab.rs` (a test that `owner-fix` is refused;
  a doc comment naming the split), `tests/e2e_review_golden.rs` (VT-7's
  legacy-read ledger). The route-teaching memory names `owner-fix` only as
  retired.
- **Not about routes**: the "5-set" hits in `vocab.rs` are the finding-status
  and disposition vocabularies, which really are five.

## PHASE-02 notes (2026-09-27)

- **EX-4 had no fallout.** No existing fixture locks over a disposed severe
  finding with no route. `defect_warning_printed_gate_unchanged`, predicted to
  break, exercises disposition admission, not the lock, and passes unchanged.
- **Remedy text departs from design sec-3.** `review amend` requires
  `--response`, so the refusal reads `review amend <RV> --finding F-n --route
  <route> --response … --note …`. The design's version omits `--response`.
- **Remedy placement.** `design_run` has no per-cause remedy (`Contract::remedy`
  is per condition, rendered from the rule), so the per-status repair is in
  `Cause::SevereFindingsUnrouted`'s display text.
- **Control for RV-400 F-7, observed.** With the "already an undisposed
  blocker" filter removed, VT-2 fails and lists F-9 in both lists. Restored.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-27 · closed · 7e1844cd4

### Produced
- research + pre-registered re-rate — RFC-026 E14 (`.doctrine/rfc/026/route-rerate/`); QUE-224, QUE-225 answers revised (commits 4217bb9a4..57e9f3b9b)
- design locked — DEC-326, DEC-327, DEC-330, DEC-331, DEC-333, DEC-334 (supersedes DEC-276); RV-400 design review, 3 raiser rounds (commits 3d012dd81..b196d9183)
- minted: REV-065 — ADR-007 D-C5 / SPEC-032 route set + lock exception, apply at reconcile; CHR-082 — runs trial two after SL-270 closes
- scope change during design: facet refusal and conclude-writes-counters dropped (DEC-331, DEC-327); route split added (DEC-330)
- plan — two phases (commits 532797c62..2d27b075a)
- PHASE-01 six-route set — f27cad64b, 078fc994e; VT-1/6/7 pass, VA-1 recorded above
- PHASE-02 lock route check — b8d10e714, d6d85fe1d; VT-2..5 pass; gate green
- reconcile — design sec-3/sec-4 edited; REV-065 done (ADR-007, SPEC-032) — 7e1844cd4
- close — PATH doctrine 0.46.1 serves six routes (CLI + MCP); `doctrine install` run; memory harvest: none new (route memory already current), consciously rejected
- audit — RV-401 (F-1..F-5 terminal; brief: design sec-3, sec-4, REV-065); RV-400 F-7 verified; minted IMP-498 (after CHR-082)

### Learned
- DEC-326 — design-lock checks on a review ride observe_pass / PassFacts beside undisposed_blockers; conclude stays unblocked (ADR-007 D-C8)
- RFC-026 E14 — E13's rater prompt steered owner-fix; probe/control is the weakest route boundary
- `review amend` requires `--response`; a remedy naming amend must too (PHASE-02 notes)
- plan.toml: phase keys stranded under `[requirements]` pass `doctrine validate` (friction observation 01a0e1b0)

### Open
- RV-400 F-5 — control-routed; hosted by CHR-082's window-open step (stays answered)
- IMP-498 — RV-400 F-1 / RV-401 F-3 coverage gap; decide after CHR-082
- tell SL-271's owner: RV-399 severe findings need routes before any re-lock
- RV-400 carries no `route` on its own severe findings; re-locking SL-270's design would now refuse. Not needed
- 4 untracked friction observations under `.doctrine/observations/records/` — owner to sweep
