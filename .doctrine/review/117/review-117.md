# Review RV-117 — design of SL-129

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

This inquisition tests `.doctrine/slice/129/design.md` against SL-129 scope,
SPEC-004, SPEC-008, ADR-001, IMP-067, IMP-131, and the live source surfaces that
would absorb the proposed `entity::Kind::stem` and `entity::id_path` change.

Lines of attack:

- The design claims behaviour preservation while touching the shared entity
  machinery. The gate must match the actual `just check` surface, not a smaller
  hand-counted subset.
- The design treats path equality as the whole observable contract. Adding data
  to a serializable `Kind` may alter CLI JSON without changing a path.
- The design gives stem-less sub-kinds an empty string and relies on assertions;
  every helper that can consume a `Kind` must carry the same guard or explicitly
  exclude those kinds.
- The design and slice scope must agree on affected files and replacement count
  before `/plan`, or planning inherits stale reconnaissance.

## Synthesis

Verdict: RV-117 is terminal, but SL-129 design is not ready for `/plan` until
the verified findings are reconciled.

- F-1 (major): the verification gate understates the shared entity-engine proof.
  The design must require the full unchanged `just check` surface before and
  after, not a seven-test count.
- F-2 (major): adding `Kind::stem` can change `doctrine catalog scan --json`
  because `Kind` is serialized through `CatalogEntity`. The design must either
  preserve that JSON shape or explicitly accept and test the drift.
- F-3 (minor): `rel_path` lacks the same stem-less sub-kind guard described for
  `id_path`; both helpers need one guarded filename computation or equivalent
  documented exclusion.
- F-4 (minor): the slice scope inventory is stale relative to the design's
  corrected replacement count and `meta.rs` exclusion.

All four findings were disposed `design-wrong` by the responder and verified by
the raiser. No charge was withdrawn.
