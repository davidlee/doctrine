# REV REV-042 — Correct the product altitude ladder to three rungs

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

IMP-350 retired `story` from `ProductLevel`, leaving the three-rung ladder
`domain | capability | feature` with `feature` as a recursive floor (RFC-024
half one). The code shipped; **two active requirements on PRD-002 did not
follow it**, and REQ-259 now states an obligation the binary rejects.

The staleness was missed at IMP-350 close because the sweep grepped requirement
`.md` files. Requirement text lives in the `.toml` tier — `description` and
`acceptance_criteria` are structured fields, not prose. A corrected sweep with a
positive control found REQ-259 as the sole governance carrier of `story`, two
rows, nothing else in the corpus.

Both rows are `modify` against live requirements on an active product spec, so
they route through a Revision rather than a direct edit. No `introduce` row is
needed — which matters, because IMP-297 records that `revision change add
--action introduce` is SPEC-only and has no path to a PRD requirement. Framing
both changes as modifications of the requirements that already own the relevant
obligation avoids that limitation rather than working around it.

### Scope boundary

This revision corrects **requirement text only**. It does not:

- give the product ladder a spine spec — `product_level` remains absent from
  SPEC-006's spine enumeration and out of SPEC-017's tech-only scope. That is
  the open structural gap (see `.doctrine/state/spec-coverage-taxonomy.md`
  gap 2), and it is a `/spec-tech` pass, not a REV.
- touch PRD-012 §6, which currently describes rank-adjacency for *both* ladders
  from the tech-spec side. Whether that prose should move depends on how gap 2
  is settled.

## Change rows — before / after

### REQ-259 (PRD-002 FR-005) — *Label a product spec with its product level*

The closed set is wrong, and the ladder's relationship to C4 is unstated.

**`description` — before**

> A product spec can record a single product level from the closed set
> domain|capability|feature|story.

**`description` — after**

> A product spec can record a single product level from the closed set
> domain|capability|feature. The ladder is deliberately shorter than the C4
> ladder; the two are comparable forms of zoom and are not depth-matched.

**`acceptance_criteria[0]` — before**

> A product spec records a single product level from the closed set
> domain|capability|feature|story.

**`acceptance_criteria[0]` — after**

> A product spec records a single product level from the closed set
> domain|capability|feature.

**`acceptance_criteria` — appended**

> An out-of-set level, including the retired `story`, is refused at parse.

Rationale for the asymmetry sentence: the four-rung ladder existed because
`story` was placed opposite C4's `code`. Removing the variant without stating
that the ladders are *not* meant to align invites a future author to restore a
fourth rung to make them line up again. RFC-024 records the argument in full.

### REQ-260 (PRD-002 FR-006) — *Decompose a product spec into a single-parent acyclic hierarchy*

The requirement states single-parent acyclicity and is silent on whether a
parent and child may share an altitude. That silence is now load-bearing:
`feature` is the ladder floor and recursion at the floor is how product
decomposition continues past it.

**`acceptance_criteria` — appended**

> A parent and child may share the same product level: `feature` is the ladder
> floor and may parent a narrower feature, adding depth without adding altitude.

No change to `description` — the requirement's subject is unchanged; this adds
a property of the decomposition it already governs.

Note the behaviour is not new. `parent_rank_findings` flags only rank inversion
(`delta < 0`) and rank gap (`delta > 1`); `delta == 0` has always passed. Until
IMP-350 that was an accident of a permissive check with tech-side test coverage
only. IMP-350 added `parent_rank_product_recursive_feature_is_clean` to pin it
product-side; this row supplies the obligation that test now proves.
