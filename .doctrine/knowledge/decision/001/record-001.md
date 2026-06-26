# DEC-001: Gating direction — outbound-from-record

RFC-008 D-c: which direction do gating edges point?

## Decision

**Outbound-from-record.** An unsettled record authors `gates → target` — the gate
is an assertion of what the record *affects*. The dependent never declares "this
record gates me"; it derives `blocked_by` as the inverse of outbound gates.

## Rationale

ADR-004 (relations outbound-only, reciprocity derived) establishes the pattern
canonically. The record is the authority on what it shapes; making the dependent
declare `needs → record` would conflate two different intents:

- **`needs`**: the dependent's own declaration of a missing input — "I need X before I can start"
- **`gates`**: the record's assertion of its effect — "while I'm unsettled, X should wait"

The record knows what it affects. The dependent can derive which records gate it
by tracing the inverse of the outbound gate edges.

## Alternatives

- **Inbound from record**: dependent declares `needs → record`. Rejected because
  it places the onus on the dependent to discover and declare every unsettled
  record that might affect it, and conflates the two intents above.
- **Bidirectional**: both sides author. Rejected as a coordination hazard — two
  sources of truth for the same gate with no reconciliation mechanism.

## Consequences

- SL-158's graph layer projects `blocked_by` as the derived inverse of outbound gates
- Coordinate with IMP-033: `gates` and `needs` are structurally inverses — ensure
  the dep/seq layer treats them consistently
- The `shapes` relation stays purely semantic; the gating overlay is a separate
  projection

## References

- ADR-004 — relations outbound-only, reciprocity derived
- IMP-033 — cross-kind dep/seq capture
- QUE-001 — the D-a fork this decision informs
- RFC-008 § Outcome, D-c
