# QUE-195: Managed design entry with an existing design document

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

A slice-id-only `doctrine design resume SL-NNN` is unambiguous while a managed
runtime snapshot survives. If no snapshot exists but an authored `design.md`
does, Doctrine must not silently treat prose as a fully reconstructed inquiry
map, gate ledger, or review history.

## Options

1. **Explicit, conservative import (recommended).** Plain `resume` reports that
   only authored design prose exists. `doctrine design start SL-NNN
   --from-design` imports headings and bodies as section drafts, records their
   source fingerprint, marks them unreviewed, and enters the drafting stage. It
   creates no inferred inquiry nodes, decisions, clearances, or review
   attestations. The agent can then inspect and extend the run normally.
2. **Refuse legacy entry in v1.** Managed design only starts where no authored
   `design.md` exists. Existing slices continue with the old skill or require a
   manually chosen new slice. This is the smallest implementation, but makes
   adoption and runtime-loss recovery materially less useful.
3. **Automatic reconstruction.** Plain `resume` parses the document and infers
   workflow state, inquiry nodes, gates, and decisions. This is convenient but
   manufactures procedural evidence that the authored prose cannot support.

The recommendation is option 1. It preserves the strong meaning of exact
resume, provides a bounded bridge for existing designs, and keeps imported
content visibly weaker than state produced by the managed protocol.

## Answer

Option 1 was accepted and is recorded by DEC-084. QUE-196 separately settles
which durable or prose question sources may seed the reconstructed inquiry map.
