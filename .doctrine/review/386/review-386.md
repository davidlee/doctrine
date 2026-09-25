# Review RV-386 — design of SL-264

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Probed the carried-key comparison against joiners, leavers, late-node moves and
rewrites, incoming `needs` edges, and stable node identities; then traced the
new cumulative guard through declaration admission, confirmation digests,
blocking-disposition derivation and the proposed VT controls. Checked sparse
`needs` clearing against row emission and revision handling, the condition
corpus, accepted decisions, and the read-side snapshot boundary.

The incumbent mechanisms support several parts of the draft: a leaver among
carried keys remains detectable; a redeclaration preserves a node's `seq`;
`CoverageStale` and `ConfirmationStale` are separate; `needs: []` already emits
one removal row per edge, while the create-edge path is distinct from ISS-481.
All 22 current snapshot files are readable by the incumbent CLI. This read
does not establish how the proposed rule will reinterpret their receipts.
DEC-062 cannot be targeted by a Revision, and REQ-427 governs accepted record
status rather than these gate attestations.
