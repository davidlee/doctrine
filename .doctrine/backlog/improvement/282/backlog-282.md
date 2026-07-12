# IMP-282: Conformance: exclude slice-own process artifacts from undeclared leads

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Surfaced at SL-217 audit (RV-270 F-1): `slice conformance` reports the slice's
own `notes.md` and `.doctrine/rfc/011/case-notes.md` (instrumentation mandate)
as undeclared paths. These are expected process artifacts — every audit
re-dispositions the same noise. Classify (or config-exclude) slice-own
`.doctrine/slice/<id>/**` and designated instrumentation paths out of the
undeclared cell, or badge them `process` so they read as informational.
