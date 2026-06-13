# IMP-044: Migrate priority render surfaces onto RenderOpts

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Origin: SL-054 code-review finding #5 (deferred, not a defect).

SL-054 bundled the list spine's render axes into `RenderOpts { color, term_width }`
(D1) so future axes are a struct field, not a new positional through ~10 call
sites. But `priority::render::{survey_human, next_human}` and the lower primitive
`listing::render_table` take a **bare** `term_width: Option<u16>` — the design's
deliberate carve-out, since priority surfaces stay monochrome (colour deferred) and
`render_table` has never carried colour.

Consequence: priority surfaces wrap but don't colour, and the "bundle once"
principle is only half-applied. When colour reaches priority, those sites re-churn
(a second positional, or migration to `RenderOpts` then). This item tracks doing
that migration so the render seam is uniform.

Not urgent: current behaviour is correct and the carve-out is design-sanctioned
(SL-054 §7 D1). Pick up if/when priority gains colour, or as a standalone
seam-uniformity cleanup.
