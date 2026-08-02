# Review RV-320 — design of SL-233

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Pre-reading covers the complete projection-bounds sketch; `design.md` §§5.2,
5.3, 5.4, 8, and 9.3; `plan.md`'s three-design-gates rationale; `plan.toml`
PHASE-02 EN-2, EX-1..EX-8, and VT-1..VT-3; and the shipped review-ledger
protocol. The review holds the sketch to PHASE-02 EN-2's seven-part entrance
contract, especially the requirement that an answer settle rather than merely
name each question.

Lines of attack:

- Try to falsify the governing constant-size claim by tracing every serialized
  scalar and collection to a named cardinality and encoded-size bound, including
  identifiers, titles, counts, revisions, notices, derived facts, and all three
  renderings of DEC-064's one envelope.
- Recompute the byte table and distinguish an empirical average
  bytes-per-token ratio from a defensible worst-case token ceiling.
- Test every bounded field for a total selection order, an explicit drop end,
  an exact omission signal, and an account of what information the agent loses.
- Exercise the frontier rules on non-child distance-two nodes, `needs`
  neighbours, empty and fully resolved maps, leaf cursors, and pinned nodes that
  are resolved, deferred, or derived-blocked; reject clock, RNG, hash-iteration,
  or underspecified insertion-order dependence under PHASE-02 EX-5.
- Cross-examine the new per-revision change log against DEC-059 revision/CAS
  semantics, §5.3 atomic replacement, PHASE-03's declared snapshot contract,
  multi-change revisions, retention-window boundaries, and cheaper designs.
- Check the normal/full subset claim against truncation direction, especially
  the root-to-cursor active path, and check DEC-060/061 derived blocking,
  DEC-066 fingerprint-bound invalidation, DEC-064's single read model, and R3's
  orthogonal-state boundary.
- Attack the R2 mitigation with runs that are globally large but locally sparse:
  exact truncation counts help only if the projection names the relevant global
  totals and distinguishes out-of-neighbourhood state from post-selection
  truncation.
