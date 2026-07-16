# SL-220 Phase 0 baseline evidence (PHASE-01 EX-4)

Pre-flip ranking baseline per design §5 — the R2 accepted-evidence base every
later ranking delta (PHASE-07 post-flip diff, /audit) is judged against.

## Provenance

- Captured: 2026-07-17, primary tree on `edge` at `41a88a1c` (corpus content
  identical to dispatch base `2efcb52e` — the intervening commits touch
  `.doctrine/rfc/011/` and `revision/024/` metadata only, neither scored
  differently).
- Instrument: `scripts/value_baseline.py` at coord commit `0e043305`
  (worker-authored `c08073a3` + operator fixup excluding runtime
  `.doctrine/state/` from the neutral copy).
- Binary: `doctrine 0.21.0` (`doctrine survey --json --limit 0`).
- Clean-tree precondition held for every invocation; `--neutral` operated on a
  temp copy — primary tree verified untouched (`git status --porcelain` empty
  throughout).

## Artifacts

- `phase0-live.json` — live ranking, 244 rows, `[rank, id, score]`.
- `phase0-neutral.json` — ranking with `[priority.coefficients] value = 0`,
  244 rows.
- `phase0-diff-top20.txt` — rank-move report, top-20 window.
- `phase0-diff-full.txt` — rank-move report, uncapped (`--top 0`).

## Finding: the neutral snapshot is degenerate — deliberately kept

Every neutral score is 0.0000 (ranking collapses to canonical-id order). This
is arithmetic, not a capture fault: `score = risk_dim + leverage +
optionality + burndown_term`, and every term except `risk_dim` propagates
`value_dim` (which carries `coefficients.value` as a factor); no entity in the
eligible set carries a `[risk]` facet, so zeroing the value coefficient zeroes
the entire surface. The degeneracy is itself baseline evidence: **the whole
live ranking today flows through the value dimension** — largely
`DEFAULT_VALUE = 1.0` plus the unattributed authored `[value]` facets whose
constitutional authority IMP-290 / RFC-020 exists to dissolve. The operative
pre-flip comparator for PHASE-07 is therefore `phase0-live.json` (live vs
post-flip live); the neutral pair quantifies total value influence, not a
value-free ranking.
