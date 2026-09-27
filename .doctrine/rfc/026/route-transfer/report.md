# RFC-026 E13 — Does the route taxonomy transfer beyond design review?

Added **2026-09-27**. Produced at `CHR-077`'s trial conclusion to answer `QUE-224`
(*should the route axis extend beyond design-review ledgers?*), which `SL-260` left
to the trial rather than answering by assumption. Apparatus preserved in this
directory: `rater-prompt.txt` (the instrument both raters received),
`classification-rater-a.md` and `classification-rater-b.md` (verbatim).

**Answer: the taxonomy is not extended — not for want of evidence, but because the
evidence says the gap is in one route's definition rather than in the axis's
scope.** Extension would carry a term that no longer discriminates.

## Method, fixed before the run

`QUE-224` could not be answered by the trial window itself, and waiting would not
have answered it. Nothing instructs an audit or code-review pass to route — the
axis is scoped to design-review ledgers in `install/review-ledger.md` — so the
population that would produce observational evidence never arises. That circularity
is why this was run as a **classification** study rather than a use study: the
question is whether the five routes can *name* audit and code-review findings, which
is answerable over ledgers that already exist.

- **Population, by rule:** the 12 most recent ledgers by id, descending, in each of
  the reconciliation and code-review facets (24 ledgers), *every* blocker and major
  finding on them. No sampling, no filtering on outcome.
- **47 severe findings** — 25 reconciliation, 22 code-review (3 and 5 blockers
  respectively, the rest major). Every ledger was read through
  `doctrine review show RV-NNN --json`.
- **Two independent raters**, same instrument, neither shown the other's labels.
  Each assigned one of the five routes, or the literal `none` — meaning the finding
  is severe but no route question is the question it asks. `none` was made available
  deliberately, because a forced fit would have destroyed the very signal the test
  exists to find.

## Result

| facet | severe | owner-fix | control | demonstrate | review | probe | none |
|---|---|---|---|---|---|---|---|
| reconciliation | 25 | **13** / **13** | 5 / 1 | 1 / 3 | 4 / 4 | 1 / 0 | 1 / **4** |
| code-review | 22 | **8** / **8** | 6 / 7 | 3 / 4 | 1 / 0 | 4 / 2 | 0 / 1 |
| **total** | **47** | **21 / 21 (44.7%)** | 11 / 8 | 4 / 7 | 5 / 4 | 5 / 2 | 1 / **5 (10.6%)** |

Rater A / rater B. Both put `owner-fix` at **21 of 47 (44.7%)** independently, and
at **13 of 25 (52%)** on reconciliation independently. Every one of the five routes
fired at least once in both readings.

**The agreement is the finding.** `owner-fix` is the modal route on both facets
under both raters, and on reconciliation it takes just over half the population.
That is not a taxonomy that fails to fit — it is one where a single term does most
of the work, which is a discrimination problem rather than a coverage problem.

**Both raters independently diagnose the same cause.** `owner-fix` is defined as
*"do two accounts of one fact disagree?"*, built for duplicate sources of truth. On
these facets it absorbs the far larger class of *a design, decision, or governance
record says X and the shipped code does Y* — stale-account drift, not duplication.
Rater A: under a narrow reading, duplicates only, about a dozen rows become `none`
and "the taxonomy's fit drops sharply"; the reported 44.7% "is produced by reading
code-vs-design/governance mismatch as two accounts of one fact". Rater B: "that
breadth is the main transfer risk, and it is structural, not incidental" — a reader
counting `owner-fix` cannot tell a duplicate from drift, and 44.7% of the population
disappears into one bucket.

**The `none` class is real; its threshold is not agreed.** The raters differ on how
often it applies (1 vs 5) but name the same shape: durability and crash
recoverability (`RV-372` F-14, `RV-342` F-4), harness hermeticity against shared
state, verification venue — whether a delivered artefact is checkable where it lands
(`RV-372` F-13), ambient-environment independence of a gate (`RV-387` F-4), and
record reachability (`RV-366` F-6). None of these is a design commitment, a disputed
connection, an adversary, a check, or a duplicate. Per `DEC-276` this disagreement is
reported rather than reconciled into a point estimate.

## Reading rule, and its disclosure

The discrimination reading applied was fixed before the run: *≥3 of the 5 routes
firing with the modal route at or under 50% counts as discriminating.* Code-review
passes it (modal 36%, 5 routes). Reconciliation narrowly fails (modal 52%). The
modal-route half of the rule is the half doing the work, and it is the half `E12`
would predict to be unmeasurable if the population were too small; 25 findings is
small.

**Disclosed rather than buried:** the population rule was fixed before the raters
ran, but this reading rule was fixed only in the analyst's reasoning and was not
published before the run. It is recorded here so it can be judged on its merits
rather than trusted, which is the discipline `DEC-276` exists to enforce and this
test partly failed to follow.

## Residual

`--route` is not gated by facet — a code-review or reconciliation finding can carry
one today — while `install/review-ledger.md` scopes the axis to design-review
ledgers. The mechanism therefore permits what the governance forbids, and the
forbidding is unenforced. Neither direction is currently asserted by code. This is
unrelated to the transfer result and is stated because the test brought it to light.

## What this does not claim

It is not a defect rate and not a verdict on `P10`. It is 24 ledgers and 47 findings;
rater agreement is reported raw and no reliability statistic is computed. Both
raters worked from finding text and disposition, not from the convention's own
document, so their reading of each route's intent is inferred (rater B marks this
explicitly). Whether one facet's distribution would hold on a different 12 ledgers is
untested, and the two facets sampled differ in size by an order of magnitude in the
wider corpus (`E1`: 183 reconciliation ledgers against 19 code-review).
