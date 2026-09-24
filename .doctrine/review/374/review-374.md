# Review RV-374 — design of SL-261

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

## Amendment — routes for severe findings (RFC-026 P10 trial)

F-1 (blocker) and F-2 (major) were disposed `fixed` without the trial's
`route:` token; both are verified, so the route is recorded here.

- **F-1 — route:owner-fix.** Two reads of `design.md` gave two accounts of one
  document (fingerprint vs sections). The duplicate read goes; the single
  `read_design_doc` in the verb is the surviving owner (design sec-3, *One read
  of the document*). Class swept: `start --from-design` already reads once;
  `materialise` fingerprints but takes content from the snapshot, not a second
  document read; the two-read adoption path is deleted.
- **F-2 — route:review.** The question was whether to accept the parser's
  whitespace-head canonicalisation as a disclosed commitment; settled in prose
  (sec-2, *Bytes outside section bodies*) and DEC-279.
