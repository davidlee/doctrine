---
seq: 0019
scope: spec
target: requirement corpus — `acceptance_criteria` field (REQ kind)
confidence: med
reversible: yes (proposal only; read-only analysis — nothing authored)
---
## What
The requirement corpus is **inconsistent in whether requirements carry inline
acceptance criteria**: of ~304 real requirements (`.doctrine/requirement/NNN/`),
**roughly half (~157, ~52%) have `acceptance_criteria = []`** while the other half
carry crisp criteria. The field is **production-rendered** — `spec show` prints it
when non-empty (`src/spec.rs:469-471`: `if !req.acceptance_criteria.is_empty() { …
for c in &req.acceptance_criteria … }`). So a reader walking a spec sees uneven
quality: some requirements display concrete acceptance criteria, ~half display
none.

Important framing (so this isn't over-claimed): this is **not** a verifiability
crisis. Doctrine's actual verification basis is the coverage matrix (REQ↔VT, the
`coverage*` engine) plus slice-level `VT/VA/VH` exit criteria — not this inline
field. `acceptance_criteria` is `#[serde(default)]` optional (`src/requirement.rs:169`)
and supplementary. So the empty-half are still verifiable *through coverage*; what's
inconsistent is the **inline, human-readable acceptance statement on the requirement
itself**.

The open question is therefore *what this field is for*:
- If inline acceptance criteria are **expected** on every requirement (the spec-product
  intent is that a requirement states how you'd know it's met), then ~157 empties are a
  **backfill/quality gap** and the renderer's uneven output is the symptom.
- If the field is **supplementary-by-design** (verification lives in coverage; inline
  AC is an optional convenience), then the gap is smaller — but then an always-half-empty,
  rendered field is mild noise, and nothing signals authors when to fill it.

No backlog item tracks this (a scan for acceptance/criteria/verifiable finds only
IDE-008, which is about executable phase gates — unrelated). At ~52% it is the
single largest consistency asymmetry I found in an otherwise clean corpus (FK/orphan
clean, lineage edges resolve, etc.).

## Options
1. **Declare inline AC optional-by-design; document it.** State (in the REQ kind
   spec / authoring guidance) that requirement verification rides the coverage
   matrix and inline `acceptance_criteria` is an optional elaboration. Tradeoff:
   zero backfill; removes the "is this incomplete?" ambiguity; the rendered-uneven
   output stays but is understood as intentional.
2. **Declare inline AC expected; drive consistency forward-only.** Keep existing
   empties, but have requirement authoring (and/or `spec validate`) **warn** on a
   new requirement with empty AC, so the ratio improves over time without a mass
   backfill. Tradeoff: cheap, ratchet-style; doesn't fix the existing ~157.
3. **Expected + backfill.** (2) plus a campaign to backfill criteria on the existing
   empties. Tradeoff: best corpus quality; expensive (~157 requirements of authoring
   judgement) and arguably redundant with coverage.
4. **Leave as-is.** Tradeoff: zero effort; the rendered field stays half-empty and
   readers can't tell "no criteria authored" from "criteria not needed."

## Recommendation
Decide the field's *contract* first (Option 1 vs 2), because everything else
follows from it — and I lean **Option 1 with a light touch of 2**: treat inline AC
as an optional elaboration (verification authority is coverage + slice VT/VA/VH, per
the verification taxonomy), document that, and have `spec validate` emit an
**advisory** (not failing) note counting empty-AC requirements so the asymmetry is
visible without forcing churn. Rationale: a mass backfill (Option 3) duplicates the
coverage matrix's job; but leaving it undocumented (Option 4) means every reviewer
re-asks "is this requirement unfinished?" The cheapest high-value move is to *name
the contract* and surface the count, not to fill 157 fields.

Decisions deferred to YOU:
- (a) **is inline `acceptance_criteria` expected or optional?** (the load-bearing
  call — sets everything else). What is the field *for*, given coverage already
  carries verification?
- (b) if optional, **document where** (REQ kind spec / `spec-product` guidance); if
  expected, **forward-only warn vs backfill**.
- (c) should `spec validate` count/report empty-AC requirements (advisory), so the
  ratio is visible in the corpus-health surface (ties to proposal 0011)?

## Next doctrine move
```
# confirm the ratio + that it's rendered (read-only):
grep -rlE 'acceptance_criteria = \[\]' .doctrine/requirement/[0-9]*/requirement-*.toml | wc -l
sed -n '465,475p' src/spec.rs        # the renderer that prints AC when present

# settle the contract (authored guidance / spec) — route it (NOT executed; fence):
/route        # → spec-product/spec-tech guidance edit, or a backlog idea:
doctrine backlog new idea "Clarify requirement acceptance_criteria contract: \
  optional elaboration vs expected-per-requirement; ~52% empty today; verification \
  authority is coverage matrix + slice VT/VA/VH. Consider advisory empty-AC count \
  in spec validate" --tag area:spec --tag area:coverage
```
(Verbs described, NOT executed.)

## Illustration (optional)
None — the substance is the contract question (a) and the ~52% ratio, not a diff.
Backfilling criteria is authoring judgement per requirement, not mechanizable.
