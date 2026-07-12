# IDE-039: Magnitude claims as ledgered evidence

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Intent

Absolute value and estimate determinations become ledgered claims, with the
same evidential machinery relative judgements already enjoy (SL-210/213):
provenance (rater, date), supersession, multiple competing claims per item,
contradictions surfacing as findings instead of last-write-wins TOML edits.

Source: SL-217 product-critique §5/§6 tensions, surfaced during SL-219 design
(2026-07-13) —

- one hand-typed float is constitutional truth while dozens of ledgered
  judgements are "evidence" (against the subsystem's own product pitch);
- provenance is modelled but epistemic authority is binary and equal
  (agent rows close what stakeholder rows close).

## Sketch

- New `RowForm::Anchor` (single-subject absolute magnitude claim) beside
  `Order`/`Ratio` — rides the existing wire/resolve machinery; R-rules give
  supersession, cross-rater concurrency, and provenance for free.
- Claim resolution becomes an upstream **builder** of the `AnchorMap` fed to
  `compile` (competing claims → one anchor per item + findings). SL-219 D-NF
  pins `AnchorMap` as the sole anchor seam precisely so this slots in without
  reshaping the constraint layer.
- The authored facet reframes as an **operator pin** — one claim class with
  override authority, not a different ontology (critique §5 wording; SL-219's
  ADR-015 REV already adopts the pin framing for estimates).
- Trust/demotion (critique §6, T7) lands as claim-resolution policy on the
  same seam SL-213 D7 left open — not a special case.

## Consequences

Dissolves REV-022's anchors-win posture into claim resolution — governance
change (REV against ADR-015, likely RFC first). Touches capture surfaces
(`value set` / estimate edit become claim-emitting), facet-write, resolution,
and the demotion knob. Wrong altitude for a slice rider; needs its own RFC.
