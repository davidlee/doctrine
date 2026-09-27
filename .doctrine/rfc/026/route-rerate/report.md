# RFC-026 E14 — The route taxonomy re-rated, unsteered and in the design facet

Added **2026-09-27**. Run under `preregistration.md`, committed (`e02839ffd`;
its B/D model amended before any run) before any rater ran. Apparatus
in this directory: both prompts, four classifications verbatim, rater B's
excluded first run, the analysis script `rates.py`, and its outputs
`transfer-AB.txt` / `design-CD.txt`. Produced for `SL-270` step 1.

## Answer

- **P11: split `owner-fix`.** On the design-review population, where routing
  applies, the two raters agree on duplicate-vs-stale for 17 of 19 jointly
  `owner-fix` findings (89.5%), and 9 of those 19 (47%) are jointly `stale`.
  Both exceed the pre-registered thresholds (≥80%; ≥5 and ≥⅓).
- **`E13`'s steering was material, narrowly.** Unsteered, `owner-fix` falls to a
  mean 33.0% of the 47 audit/code-review findings (A 40.4%, B 25.5%), against
  `E13`'s 44.7%. Threshold ≤34.7%. The margin is under one finding per rater.
- **The taxonomy discriminates in both populations**, under both raters.
- **The weakest boundary is not `owner-fix`. It is `probe` / `control`.** In the
  design population it is 26 of the 70 disagreements (37%). This was not
  pre-registered and is reported as exploratory.

## Populations and raters

| population | findings | raters | route agreement | Cohen's κ |
|---|---|---|---|---|
| T — `E13`'s 24 reconciliation + code-review ledgers | 47 | A `pi-research`, B `gpt-6-luna` | 31 / 47 (66.0%) | 0.56 |
| D — `E11`'s 11 + `E12`'s 3 design ledgers | 155 | C `pi-research`, D `gpt-6-luna` | 85 / 155 (54.8%) | 0.44 |

Rater pairs span model families. κ 0.44–0.56 is moderate agreement: the route
set is usable by an agent but not settled by the definitions alone.

## Route counts

| route | A (T) | B (T) | C (D) | D (D) |
|---|---|---|---|---|
| `review` | 5 | 5 | 6 | 18 |
| `demonstrate` | 3 | 4 | 31 | 29 |
| `probe` | 7 | 7 | 48 | 17 |
| `control` | 10 | 17 | 31 | 66 |
| `owner-fix` | 19 | 12 | 33 | 24 |
| `none` | 3 | 2 | 6 | 1 |
| modal share | 40.4% | 36.2% | 31.0% | 42.6% |

**R2, discrimination:** every rater uses all five routes and has a modal share
≤50%. Both populations **pass**. R2 is read per population, so `E13`'s
reconciliation-only "fail" (52%) is neither confirmed nor refuted here.

## R4 — the P11 decision (population D)

| | count |
|---|---|
| jointly `owner-fix` | 19 |
| second label agreed | 17 (89.5%) |
| jointly `stale` | 9 (47.4%) |
| jointly `duplicate` | 8 |
| split on the label | 2 (`RV-307` F-19, `RV-325` F-9) |

→ **split**. `owner-fix` is *not* overloaded in design review (15–21% of
findings, against 25–40% in T). The case for splitting is that it holds two
classes of near-equal size that the raters tell apart reliably. Deleting a
duplicate and editing a stale record are different acts. In T the same rule
reads 72.7% agreement and would be indeterminate; T is not decisive (`QUE-224`).

## R5 — `none`

C marked 6, D marked 1, and they share no finding. No recurrent shape in D, so
no candidate route is recorded. In T, both raters mark `RV-372` F-13
(*can the handed-back worktree be verified where it lands?*), as `E13`'s rater A
did.

## Exploratory — where the raters split

| pair (C, D) | count |
|---|---|
| `probe` / `control` | 26 |
| `owner-fix` / `control` | 7 |
| `demonstrate` / `control` | 6 |
| `owner-fix` / `review` | 5 |
| `probe` / `demonstrate` | 5 |
| others | 21 |

C leans `probe` (48) and D leans `control` (66). The findings in that cell are
typically hostile input against an integrity check: *the adversary defeats the
check* is both "does the mechanism withstand the adversary?" and "would the
check notice failure?". `reviewing.md`'s tie-break puts `control` before
`probe`, and the rater prompt did not carry it, so part of this split is the
tie-break's absence rather than the definitions. Whether the tie-break alone
closes it is untested.

## Deviations, disclosed

1. **Rater B was re-run (R6).** Its first run read the blindness rule *"if a
   finding carries a route … ignore it"* as *skip the finding* and dropped 8 of
   47. The rule's wording was ambiguous, which is the analyst's fault. Re-run
   once from the same prompt, as R6 requires; the second run covered all 47. The
   first run is kept as `classification-rater-b.run1-excluded.md`. Raters A, C
   and D read the rule as intended.
2. **Rater D omitted the severity column on 9 `RV-377` rows.** The route shifted
   one column left. `rates.py` detects the shift and reads the route from it.
   No content is inferred.
3. **Rater C adds two placeholder rows** for `RV-347` and `RV-359`, which have
   no severe findings. They are excluded by the `F-N` filter.
4. **Thresholds were close:** R3's margin is 1.7 points; R4's second-label
   agreement rests on 19 rows.

## What this does not claim

It does not validate `P10`, which only a use trial can. It does not revise any
count in `E11`–`E13`. It supersedes `E13`'s *reading*, that `owner-fix` fails
to discriminate outside design review, and not its data. With κ in the 0.4s, a
single agent's route is a noisy label; a trial that counts routes should expect
that noise.
