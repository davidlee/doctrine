# DEC-144: VT-2's width clause splits to VT-4

## The question

SL-244 PHASE-07's `VT-2` mandated three clauses in
`src/design_run/render/envelope.rs`: the severity summary counts every
non-terminal finding; it omits itself entirely when all four counts are zero;
and it is **wider than the gating predicate** — an `answered` blocker is counted
outstanding here and does not hold the run's `reviewing → locked` edge.

The first two are render facts and land in that file. The third is not a render
fact at all. It is a comparison between two predicates over one ledger, and
`envelope.rs` holds neither: it is leaf-tier, it may not import `review`
(ADR-001), and the counts reach it as an opaque argument with no ledger behind
them. A test there could assert that four integers render; it could not assert
that they were derived by a wider filter than the one the gate uses, because it
cannot ask the gate's predicate anything.

## The decision

Strike the width clause from `VT-2`, both in its `expects` and from its
`keywords`, and append `VT-4` carrying it with `test_file = "src/review.rs"`.
The clause is rehoused, not weakened or waived.

## Why this shape and not another

**Not a widened import.** Letting `envelope.rs` reach `review` to make the
comparison would invert the layering the whole `OutstandingBySeverity` /
`OutstandingCounts` pairing exists to preserve — the record is the leaf's, the
predicate is `review`'s, and the command tier joins them. Buying one test
assertion with that edge is the worst available trade.

**Append, not renumber.** Criteria ids are immutable and edits append. A new
`VT-4` is the sanctioned edit; amending `VT-2`'s text with a dated note is the
form this slice's `VT-5`/`VT-6` split already carries.

**Not a relocated `VT-2`.** Moving the whole criterion to `src/review.rs` would
strand the two render clauses at a tier that cannot render anything. The
criterion genuinely spans two tiers, so it becomes two criteria.

## What it costs

One criterion's evidence sits in a different file from its two siblings, and a
reader auditing PHASE-07's verification has to visit both. That is the honest
price of a criterion whose clauses were never at one altitude.

## Consequences

- `VT-4`'s test takes both halves off **one** `read_pass_facts` call rather than
  calling the two predicates separately, so it pins the divergence surviving the
  shared reader — which is the thing that could actually regress once `D3` made
  them share a parse.
- No selector change was owed: `src/review.rs` is already scope-relevant to this
  slice, and PHASE-07's close-out declares it as a conformance selector
  regardless.

Related: [[DEC-141]] — the same amendment class, one phase earlier.
