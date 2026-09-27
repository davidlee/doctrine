# Research Brief: Does the design-review route taxonomy transfer to audit/reconciliation and code-review ledgers?

## Answer

Yes, it transfers to all 46 of 47 severe findings — but only because `owner-fix` is read broadly as *"any disagreement between the shipped code and what the design or a governance record says."* That single reading absorbs 21 of 47 findings (44.7%), almost all of the reconciliation facet. Under a narrower reading of `owner-fix` (duplicate *sources of truth* — two implementations, copied constants, one id naming two items) roughly a dozen of those become `none`, and the taxonomy's fit drops sharply. One finding genuinely has no term: `RV-372`'s `F-13`, which asks whether the handed-back worktree can be built/gated at all. All five routes were used; each facet's distribution differs strongly (reconciliation ≈ 52% owner-fix; code-review ≈ 27% owner-fix + 27% control).

## 1. Complete table (47 rows)

| led | facet | F-N | sev | route | reason (≤12 words) |
|---|---|---|---|---|---|
| RV-366 | recon | F-1 | major | owner-fix | design's four-cell account vs code's three disagree |
| RV-366 | recon | F-2 | major | owner-fix | design's no-witness claim vs reproduced witness |
| RV-366 | recon | F-3 | major | owner-fix | design's code-impact table vs actual diff disagree |
| RV-366 | recon | F-4 | major | owner-fix | selector registry's declared paths vs changed tree disagree |
| RV-366 | recon | F-6 | major | review | audit owes a REV-vs-reconciliation-line design ruling |
| RV-369 | recon | F-1 | blocker | demonstrate | real pty shows shell feed and pure rule don't connect |
| RV-369 | recon | F-2 | major | control | pure tests can't convict broken -X; needs real-seam test |
| RV-369 | recon | F-6 | blocker | review | bound choice and behaviour at it is a design decision |
| RV-369 | recon | F-7 | major | review | accept the q=2 tradeoff and its silent-drop consequence |
| RV-372 | recon | F-1 | major | owner-fix | two backlog items claim one id, ISS-462 |
| RV-372 | recon | F-3 | major | owner-fix | C7 design mandate and doc comment vs code |
| RV-372 | recon | F-4 | major | owner-fix | locked R3's 30% saving claim vs measured 11.5% |
| RV-372 | recon | F-5 | major | review | accept mechanism-met/outcome-unmet; human product verdict |
| RV-372 | recon | F-6 | major | owner-fix | STD-003 disclosure rule vs silently withholding renderer |
| RV-372 | recon | F-13 | major | none | can the handed-back worktree be gated where it lands? |
| RV-372 | recon | F-14 | blocker | probe | inherited GIT_DIR adversary rewrote shared repo config |
| RV-372 | recon | F-15 | major | owner-fix | priced-migration account missed a population; sweep owed |
| RV-373 | recon | F-1 | major | owner-fix | DEC-263 scope vs shipped text; two binding accounts |
| RV-381 | recon | F-5 | major | owner-fix | governance's account of shipped surface vs reality |
| RV-381 | recon | F-6 | major | owner-fix | REQ-018 quoted-attributed rule vs formatter's output |
| RV-387 | recon | F-4 | major | control | isolate ambient env; disprove the gate's false red |
| RV-390 | recon | F-7 | major | control | empty-map substitute can't exercise covered-node scoping |
| RV-395 | recon | F-1 | major | control | key-universe check compared source, not delivered, corpus |
| RV-395 | recon | F-2 | major | owner-fix | design's no-src claim vs two edited src files |
| RV-395 | recon | F-13 | major | control | no gate compares materialised corpus to sources |
| RV-317 | code-rev | F-1 | blocker | probe | hostile record injects rows/ESC; escaper bypassed |
| RV-317 | code-rev | F-2 | major | probe | non-ASCII uid crashes read verbs; hostile input |
| RV-317 | code-rev | F-3 | major | demonstrate | engine diagnostics never connect to a CLI render site |
| RV-321 | code-rev | F-1 | blocker | probe | corrupt design.toml bypasses admission; render unbounded |
| RV-321 | code-rev | F-2 | blocker | owner-fix | copied framing/literal constants vs encoder's real values |
| RV-321 | code-rev | F-3 | major | control | constructor-bypass test counts one spelling; bypasses green |
| RV-324 | code-rev | F-1 | blocker | demonstrate | accepted proposal never connects to checkpoint effect protocol |
| RV-324 | code-rev | F-2 | blocker | review | accept reachable overwrite race as disclosed tolerance |
| RV-324 | code-rev | F-3 | major | control | citation test omits numeric-prefix lookalike QUE-177abc |
| RV-324 | code-rev | F-4 | major | owner-fix | DEC-085 requires fingerprint; node provenance omits it |
| RV-342 | code-rev | F-1 | major | control | payload-term test saturates declared kinds; mismatch invisible |
| RV-342 | code-rev | F-4 | major | probe | SIGINT window bricks recovery; fault seam can't reach |
| RV-380 | code-rev | F-1 | major | owner-fix | design's owned-fields scope vs None-means-absent code |
| RV-380 | code-rev | F-2 | major | demonstrate | TS→Rust wire never round-tripped; stub ignores argv |
| RV-389 | code-rev | F-8 | major | control | no negative control; cited suite parses no snapshots |
| RV-389 | code-rev | F-9 | major | owner-fix | design's NodeCreated judgement vs parent/provenance only |
| RV-389 | code-rev | F-13 | major | control | compatibility tests never parse a stored snapshot |
| RV-389 | code-rev | F-14 | major | control | agreement test covers graph review only, not sufficiency |
| RV-389 | code-rev | F-16 | major | owner-fix | direct apply refuses null; proposal route drops it |
| RV-389 | code-rev | F-17 | major | owner-fix | IMP-483 id resolves to different owners across repos |
| RV-392 | code-rev | F-1 | major | owner-fix | run.slice vs directory-slice accounts disagree on status |
| RV-392 | code-rev | F-3 | major | owner-fix | partial title read vs canonical read disagree |

## 2. Per facet

**Reconciliation (25 severe):** owner-fix 13 · control 5 · review 4 · probe 1 · demonstrate 1 · **none 1 (4.0%)**.
**Code-review (22 severe):** owner-fix 8 · control 6 · probe 4 · demonstrate 3 · review 1 · **none 0 (0.0%)**.

## 3. Across both facets

- owner-fix 21, control 11, review 5, probe 5, demonstrate 4, none 1.
- The single most common route, **owner-fix, takes 21/47 = 44.7%** of the population.
- **5 distinct routes used** — every route in the closed set fired at least once.

## 4. Hesitation and near-`none`

Hesitated between exactly two routes on **18 of 47** findings (38%):
RV-366 F-4 (owner-fix/control), RV-366 F-6 (review/owner-fix), RV-369 F-1 (demonstrate/probe), RV-369 F-2 (control/demonstrate), RV-372 F-6 (owner-fix/review), RV-372 F-13 (none/demonstrate), RV-372 F-14 (probe/none), RV-387 F-4 (control/none), RV-390 F-7 (control/owner-fix), RV-317 F-2 (probe/control), RV-321 F-1 (probe/owner-fix), RV-324 F-1 (demonstrate/owner-fix), RV-324 F-4 (owner-fix/control), RV-342 F-4 (probe/control), RV-380 F-1 (owner-fix/control), RV-389 F-9 (owner-fix/control), RV-389 F-16 (owner-fix/control), RV-389 F-17 (owner-fix/none).

Near-`none` but assigned a route anyway: **4** — RV-369 F-1 (→demonstrate), RV-372 F-14 (→probe), RV-387 F-4 (→control), RV-389 F-17 (→owner-fix). RV-372 F-13 was the one that stayed `none`.

## 5. Caveats

1. The 44.7% owner-fix share is produced by reading "code vs design/governance mismatch" as "two accounts of one fact." The shipped definition (`install/design-prompts/reviewing.md:120-128`) is compatible but does not compel it; a strict duplicate-source reading would move ~12 rows to `none`.
2. Classified from finding text + disposition only. The routed design-review ledgers are in the off-limits trial area, so no empirical route assignments were available for calibration.
3. `RV-372` F-13 is the only honest `none`; it asks "can the handed-back worktree be gated?", a provisioning question the five routes do not pose.
4. Instrument-route deliverability is not assessed: `demonstrate`/`probe`/`control` presuppose a phase criterion, which audit/reconcile ledgers lack (`install/design-prompts/reviewing.md:141-181`). A fit-on-paper may not be actionably routable.
5. Severities and dispositions are as stored; several `verified`/`tolerated` findings were still `major`.
6. All five routes fired, so the closed set is not empty on these facets; `none` is used sparingly, not as an escape hatch.
7. `RV-372` F-14 and `RV-387` F-4 are environment/process defects read as `probe`/`control`; an alternative reading is `none`.
8. Where a design claim and the tree disagree, I did not down-rank severity; the mismatch decided the route (per instruction).
9. `owner-fix`'s remedy clause ("sweep the affected class") fits several reconciliation findings (e.g. RV-372 F-15, RV-366 F-3) more than the "duplicate" half does.
10. Counts reconcile: 25 + 22 = 47.

[RV-389]: route-transfer classification across 24 ledgers (47 severe findings).
