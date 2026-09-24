# REV REV-057 — SPEC-029 names design adopt as the watermark crossing

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

SL-261 replaced the `adopt_authored` payload key with the `doctrine design adopt`
verb (design `sec-2`/`sec-3`, `DEC-279`) and retired the key (`DEC-278`). SPEC-029
should name the verb as the crossing.

### SPEC-029 — command family (responsibility line)

Before:

> Front the capability through one command family — start, show, apply, resume, materialise, contract — …

After:

> Front the capability through one command family — start, show, apply, adopt, resume, materialise, contract — …

No other SPEC-029 text names `adopt_authored`.

### REQ-434 — one acceptance criterion appended

The four existing criteria stand. Appended:

> The crossing is `design adopt`: the engine derives the section map from the
> document; the caller may confirm the document fingerprint with `--expect`;
> without it the entry read is the basis. On a locked run, a document aligned
> with the watermark is a no-op and a diverged one is refused before it is
> parsed.
