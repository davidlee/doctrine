# DEC-127: Objective 6 ships on the published surface, generated

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The constraint that decides it

**A client repo has no access to this repo's spec.** Doctrine is installed into
other repositories; what reaches them is the embedded corpus, published through
`doctrine library show` or projected by `install`. This repo's `.doctrine/` tree
— its specs, ADRs, slices, knowledge records — is not distributed at all.

Objective 6's primary audience is *the agent standing at the gate*, and that
agent is running `doctrine design` in a client repo. It is always a
repo-external consumer. So a diagram in `.doctrine/spec/tech/029/` is readable
by exactly one population: people working in the doctrine repo itself. That is
not the audience.

Research thread 4 recommended the spec-sibling because it costs nothing —
no manifest entry, no embed change, no flake graft. The cost accounting was
right and the conclusion was wrong: it optimised the cheap axis and did not
weigh reachability at all.

## Why generated, not merely published

Publishing alone gets the diagram in front of the reader. It does not keep it
true.

A repo-external consumer **cannot verify freshness**. It has no view of
`src/design_run/gate.rs`, no way to diff the diagram against the table it
depicts, and no signal when the two diverge. For an in-repo reader a stale
diagram is an annoyance; for an external one it is indistinguishable from a
correct one. So freshness has to be structural rather than promised.

`SPEC-021`'s `funnel-machine.md` already solves this and is the only in-tree
precedent for documenting a machine with a diagram: a `stateDiagram-v2`
rendered from the `const` transition table in `src/funnel_machine.rs`, with a
golden test pinning the file to the code byte-for-byte and a header forbidding
hand-edits. `SPEC-029` D1 specifies the design-run gate as the same kind of
`const fn` table, so the mechanism transfers unchanged.

Publication answers *can the reader reach it*. Generation answers *is what they
reach true*. Both are required; neither substitutes.

## What inverts

Thread 4 proposed the spec holding the artefact and, optionally, a published doc
citing it. **The citation direction reverses.** The spec is the private artefact
here, so `SPEC-029` cites the published address; the published asset cites
nothing that a client cannot resolve.

Pointing a published doc at a repo-private path would be the same defect wearing
one layer of indirection — which is worth stating explicitly, because that is
the shape the corpus has already drifted into.

## The general rule, and the debt it exposes

**No shipped asset may cite a repo-private artefact.** Not a path, not an
entity id.

The id half is the one that bites hardest, and it is not merely a broken link.
Entity ids are **per-repo sequential**: a client repo mints its own `DEC-101`,
its own `SL-233`, its own `ADR-007`. A shipped asset citing `DEC-101` therefore
does not fail to resolve in a client repo — it resolves to *a different,
unrelated record*, with no error and no signal. A dangling reference announces
itself; this one does not.

`ISS-309` records the existing violations and owns the sweep. Two shapes are
already present:

- `install/design-prompts/inquiring.toml` cites `sketches/thin-adapter.md`,
  which is really `.doctrine/slice/233/sketches/thin-adapter.md` — a path with no
  correct client-repo spelling at all;
- shipped assets cite this repo's `DEC-`, `SL-`, `RV-`, `IMP-`, `ADR-` and
  `ISS-` ids throughout — `install/design-prompts/*.toml`,
  `install/dispatch-mechanics.md`, `install/review-ledger.md`, among others.

This decision does not fix that corpus; it establishes the rule the fix answers
to, and objective 6 is the first asset built under it.

## Mechanics this implies

- Source under `install/` — an **already-grafted** embed root, so `flake.nix`'s
  `srcWithDist` needs no change. (A new sibling root would.)
- A `[[entry]]` in `publication/manifest.toml`: `address` the stable logical
  contract, `backing` the physical embed key. An embedded-but-undeclared asset is
  invisible (`src/commands/library.rs:112-113`); a declared entry whose backing is
  missing is a gate failure.
- A golden test pinning the rendered diagram to `gate.rs`'s `can_advance` /
  `boundary_conditions` tables.
- A stated hand-edit-vs-generated policy in the asset's own header, following
  `funnel-machine.md`. This is the **first diagram in the shipped corpus**, so the
  policy is net-new rather than inherited.
