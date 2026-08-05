# DEC-151: Synthetic-corpus goldens pin the mechanics; the acceptance specimen is verified by agent

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The question was half-answered before it was asked

`SL-246`'s `inq-7` worried about freezing a live, mutable corpus into a golden.
The suite already solved that: `tests/e2e_inspect_golden.rs` hand-seeds a
synthetic corpus in a temp dir, because *"`inspect` reads only authored TOML —
no clock, no rng — so a hand-seeded corpus with fixed bytes yields byte-exact
output."*

What was genuinely open is narrower — the slice's own closure intent names
`SL-244` as the acceptance specimen and asks whether the output is *worth what
it costs*. That is not a golden-shaped claim.

## The split

**Fixture-provable, so goldened.** Tier filtering, source-kind selection, dedup
across two inbound labels, the two distinct empty markers, and `skip` byte
identity. The fixture needs: a filled `DEC`, an unfilled one, a `CPT`, a record
reachable under two labels (the `EVD-012` case), and a backlog item plus a review
as inbound noise.

**A judgement, so attested by agent.** Whether reading `SL-244` at `facets`
surfaces the rulings at a price a working agent would pay. Dressing that as a
test would be pretending; the verification taxonomy has a by-agent mode for it.

## The live-corpus invariant test, and why it lost

Declined on inspection, not on unavailability — recorded here so it is not
re-litigated from scratch.

**The plumbing exists.** `common::repo_root()` is documented *"for the few
goldens that read the real tree"*, and `tests/e2e_relation_migration_storage.rs`
reads the committed `.doctrine/` directly to assert on-disk invariants. Its
docstring even argues the general case: *"Render goldens are necessary but NOT
sufficient… this test is the real migration oracle."*

**Half of it is the arm already rejected.** *"`skip` is byte-identical to
today's"* requires a stored baseline of today's output to compare against — a
live-corpus golden, with all of the rot that arm was rejected for.

**The other half re-proves a structural guarantee.** *"`facets` ⊆ `full`"* holds
because [[DEC-150]] puts both levels behind one field-order table with a tier
filter. There is no code path on which they can disagree. A test asserting it
catches not a bug the design permits, but a refactor that abandons the design —
which the goldens catch anyway.

**And it has a real brittleness mode.** A line-wise subsequence assertion across
the whole corpus goes red the day someone writes a multi-line TOML string into a
facet. A corpus edit reddening the build is the worst kind of failure: real,
unhelpful, and not about the code.

The precedent test earns its live read because on-disk TOML shape *cannot* be
proven from a fixture — a migration either landed on the real files or it did
not. Rendered-output relations have no equivalent property.

Shapes [[SL-246]]. Resolves that slice's `inq-7`.
