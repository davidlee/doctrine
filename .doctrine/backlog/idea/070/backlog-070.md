# IDE-070: Operator altitude: per-work human engagement level

How involved the human wants to be in a piece of work: how much the agent must
make sure they understand it, and how interactive the work should be to keep
their understanding current. Distinct from a **design autonomy envelope**
(what the agent may change without escalating) — altitude is about
comprehension, not authority.

Raised by the user 2026-11-08 during `SL-287`; name not settled.

## Nearest existing pieces

- `ADR-009` conduct axis — `actor` / `autonomy` per lifecycle state (advisory);
  per state, not per piece of work, and about who advances, not understanding.
- `/pair` dials — role, detail, autonomy; session-scoped.
- Project governance "Human engagement" section — calibrate explanation and
  participation; prose only.

## Related

- Design autonomy envelope: `RFC-026` `G15`/`S2`, `SL-258` `OQ-1`; `SL-287`
  `DEC-420` carries a thin realisation/intent split.
- A commitment ledger's quotes are a compact human-readable digest of a design —
  a possible input to an altitude-aware walkthrough.
