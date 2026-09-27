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

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-27 · design locked · b196d9183

### Produced
- research + pre-registered re-rate — RFC-026 E14 (`.doctrine/rfc/026/route-rerate/`); QUE-224, QUE-225 answers revised (commits 4217bb9a4..57e9f3b9b)
- design locked — DEC-326, DEC-327, DEC-330, DEC-331, DEC-333, DEC-334 (supersedes DEC-276); RV-400 design review, 3 raiser rounds (commits 3d012dd81..b196d9183)
- minted: REV-065 — ADR-007 D-C5 / SPEC-032 route set + lock exception, apply at reconcile; CHR-082 — runs trial two after SL-270 closes
- scope change during design: facet refusal and conclude-writes-counters dropped (DEC-331, DEC-327); route split added (DEC-330)
- no code yet; gate not applicable

### Learned
- DEC-326 — design-lock checks on a review ride observe_pass / PassFacts beside undisposed_blockers; conclude stays unblocked (ADR-007 D-C8)
- RFC-026 E14 — E13's rater prompt steered owner-fix; probe/control is the weakest route boundary

### Open
- RV-400 F-1 — owner follow-up at audit: is leaving open majors, superseded passes and post-lock dispositions outside the lock check the long-term behaviour, or backlog?
- RV-400 F-5, F-7 — control-routed; verify against their criteria (F-7: VT-2 in the lock-check phase; F-5: CHR-082 window-open step)
- REV-065 — apply at reconciliation
- mem_01a0d91d7a0d75f1853c505d08237451 — teaches --route owner-fix; update in the route-set phase
- 3 untracked friction observations under `.doctrine/observations/records/` — owner to sweep
