## Findings

| ledger | F-N | severity | route-or-none | second-label | reason |
|---|---|---|---|---|---|
| RV-366 | F-2 | major | owner-fix | stale | The design’s no-witness claim lags a reproduced implementation case. |
| RV-366 | F-3 | major | owner-fix | stale | The code-impact table omits changes that landed. |
| RV-366 | F-4 | major | owner-fix | stale | The selector registry omits deliberate implementation paths. |
| RV-366 | F-6 | major | none | neither | Is this decision record properly completed and linked? |
| RV-369 | F-1 | blocker | demonstrate |  | A real terminal run shows the proposed endpoint connection fails. |
| RV-369 | F-2 | major | control |  | The planned tests must reject the shipped endpoint defect. |
| RV-369 | F-6 | blocker | review |  | Should the design accept the cost and limits of large renders? |
| RV-369 | F-7 | major | probe |  | A terminal rejection leaves the mechanism unable to detect failure. |
| RV-372 | F-3 | major | control |  | The planned checks omit a required facet-only read path. |
| RV-372 | F-4 | major | review |  | Should the design accept the measured cost of its facets feature? |
| RV-372 | F-5 | major | review |  | Should the commitment stand given the human acceptance result? |
| RV-372 | F-6 | major | none | neither | How should deliberately withheld fields be disclosed? |
| RV-372 | F-13 | blocker | demonstrate |  | The handed-back worktree cannot connect to the gate in its environment. |
| RV-372 | F-14 | blocker | probe |  | The workaround exposes shared repository configuration to corruption. |
| RV-372 | F-15 | major | owner-fix | stale | Other notes still describe the superseded command behavior. |
| RV-381 | F-5 | major | owner-fix | stale | Governance documents lag the shipped capability. |
| RV-381 | F-6 | major | review |  | Should pointer rendering meet the stated presentation commitment? |
| RV-387 | F-4 | major | none | neither | Why does this ambient environment make the gate fail? |
| RV-390 | F-7 | major | control |  | The check must detect substitution of a synthetic run for real state. |
| RV-395 | F-1 | major | control |  | The acceptance check must reject citations to undelivered keys. |
| RV-395 | F-2 | major | owner-fix | stale | The design’s boundary statement conflicts with changed source paths. |
| RV-395 | F-13 | major | owner-fix | stale | The delivered corpus lags the edited shipped masters. |
| RV-317 | F-2 | major | probe |  | A malformed UID triggers the reported panic. |
| RV-321 | F-1 | blocker | probe |  | Persisted strings can exceed the mechanism’s claimed bounds. |
| RV-321 | F-3 | major | control |  | The test must reject alternate constructor paths. |
| RV-324 | F-3 | major | probe |  | A noncanonical prefix can trigger the reported merge. |
| RV-324 | F-4 | major | none | neither | How should imported prose retain its source fingerprint? |
| RV-342 | F-1 | major | control |  | The check must reject mismatches between declarations and construction sites. |
| RV-342 | F-4 | major | control |  | The fault check cannot reach the unrecoverable interruption window. |
| RV-380 | F-1 | major | review |  | Should the design preserve operator-set handler fields? |
| RV-380 | F-2 | major | demonstrate |  | The real decoder must round-trip the cross-language wire. |
| RV-389 | F-9 | major | demonstrate |  | Creation’s judgement must connect to the proposed change row. |
| RV-389 | F-13 | major | control |  | A parser-rejection mutant must fail the compatibility check. |
| RV-389 | F-14 | major | control |  | A suppressed invalidation row must fail the agreement check. |
| RV-389 | F-16 | major | probe |  | A delegated null proposal bypasses the stated refusal behavior. |
| RV-389 | F-17 | major | none | neither | Does shipped text expose an internal identifier to host users? |
| RV-392 | F-1 | major | demonstrate |  | The rendered run’s own slice status must govern its selection. |
| RV-392 | F-3 | major | owner-fix | duplicate | Two title-reading paths disagree about record validity. |

## Counts

| route | count |
|---|---:|
| review | 4 |
| demonstrate | 5 |
| probe | 6 |
| control | 9 |
| owner-fix | 8 |
| none | 6 |

Second labels: `duplicate` 1, `stale` 7, `neither` 6.

## Hesitations

I hesitated between two routes for 5 findings:

- RV-369 F-6: `review` / `probe`
- RV-372 F-13: `demonstrate` / `none`
- RV-372 F-14: `probe` / `none`
- RV-381 F-6: `review` / `demonstrate`
- RV-389 F-9: `demonstrate` / `control`

Near `none` but assigned a route anyway: 2 findings: RV-372 F-13 (`demonstrate`) and RV-372 F-14 (`probe`).

## Caveats

- The CLI showed 47 blocker/major findings across the 24 ledgers, matching the stated total.
- Eight findings were ignored because a route field was present or the disposition/response named a route: RV-366 F-1; RV-373 F-1; RV-317 F-1 and F-3; RV-321 F-2; RV-324 F-1 and F-2; RV-389 F-8.