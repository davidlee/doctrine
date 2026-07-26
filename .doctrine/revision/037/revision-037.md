# REV REV-037 — Clarify observation worker and query contracts

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

The second adversarial review of SL-231 found two bounded but load-bearing
omissions and seven smaller contract edges. The active observation spec must
name the existing worker-mode write guard as the enforcement seam, map every
allowlisted enrichment value to a declared facet field, define resolved-chain
and irreversible-correction behaviour, and tighten atomic-publication cleanup
and verification.

SPEC-013 also needs the non-entity observation carve-out recorded on the side
that otherwise claims every kind's list and canonical identity. The revision
does not reopen DEC-048: it keeps the versioned measurement schema and closed
source-admission boundary, while clarifying that the first instrumentation
slice owns the concrete harness adapter interface under QUE-176.
