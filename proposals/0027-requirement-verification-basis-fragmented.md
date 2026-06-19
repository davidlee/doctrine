---
seq: 0027
scope: spec
target: requirement verification basis (refines 0019; ties 0011)
confidence: med
reversible: yes (read-only analysis; nothing authored)
---
## What
"Is this requirement verified?" has **no single authoritative answer** — the basis
is split across three *partial* surfaces, none complete:
1. **Inline `acceptance_criteria`** on the requirement — ~52% empty (proposal 0019).
2. **The coverage matrix** (per-slice `coverage.toml`, REQ↔VT, SL-057) — present on
   **only 22 of 121 slices (~18%)**. (`find .doctrine/slice/*/coverage.toml | wc -l`
   = 22.) The other 99 slices — mostly pre-SL-057 — carry no coverage.toml.
3. **Slice exit criteria** (VT/VA/VH) — checked at execution/audit, not stored as a
   queryable per-requirement verification record.

This **corrects an overstatement in my own proposal 0019**, which leaned on "verification
rides the coverage matrix" to argue inline AC is merely supplementary. It does ride
coverage — but coverage covers ~18% of slices. So for a large fraction of
requirements, verification is asserted (if at all) only through inline AC (half
empty) or slice exit criteria (not stored per-requirement). There is no surface that
answers, per `REQ-NNN`: "verified by AC, by coverage, by a passing slice exit, or
not at all?"

Caveat (why med, not high): the 18% coverage figure is largely **incremental
adoption** — SL-057 introduced coverage; the 99 uncovered slices mostly predate it
and many are closed. Backfilling historical coverage is likely unwarranted. So this
is **not** "99 slices are broken"; it's "the verification basis is *fragmented*, and
nothing rolls the three surfaces into one per-requirement verdict." That fragmentation
is the finding, not the counts.

The graph-topology angle: requirements are nodes; verification is the property a team
most wants to query over them ("what's unverified?"). Today that query is
unanswerable without manually unioning three surfaces — the same shape as proposals
0011 (scattered checks) and 0019 (the AC contract).

## Options
1. **Define a per-requirement verification rollup** — a derived view that, for each
   `REQ-NNN`, reports the strongest verification it has (coverage > slice-exit > AC >
   none) and flags `none`. Tradeoff: turns three partial surfaces into one queryable
   verdict; pure derivation over existing data; the design work is the precedence
   rule + what "slice-exit verified" means for a requirement not in coverage.
2. **Settle the AC contract first (0019), then revisit.** Decide whether inline AC is
   expected; that narrows the question before building a rollup. Tradeoff: sequences
   the decisions cleanly; defers the unified view.
3. **Leave fragmented.** Tradeoff: zero work; "what's unverified?" stays a manual,
   three-surface union — and the requirement corpus's verification state is opaque.

## Recommendation
Option 1, sequenced after 0019's AC-contract decision and folded into the same
"verification hygiene" conversation as 0011 (doctor): a derived per-requirement
verification rollup (`coverage > slice-exit > AC > none`) is the artifact that makes
"what's unverified?" answerable, and it's pure derivation over data doctrine already
holds (coverage.toml, slice criteria, requirement AC). Don't backfill historical
coverage; do make the *current* verification state legible. This is the requirement-
tier analog of the consumption-surface thesis (0014): the data exists; the query
doesn't.

Decisions deferred to YOU:
- (a) **is a unified verification rollup wanted**, or is per-surface inspection the
  intended ergonomics?
- (b) **precedence & semantics** — does a passing slice-exit count as "verified" for
  a requirement absent from coverage? how do AC (human-asserted) and coverage
  (machine-checked) rank?
- (c) confirm the framing: is 18% coverage purely incremental-adoption (my read), or
  is forward coverage also lagging on *new* slices? (worth a glance at recent slices.)

## Next doctrine move
```
# confirm the three-surface split (read-only):
find .doctrine/slice/*/coverage.toml | wc -l         # 22 of 121
grep -rlc 'acceptance_criteria = \[\]' .doctrine/requirement/[0-9]*/*.toml | wc -l
doctrine coverage show SPEC-001                        # the matrix view, where it exists

# capture the rollup (NOT executed — fence forbids backlog transition):
doctrine backlog new improvement "Per-requirement verification rollup: derive one \
  verdict per REQ from coverage > slice-exit > acceptance_criteria > none; answers \
  'what is unverified?' across the three partial surfaces. Pure derivation; fold \
  into doctrine doctor (0011); sequence after the AC-contract decision (0019)" \
  --tag area:coverage --tag area:spec
```
(Verbs described, NOT executed.)

## Illustration (optional)
None — a derived view over existing surfaces; the design question is the precedence
rule (b), which a speculative diff would prejudge.
