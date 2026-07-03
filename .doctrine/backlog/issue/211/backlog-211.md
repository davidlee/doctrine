# ISS-211: CM contextualizes edges writable but read-dropped

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Symptom

The `Contextualizes` relation rule (`relation.rs:498`) is `link:Writable`,
`tier:One`, `sources:&[CM]` — so `doctrine link CM-x contextualizes SL-y` passes
`validate_link` and `append_edge` writes a `[[relation]]` row onto the CM entity.
But the read side drops it: `CM` is absent from the `relation_edges` dispatch
(`catalog/scan.rs:52-53`, "CM authors no outbound relations"), so that authored row
is never scanned, hydrated, rendered, or indexed. A user authors an edge that
silently vanishes.

## Impact

Latent, not user-reported. Surfaced by the SL-196 external inquisition (finding
F-A) while validating the descriptor scope boundary. Consequence for SL-196: blocks
a future `contextualizes` descriptor home (a descriptor there would be equally
invisible) — see SL-196 follow-ups.

## Resolution options

1. Close the write seam — mark `Contextualizes` `TypedVerbOnly` (or hard-refuse in
   `link`), so CM edges are authored only through their real path.
2. Open the read seam — add `CM` to `relation_edges` so authored contextualizes
   rows are scanned/hydrated (unblocks the descriptor follow-up).

Pick per the concept-map ownership model — needs a design conversation, not a
drive-by fix.

## Provenance

Discovered: SL-196 design inquisition, 2026-07-04.
