# REQ-336: Retain the full claim schema on the wire

## Statement

The wire v3 anchor-claim row retains its **complete evidential schema**: the
optional provenance columns `observed_at`, `basis`, and `admission` alongside
`rater`, `by`, `date`, and the **per-domain payload column set** (value:
`{magnitude}`). Parse is lossless over absent optionals; capture-time
validation — not parse — enforces payload exactness per domain. New claim
domains (estimate, RFC-020 Phase 2) extend the row by **adding optional
columns only** — never by reshaping existing rows and never by a version
bump within v3. Every v2 document remains a valid v3 document.

## Rationale

The RFC-020 T2 invariant ("one judgement interface; domains differ only in
payload") is what keeps Phase 2 (estimate claims) and Phase 3 (hierarchy
admissibility) cheap — zero schema motion, additive columns. SL-220 proved
the shape (design D1/D2, wire suite §8.1); this requirement pins it so a
later change that drops a provenance column, reshapes rows per-domain, or
forces a version bump for a new payload is spec-visible drift, not silent
refactoring. Landed at SL-220 reconciliation (REV-025, RV-277 F-10).
