# Research Brief: Routing the 47 severe findings on the named design-review ledgers

## Answer

I classified all 47 blocker/major findings from their own detail text and disposition, against the five-route convention as it is shipped in `install/design-prompts/reviewing.md:118-141` and `install/review-ledger.md:238-259`. The population is exactly 47; I found no more and no fewer. The distribution is heavily weighted to `owner-fix` (19) and `control` (10), which is what a corpus dominated by reconciliation/audit ledgers should look like: most findings are either "two records disagree" (design.md vs the tree, registry vs diff, copied literals vs derivation) or "the check cannot notice the fault" (tautological tests, missing negative controls, mutation survivors). Three findings get `none`: RV-366 F-6 (an unsettled, unlinked record — its question is discoverability, not one of the five), RV-372 F-13 (hand-back provisioning fit to gate), and RV-389 F-17 (a POL-002 shipped-surface violation). I used the convention's tiebreak where two routes genuinely fit, and flagged every such case rather than resolving it silently. My confidence is moderate-to-high on the `probe`/`control`/`demonstrate` split and on the `duplicate`-vs-`stale` second labels; it is lowest on the owner-fix-vs-review boundary for design-omission findings (RV-372 F-6, RV-381 F-5).

## Evidence

- Routing convention and closed set: `install/design-prompts/reviewing.md:111-141` (the five-route table, "There is no default", the tiebreak order `owner-fix, control, probe, demonstrate`, instrument routes not repaired in prose); `install/review-ledger.md:238-259` (route is a separate axis from disposition; instrument close deferred).
- `DEC-263` (`doctrine knowledge show DEC-263`) confirms the route is the instrument-of-settlement axis and that `demonstrate|probe|control` findings become phase criteria; it also confirms `review`/`owner-fix` are "repaired in prose".
- Route table quoted from the task is the same closed set; I did not read `.doctrine/rfc/026/`, `.doctrine/slice/270/`, `QUE-224`, or `QUE-225`, and I ignored `route`/`route:` mentions inside dispositions and responses.
- Population verified: 24 named ledgers; severe-finding counts sum to 47 (`RV-366`5, `RV-369`4, `RV-372`8, `RV-373`1, `RV-381`2, `RV-387`1, `RV-390`1, `RV-395`3, `RV-317`3, `RV-321`3, `RV-324`4, `RV-342`2, `RV-380`2, `RV-389`6, `RV-392`2).
- Note: `RV-321 F-1` is stored `blocker` but its own response reduces severity to `major`; I used the stored severity for the row.

## 1. Table

| ledger | F-N | severity | route-or-none | second-label | reason |
|---|---|---|---|---|---|
| RV-366 | F-1 | major | owner-fix | stale | design's four-cell count lags the three-cell code |
| RV-366 | F-2 | major | owner-fix | stale | "no witness" claim lags a reproduced witness |
| RV-366 | F-3 | major | owner-fix | stale | code-impact table lags the actual diff |
| RV-366 | F-4 | major | owner-fix | stale | selector registry lags changed design-target paths |
| RV-366 | F-6 | major | none | stale | asks whether an unsettled, unlinked record is reachable |
| RV-369 | F-1 | blocker | demonstrate | | shell's terminal feed to the pure rule is unexercised |
| RV-369 | F-2 | major | control | | suite cannot convict a broken `-X` |
| RV-369 | F-6 | blocker | review | | choosing the resource bound is a design decision |
| RV-369 | F-7 | major | review | | conscious tradeoff: keep `q=2` despite silent rejection |
| RV-372 | F-1 | major | owner-fix | duplicate | one ISS id minted for two different items |
| RV-372 | F-3 | major | owner-fix | stale | doc comment denies the body read the code performs |
| RV-372 | F-4 | major | owner-fix | stale | design's ~30% saving contradicts measured 11.5% |
| RV-372 | F-5 | major | review | | accept the feature despite the negative product verdict |
| RV-372 | F-6 | major | owner-fix | stale | design's marker model lacks the withheld state |
| RV-372 | F-13 | major | none | neither | asks whether the hand-back is fit to be gated |
| RV-372 | F-14 | blocker | probe | | inherited `GIT_DIR` defeats the test fixture |
| RV-372 | F-15 | major | owner-fix | stale | migration population lags the corpus sweep |
| RV-373 | F-1 | major | owner-fix | stale | shipped verify scope lags DEC-263's instrument-only scope |
| RV-381 | F-5 | major | owner-fix | stale | governance docs lag the shipped surface |
| RV-381 | F-6 | major | review | | conscious acceptance of the REQ-018 formatter gap |
| RV-387 | F-4 | major | probe | | ambient jail env defeats the reserve suite |
| RV-390 | F-7 | major | control | | empty-map test never exercises covered-node scoping |
| RV-395 | F-1 | major | control | | key check compares source corpus, not delivered corpus |
| RV-395 | F-2 | major | owner-fix | stale | design's no-src-change claim lags the changed src |
| RV-395 | F-13 | major | control | | no gate compares materialised corpus to sources |
| RV-317 | F-1 | blocker | probe | | hostile record injects escapes and a forged row |
| RV-317 | F-2 | major | probe | | malformed non-ASCII uid panics the read verbs |
| RV-317 | F-3 | major | owner-fix | stale | CLI rendering lags the design's diagnostics contract |
| RV-321 | F-1 | blocker | probe | | corrupt design.toml bypasses admission bounds |
| RV-321 | F-2 | blocker | owner-fix | duplicate | copied 264/from/to/reason literals duplicate their sources |
| RV-321 | F-3 | major | control | | spelling-count test cannot catch constructor bypass |
| RV-324 | F-1 | blocker | demonstrate | | accepted proposals never reach checkpoint effect protocol |
| RV-324 | F-2 | blocker | review | | race window consciously tolerated for v0.0.1 |
| RV-324 | F-3 | major | probe | | lookalike citation text silently merges imported node |
| RV-324 | F-4 | major | owner-fix | stale | node provenance lags the section's fingerprint |
| RV-342 | F-1 | major | control | | declared kinds and construction sites never compared |
| RV-342 | F-4 | major | control | | fault seam cannot reach the unrecoverable window |
| RV-380 | F-1 | major | owner-fix | stale | implementation's field rule lags design's owned-fields rule |
| RV-380 | F-2 | major | demonstrate | | TS↔Rust neutral wire never round-tripped |
| RV-389 | F-8 | major | control | | stored-snapshot parse lacks a negative control |
| RV-389 | F-9 | major | owner-fix | stale | design says NodeCreated carries judgement; code omits it |
| RV-389 | F-13 | major | control | | parser-rejection mutant survives the compatibility tests |
| RV-389 | F-14 | major | control | | suppressed-row mutant survives the agreement test |
| RV-389 | F-16 | major | probe | | delegation route drops null, bypassing the refusal |
| RV-389 | F-17 | major | none | neither | asks whether a shipped surface violates POL-002 |
| RV-392 | F-1 | major | owner-fix | duplicate | directory slice and run.slice disagree on the run's slice |
| RV-392 | F-3 | major | owner-fix | duplicate | partial title read duplicates the canonical validated read |

## 2. Counts

- By route: `owner-fix` 19, `control` 10, `probe` 7, `review` 5, `demonstrate` 3, `none` 3. Total 47.
- Second labels (owner-fix + none = 22 rows): `stale` 16, `duplicate` 4, `neither` 2.

## 3. Hesitations

Hesitated between two routes (17 findings):
- RV-366 F-6: `none` vs `owner-fix/stale` — one account, no disagreeing second.
- RV-369 F-1: `demonstrate` vs `control` — real-pty feed as connection vs check.
- RV-372 F-6: `owner-fix/stale` vs `review` — incomplete model vs design judgement.
- RV-372 F-13: `none` vs `demonstrate` — provisioning contract vs parts connecting.
- RV-372 F-14: `probe` vs `control` — hostile env vs fixture's discrimination.
- RV-387 F-4: `probe` vs `control` (also `none`) — ambient-env vs test hermeticity.
- RV-317 F-3: `owner-fix/stale` vs `demonstrate` — contract drift vs unconnected capability.
- RV-324 F-4: `owner-fix/stale` vs `none` — two provenance accounts vs missing field.
- RV-380 F-1: `owner-fix/stale` vs `control` — conflicting rule vs missing regression.
- RV-380 F-2: `demonstrate` vs `control` — round trip vs drift detector.
- RV-389 F-17: `none` vs `owner-fix` — policy violation vs internal-id duplication.
- RV-395 F-13: `control` vs `owner-fix/stale` — missing drift gate vs stale corpus.
- RV-324 F-1: `demonstrate` vs `probe` — unreached protocol vs bogus adopt record.
- RV-324 F-3: `probe` vs `control` — lookalike input vs untested boundary case.
- RV-317 F-2: `probe` vs `control` — hostile-input panic vs absent non-ASCII test.
- RV-395 F-1: `control` vs `owner-fix/stale` — wrong test universe vs stale citations.
- RV-372 F-5: `review` vs `none` — accept-feature judgement vs product evaluation.

Near `none` but routed anyway: RV-369 F-1 (demonstrate), RV-387 F-4 (probe), RV-317 F-3 (owner-fix), RV-324 F-4 (owner-fix), RV-380 F-1 (owner-fix), RV-395 F-13 (control), RV-389 F-9 (owner-fix), RV-372 F-6 (owner-fix).

## 4. Caveats

- I classified blind: no route field was present in these ledgers' JSON; any `route:` token inside a disposition/response was ignored, per instruction.
- Older ledgers may encode routes as `route:` prefixes; I did not use those as a key and did not read RFC-026, slice 270, or QUE-224/225.
- `owner-fix` here means "two accounts of one fact disagree and one is corrected"; several design-conformance rows (RV-317 F-3, RV-389 F-9) are contract-vs-code rather than literal duplicate records — the second label says which.
- `stale` (16) is doing most of the work in the second-label column; if the distinction is "duplicate record vs lagging record", some of those could reasonably read `neither`.
- The `review`/`owner-fix` boundary is the weakest call: RV-372 F-6 and RV-381 F-5 both require a record edit but arguably a design judgement first.
- RV-321 F-1's stored severity (`blocker`) disagrees with its own response, which reduces it to `major`; row uses stored severity.
- I did not verify the underlying code claims in the findings; routing is from the finding text and disposition only.
- Unknown: whether the exercise's intended answer treats environment/provisioning findings (RV-372 F-13, RV-387 F-4) as `none` rather than a fit.
- What would change the answer: a stated rule that contract-vs-code drift is `review` rather than `owner-fix`; that would move several rows.
