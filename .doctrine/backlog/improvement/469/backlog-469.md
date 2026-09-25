# IMP-469: Map growth must not invalidate cumulative map-bound attestations

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

Two cumulative conditions bind the inquiry map: `initial-concerns-recorded` and
`user-accepts-sufficiency`. Both are `attested`, both have `binding = inquiry-map`,
both have `reach = cumulative` — and cumulative reach means every edge above their
own re-derives them against **current content** (`install/design-run-stages.md`).

So any **shape** change to the map — adding a node, adding a `needs` or `parent`
edge, re-wording a question — invalidates both, and the run re-faces the human
gates at `inquiring→drafting`, `drafting→reviewing` and `reviewing→locked`.
Resolving or reopening a node does *not*: `NodeMaterial` excludes progress
(`src/design_run/inquiry.rs`, pinned by `node_material_ignores_progress_and_observes_shape`
in `src/design_run/tests.rs`). Only shape counts.

The map is supposed to be *discovered*, not pre-authored. `DEC-061` permits nodes
to be added as conditional discoveries become concrete; `RFC-030`'s vision is a
mutable graph. This gate design makes discovery expensive at the point where it is
most valuable — after a human has said "enough has been asked" and the run has
begun to commit. The cheapest strategy is to front-load the whole map, get
sufficiency accepted once, then never touch shape again. That is what the corpus
shows.

## The over-reach is narrow

`blocking-inquiries-dispositioned` is `derived`, binds the engine's dispositions,
and is cumulative too — so a genuinely new blocking question *does* block advance
until it is dispositioned. That is correct and should stay. The over-reach is
specifically the two **attested** rows.

## Principle worth settling

An attestation is a judgement about an act; a derived row is a recompute. Growth
should invalidate the derived coverage, which can see the new node, and should not
void the attested judgement that concerns were recorded and sufficiency accepted.

`initial-concerns-recorded` is the sharper case: it requires a current
`blocking-set-declared` to be named, so growth re-faces it. Any policy here must
distinguish *the named set is still a subset of the map* from *the map moved*.

## Tension with DEC-062

`DEC-062` states: "Ordinary inquiry-map maintenance—adding, moving, pinning,
deferring, or pruning nodes—does not require human approval." A shape edit today
re-opens a human gate. Those cannot both be the policy. This item is where the
contradiction gets reconciled; it is currently recorded nowhere.

## Not the same as IMP-386

`IMP-386` argues for *more* propagation, backwards: a changed answer should reopen
or flag its dependents. This argues for *less*, forwards: a new node should not
void the human's sufficiency judgement. Different axes — progress vs shape,
backward vs forward — and they must not be conflated. A naive cascade that ignores
the distinction would make this item worse, which is an argument for settling them
in one piece of design work.

## Shape of a fix

Two families, and the second looks closer to intent:

- narrow the binding of the two attested rows — a set-identity that ignores
  additions, or a reach conditional on the map's *accepted* subset;
- derive a separate "added since acceptance" row that must be dispositioned before
  the next edge — blocks advance without voiding judgement.

## Candidate co-scope

`ISS-481` (needs: null silently no-ops) is a small correctness fix in the same
surface; both were proposed for one slice.

## References

- `install/design-run-stages.md` — the edge / condition / reach table
- `DEC-061` — conditional discoveries add nodes
- `DEC-062` — maintenance does not require human approval
- `IMP-386`, `ISS-481`, `RFC-031`
