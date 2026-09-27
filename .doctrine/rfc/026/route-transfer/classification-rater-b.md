# Research Brief: Does the five-route taxonomy transfer to audit/reconciliation and code-review ledgers?

## Answer
It transfers only partially. Across the 47 all-severe (blocker/major) findings on the 24 named ledgers, `owner-fix` absorbs the plurality — 21 of 47 (44.7%) — because reconciliation and code-review are dominated by two-accounts-of-one-fact disagreements (design/decision/governance text vs shipped code). Every route in the set is exercised at least once, but the distribution is lopsided: `probe` appears only twice (both on RV-317, the one genuinely adversarial code-review) and `review` only four times, all reconciliation. Five findings (10.6%) are honestly `none`: a crash-recovery property, a test-harness hermeticity defect, a verification-venue question, an ambient-environment gate failure, and an unreachable decision record. The taxonomy does not silently fail; it just has no clean term for durability, harness/isolation, and record-reachability questions, which is where `none` clusters.

## Evidence
- Population verified by reading each ledger's structured findings via `./target/debug/doctrine review show RV-NNN --json` (never raw files). All 24 ledgers fetched; severity histogram over their `finding[]` arrays is exactly **3 blocker + 22 major = 25** for the reconciliation facet and **5 blocker + 17 major = 22** for code-review facet — matching the stated 47.
- Four named reconciliation ledgers (RV-379, RV-383, RV-394, RV-398) and five code-review ledgers (RV-367, RV-375, RV-376, RV-378, RV-393) carry **zero** blocker/major findings — a negative result with a positive control: their populated arrays hold only minor/nit, e.g. RV-379 has 6 findings, severities `major`=0.
- Findings of the class the prompt flags — shipped code disagreeing with a design/DEC/REQ/STD — are identifiable in their own text; e.g. RV-369 F-1 "per design sec-2 and PHASE-03 EX-2"; RV-373 F-1 "drift between two binding sources"; RV-395 F-2 "design sec-1/sec-7 forbid"; RV-389 F-9 "Design .doctrine/slice/264/design.md:137-138 ... implementation ... no term for the judgement"; RV-392 F-3 "STD-003 requires ... to be disclosed." These drove `owner-fix` (or `demonstrate` where the mismatch is a missing *connection*, not a stale *text*).

## 1. Flat table (complete, 47 rows)

| ledger | facet | F-N | severity | route | reason |
|---|---|---|---|---|---|
| RV-366 | reconciliation | F-1 | major | owner-fix | design's four-cell state axis vs code's three: reconcile stale design |
| RV-366 | reconciliation | F-2 | major | owner-fix | design's "no witness" claim vs reproduced witness: qualify text |
| RV-366 | reconciliation | F-3 | major | owner-fix | design code-impact table disagrees with actual diff |
| RV-366 | reconciliation | F-4 | major | owner-fix | selector registry and changed paths are two disagreeing accounts |
| RV-366 | reconciliation | F-6 | major | none | should this proposed, unreachable decision record be settled and linked? |
| RV-369 | reconciliation | F-1 | blocker | demonstrate | shell endpoint measurement never connects to pure rule; pty exercise |
| RV-369 | reconciliation | F-2 | major | demonstrate | one real-pty test at the uncovered shell/pure seam |
| RV-369 | reconciliation | F-6 | blocker | review | accept or bound the unbounded whole-corpus render commitment |
| RV-369 | reconciliation | F-7 | major | review | accept q=2 silent image-drop tradeoff and consequences |
| RV-372 | reconciliation | F-1 | major | owner-fix | ISS-462 names two items; renumber duplicate, verify owner |
| RV-372 | reconciliation | F-3 | major | owner-fix | doc comment / design mandate and code disagree about body read |
| RV-372 | reconciliation | F-4 | major | owner-fix | design's ~30% saving claim vs measured 11.5%: reconcile |
| RV-372 | reconciliation | F-5 | major | review | human VH-1 product verdict: accept the feature or not |
| RV-372 | reconciliation | F-6 | major | owner-fix | STD-003 and the undisclosed withheld facet tier disagree |
| RV-372 | reconciliation | F-13 | major | none | can the handed-back worktree be verified where it lands? |
| RV-372 | reconciliation | F-14 | blocker | none | does the test harness stay hermetic, not corrupt shared repo state? |
| RV-372 | reconciliation | F-15 | major | owner-fix | migration's named population missed a third; sweep the class |
| RV-373 | reconciliation | F-1 | major | owner-fix | DEC-263 and locked design/shipped text give two scopes |
| RV-381 | reconciliation | F-5 | major | owner-fix | governance records are stale accounts of the shipped surface |
| RV-381 | reconciliation | F-6 | major | review | accept the known REQ-018 formatter gap and its consequences |
| RV-387 | reconciliation | F-4 | major | none | is the close gate independent of ambient jail environment? |
| RV-390 | reconciliation | F-7 | major | demonstrate | exercise covered-node scoping on a real run, not empty map |
| RV-395 | reconciliation | F-1 | major | control | acceptance check compared wrong key universe; won't catch non-delivered keys |
| RV-395 | reconciliation | F-2 | major | owner-fix | design forbids src/** edits; shipped code made them |
| RV-395 | reconciliation | F-13 | major | owner-fix | materialised shipped corpus is a stale account of its sources |
| RV-317 | code-review | F-1 | blocker | probe | stated threat model; hostile record reproduces row injection |
| RV-317 | code-review | F-2 | major | probe | malformed uid panics: does the read path withstand it? |
| RV-317 | code-review | F-3 | major | demonstrate | diagnostics computed but never connected to the render surface |
| RV-321 | code-review | F-1 | blocker | control | admission check must reject an over-bound payload-term candidate |
| RV-321 | code-review | F-2 | major | owner-fix | copied literals duplicate the derived single source; collapse |
| RV-321 | code-review | F-3 | major | control | constructor-bypass test must reject alternate construction classes |
| RV-324 | code-review | F-1 | blocker | demonstrate | accepted proposals never connect to checkpoint planning |
| RV-324 | code-review | F-2 | blocker | owner-fix | DEC-092's admitted residual and undetectable overwrite disagree |
| RV-324 | code-review | F-3 | major | owner-fix | DEC-085 and the prefix-merge parser disagree |
| RV-324 | code-review | F-4 | major | owner-fix | DEC-085 and imported-prose provenance disagree |
| RV-342 | code-review | F-1 | major | control | tautological test would not notice declaration/construction mismatch |
| RV-342 | code-review | F-4 | major | none | is the checkpoint's crash window recoverable? |
| RV-380 | code-review | F-1 | major | owner-fix | design's "owned fields" and canonicality comparison disagree |
| RV-380 | code-review | F-2 | major | demonstrate | TS↔Rust neutral wire never round-tripped; stub ignores argv |
| RV-389 | code-review | F-8 | major | control | VT-6's variant-deletion negative control is absent |
| RV-389 | code-review | F-9 | major | demonstrate | creation's judgement never connects to a change row |
| RV-389 | code-review | F-13 | major | control | compatibility test never parses a stored snapshot |
| RV-389 | code-review | F-14 | major | control | agreement test never covers sufficiency invalidation |
| RV-389 | code-review | F-16 | major | control | null guard must reject null on the delegate route too |
| RV-389 | code-review | F-17 | major | owner-fix | POL-002 and shipped prompt/refusal text disagree |
| RV-392 | code-review | F-1 | major | owner-fix | directory and snapshot give two disagreeing slice accounts |
| RV-392 | code-review | F-3 | major | owner-fix | partial and canonical reads disagree about a cited record |

## 2. Per facet

**Reconciliation (25 severe)**
| route | count |
|---|---|
| owner-fix | 13 |
| review | 4 |
| demonstrate | 3 |
| control | 1 |
| probe | 0 |
| **none** | **4 (16.0%)** |

**Code-review (22 severe)**
| route | count |
|---|---|
| owner-fix | 8 |
| control | 7 |
| demonstrate | 4 |
| probe | 2 |
| review | 0 |
| **none** | **1 (4.5%)** |

## 3. Across both facets

- Absorbing routes: `owner-fix` (21), `control` (8), `demonstrate` (7); `review` (4) and `probe` (2) are marginal.
- The single most common route, `owner-fix`, takes **21/47 = 44.7%**.
- Distinct routes used: **5 of 5** (every route fires at least once). `none` count **5/47 = 10.6%**.

## 4. Hesitation

Between two routes — 13 occurrences:
- RV-369 F-2: `demonstrate` vs `control` (test at the uncovered shell seam).
- RV-372 F-6: `owner-fix` vs `review` (design work vs accepting the projection).
- RV-373 F-1: `owner-fix` vs `review` (two binding sources both needing an act).
- RV-381 F-5: `owner-fix` vs `review` (stale governance vs approving REVs).
- RV-381 F-6: `review` vs `owner-fix` (accepted gap vs stale REQ-018 text).
- RV-390 F-7: `demonstrate` vs `control` (real-run exercise vs check-noticing).
- RV-395 F-1: `control` vs `owner-fix` (wrong-universe check vs stale citation).
- RV-317 F-2: `probe` vs `control` (panic defect vs missing negative case).
- RV-321 F-1: `control` vs `owner-fix` (admission must reject vs rule not enforced).
- RV-324 F-1: `demonstrate` vs `owner-fix` (parts not connected vs code vs design).
- RV-342 F-4: `none` vs `control` (crash window vs unreachable fault seam).
- RV-389 F-9: `demonstrate` vs `owner-fix` (missing term vs design conformance).
- RV-366 F-6: `none` vs `review` (record reachability vs accepting the decision).

Near `none` but assigned a route — 4 occurrences: RV-372 F-6, RV-317 F-3, RV-390 F-7, RV-324 F-1. Each is a defect whose question ("is this projection complete/connected?") sits close to `none`'s "no route asks this," but the finding names a specific existing part that should carry it, which pushed `demonstrate`/`owner-fix`.

## 5. Caveats (≤10 lines)
- Routes were assigned from finding title + detail + disposition text only; no taxonomy document was consulted.
- Where a finding is code-vs-design/governance, I let that decide (mostly `owner-fix`; `demonstrate` where the mismatch is a missing connection).
- `owner-fix`'s definition ("two accounts of one fact") is broad enough to swallow most conformance findings; that breadth is the main transfer risk.
- Some "demonstrate" calls name code fixes already landed at audit, not thin prototypes; the route labels the question, not the fix verb.
- Severity/disposition are the ledgers' own; I did not re-adjudicate them.
- Duplicate `F-1`/`F-3` ids recur across ledgers; ids are ledger-local, as cited.
- Whether the taxonomy was *intended* to cover audit ledgers is unknown; this is an empirical fit test only.
- No recommendation is made, per instruction.

## Judgement
The honest reading is that the taxonomy is a design-review instrument being asked to file work it was not shaped for, and the strain shows in three places. First, `owner-fix` is doing double duty. Its question — "do two accounts of one fact disagree?" — fits both a genuine duplicate (RV-372 F-1's two `ISS-462` items; RV-321 F-2's copied literals) and the far larger class of "a design/decision/spec text says X and the shipped code does Y" (RV-366 F-1..F-3, RV-373 F-1, RV-389 F-9, RV-392 F-1/F-3, and more). Those are not the same failure: a duplicate is resolved by deleting one account, whereas a stale doc is resolved by editing the doc. Collapsing both under `owner-fix` is defensible only because the remedy ends the same way — "verify the surviving owner" — but it costs resolution: a reader counting `owner-fix` cannot tell duplicate from drift, and 44.7% of the population disappears into one bucket. That is the taxonomy's largest transfer cost, and it is structural, not incidental.

Second, `control` and `demonstrate` split the code-review facet cleanly along a line the reconciliation facet barely crosses at all. Code-review produced 7 `control` and 4 `demonstrate` and 2 `probe`; reconciliation produced 1 `control`, 3 `demonstrate`, 0 `probe`. That is a real signal: code-review findings ask "would this check have caught it?" and "do these parts connect?" — verification questions — while reconciliation findings ask "which account is stale?" — bookkeeping questions. So the taxonomy does transfer, but it transfers *facet-shifted*: the instrument routes migrate to code-review, and `owner-fix` migrates to reconciliation. A single closed set covering both means each facet under-uses half the table.

Third, `none` is not noise — it is a consistent category with a shape. All five `none` findings ask about a property no route names: recoverability of a crash window (RV-342 F-4), hermeticity of a test harness against shared state (RV-372 F-14), whether a delivered artefact is verifiable where it lands (RV-372 F-13), whether the close gate is independent of the ambient environment (RV-387 F-4), and whether an unreachable decision record should be settled (RV-366 F-6). Four of these are *audit/infrastructure* questions — durability, isolation, verification venue, environment — and the fifth is record lifecycle. None is a design commitment, a disputed connection, an adversary, a check, or a duplicate. These are exactly the audit-specific concerns a design-review taxonomy would not have needed, so their landing on `none` is the clearest evidence that the taxonomy has a real seam here rather than a classification failure I should have strained past. The two `probe` findings both sit on RV-317, the only ledger among the 24 whose brief states a threat model and a reproduced hostile probe; `probe` is otherwise unused, which suggests it is a design-ledger route (security threat-models) that rarely survives into audit.

## Limits
- I did not read any forbidden material (`.doctrine/rfc/026/p10-trial/**`, RFC-026 E12); the route *definitions* used are exactly the table supplied, so my reading of each route's intent is inferred, not verified against the convention's own document. Marked inference.
- Classification is judgement on finding prose; reasonable reviewers could move several borderline rows (the 13 hesitations are the honest ones; there may be others I resolved without recording). I did not cross-check against how the trial actually routed these findings, since that would require the excluded corpus.
- The "near-none" count is my subjective threshold; a different tolerance would change it.
- I verified the population and the zero-severe ledgers through the CLI, but I did not independently re-derive any finding's underlying code/line claims.
- No `probe` appears in reconciliation and no `review` in code-review; whether that is a property of the taxonomy or of these particular ledgers cannot be settled from 24 samples. Unknown.
