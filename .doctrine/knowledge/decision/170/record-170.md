# DEC-170: Facet writes refuse absent keys and spell empty as empty string

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Two postures, already in the tree

`src/dep_seq.rs` holds both, adjacent, and distinguishes them on one criterion:

- `apply_status` — *"F-1: the managed keys are scaffold-seeded — edit in place,
  never create. A missing key means a malformed entity; refuse rather than
  tail-insert."*
- `apply_scalar` — *"Unlike `apply_status`'s F-1 refuse, an ABSENT key is
  CREATED — these fields are legitimately absent (commented in the scaffold)."*

So the question is not which posture is better in the abstract. It is which
class the facet fields fall into.

## They are scaffold-seeded

`install/templates/knowledge-decision.toml` seeds `[facet]` with every field
present and empty, under the comment *"every field typed; seeded empty
(captured later)"*. An absent facet key is therefore not a legitimate absence —
it is damage, and creating it silently would absorb the damage instead of
reporting it.

## Why the empty string, not omission

The same fact settles the clear spelling. Omitting a key on clear would have
the writer manufacture, by its own hand, exactly the malformed state the F-1
refusal exists to detect — and the next read would see a record it must refuse.
It would also break the byte-stable round-trip the read model is pinned on.

## The near-miss worth recording

`mem_019ee9fd51d87aa38a2dfb31ad6c4eec` proves that a `toml_edit` **root**
insert-if-missing is safe: a root key cannot tail-land inside a trailing
subtable, because TOML's encoder must emit header-less root keys first. Read
quickly, that looks like a licence to drop F-1 here.

It is not. The memory scopes its proof to root keys and says so — *"A key
inserted into an existing subtable still positions within that subtable …
Don't extend this to subtable-nested writes."* `[facet]` fields are
subtable-nested.

## Implementation residue

`src/facet_write.rs::set_facet_mixed` allocates an absent table and inserts
missing keys — the opposite posture. Riding it for knowledge needs either an
F-1 guard at the call site or a posture parameter on the seam. That choice
belongs to the design; the posture it must implement is settled here.
