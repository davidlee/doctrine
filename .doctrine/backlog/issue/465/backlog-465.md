# ISS-465: memory search ranking too poor to compete with raw grep

`doctrine memory search` returned the **same eleven unrelated rows** for two
unrelated queries — "black-box CLI golden byte-exact stdout test" and "inspect
knowledge facet" — during SL-246 PHASE-03/04. A raw `grep` over
`.doctrine/memory/items/*/memory.toml` found ten relevant memories in one call.

## Why this is more than an annoyance

Reading memory raw instead of through the CLI violates a stated rule of the
road: read entities via `show`, never raw TOML. Two separate phase planners
violated it independently in one session, and both were right to — the sanctioned
path cost more and returned less.

**A guardrail that is ten times more expensive and strictly less effective than
violating it will be violated.** The corpus is the asset; a retrieval surface
that cannot find what is in it teaches agents to bypass the surface, and every
bypass erodes the convention that makes the corpus readable at all. The same
session had to work around it by pasting memory ids directly into spawn prompts.

## Evidence

- Two distinct, specific queries → one identical eleven-row result set.
- `grep` over the same corpus → ten relevant hits, one call.
- Independently hit by two planners in one slice, plus the same pattern reported
  in SL-246 PHASE-01/02 as "`memory retrieve` precision too low for code-level
  hazards in two independent planners, forcing memory ids into spawn prompts".

That last point matters: the low-precision symptom now spans **both**
`memory search` and `memory retrieve`, in four planner contexts across two
orchestrator legs. It is not one bad query.

## The shape of a fix

Diagnose the ranking before tuning it — an identical result set across unrelated
queries suggests the query terms are barely influencing the score (BM25 over a
field set that does not carry the discriminating text, or a scoring path that
degrades to a constant). Compare against the raw-grep recall as the baseline to
beat.

Surfaced during SL-246 PHASE-03/04 (capsule-driver); RFC-011 token-efficiency
benchmarking.

## Compounding finding (SL-246 PHASE-05/06)

`CHR-075` records five memories that are now **actively wrong** about
`doctrine design show` after PHASE-05 moved its default. The two defects
compound: a retrieval surface agents already bypass is also less likely to have
its bad rows noticed and corrected in passing. Fixing ranking raises the value of
fixing content, and vice versa.
