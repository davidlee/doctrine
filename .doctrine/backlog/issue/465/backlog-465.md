# ISS-465: memory search: no default page, no relevance floor, no zero-hit signal

`doctrine memory search` is a ranked surface (BM25 at sort key 2 of 9, `src/retrieve.rs`)
that pages nothing by default. Two consequences, one real defect each:

1. **No default page.** `--limit` carries no default (the help shows one for
   `--offset`, none for `--limit`), so a query emits *every* filter survivor — 565
   rows today, the whole active corpus; 69 when this was filed. A triage call dumps
   the corpus into an agent's context, burying the ranking under the result set it
   was meant to order.
2. **No relevance floor and no zero-hit signal.** When a query's tokens match no
   document, BM25 ties at 0 for every candidate and the 9-key total order falls
   through to keys 4–6 — verification → trust → severity. A genuinely zero-evidence
   query therefore returns the same high-severity memories every time, with nothing
   saying "no document matched".

## What is *not* the defect

The original report below concluded the ranking was broken. It is not: on
distinctive queries the ranker orders correctly, and the headline repro no longer
reproduces — `"black-box CLI golden byte-exact stdout test"` and `"inspect
knowledge facet"` now return disjoint, relevant top rows.

The "identical result set for unrelated queries" observation was **an artefact of
defect 1, not a symptom of the ranker**. Because the row count is query-invariant
when the result set is unpaged, three unrelated queries — including the literal
garbage `zzzqqq nonsense` — all reported the same *count*, which reads as "the
query is ignored". It never was: the ordering varied; the size did not.

That false fingerprint is why this issue, and five friction observation records
around it, describe a ranking defect that is only partly real. Worth stating
plainly in the issue, since the next reader will run the same probe.

## Evidence (live probes, 2026-09-27)

| probe | result |
|---|---|
| `memory search --limit 5 "<anything>"` | `5 of 565` — the `--limit` flag is the only thing that pages |
| `memory search "zzzqqq nonsense"` vs any other query | identical row *count*; different ordering |
| `memory search "zzzqqq nonsense" --limit 4` | the same high-severity/high-trust rows any zero-evidence query returns |
| `memory search "black-box CLI golden byte-exact stdout test"` | top row: the byte-exact-goldens memory — relevant |

## Shape of a fix

- Give `--limit` a default page (the 9-key sort already ranks; the caller wants the
  top N).
- Separate "no document matched" from "matched, ranked low" — a zero-evidence query
  should say so, and probably should not fall back to severity order as if it were a
  result.

Both are read-surface changes; neither touches the ranker or the corpus.

## Original report (2026-09-19, superseded by the analysis above)

`doctrine memory search` returned the **same eleven unrelated rows** for two
unrelated queries — "black-box CLI golden byte-exact stdout test" and "inspect
knowledge facet" — during SL-246 PHASE-03/04. A raw `grep` over
`.doctrine/memory/items/*/memory.toml` found ten relevant memories in one call.

Reading memory raw instead of through the CLI violates a stated rule of the road:
read entities via `show`, never raw TOML. Two separate phase planners violated it
independently in one session, and both were right to — the sanctioned path cost
more and returned less.

**A guardrail that is ten times more expensive and strictly less effective than
violating it will be violated.** The corpus is the asset; a retrieval surface that
cannot find what is in it teaches agents to bypass the surface, and every bypass
erodes the convention that makes the corpus readable at all.

Independently hit by two planners in one slice, plus the same pattern reported in
SL-246 PHASE-01/02 as "`memory retrieve` precision too low for code-level hazards
in two independent planners, forcing memory ids into spawn prompts". The
low-precision symptom spanned **both** `memory search` and `memory retrieve`, in
four planner contexts across two orchestrator legs; that breadth is what the
unpaged-default analysis above explains.

Surfaced during SL-246 PHASE-03/04 (capsule-driver); RFC-011 token-efficiency
benchmarking.

## Compounding finding (SL-246 PHASE-05/06)

`CHR-075` records five memories that are now **actively wrong** about
`doctrine design show` after PHASE-05 moved its default. The two defects compound:
a retrieval surface agents already bypass is also less likely to have its bad rows
noticed and corrected in passing. Fixing retrieval raises the value of fixing
content, and vice versa.
