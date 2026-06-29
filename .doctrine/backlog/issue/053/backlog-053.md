# ISS-053: Stale PATH doctrine binary silently drops new BoundaryRow fields from conformance registry

Captured from [[mem.pattern.jail.stale-binary-strips-registry-field]].

A pre-schema `~/.cargo/bin/doctrine` read-modify-writes `boundaries.toml`
through the old struct, silently dropping newly-added `BoundaryRow` fields
(e.g. `provenance`) from every row. No schema-version guard exists.

Needs investigation into a schema-version sentinel checked at boundary-start;
whether the best path is a binary-build-date check, a TOML field sentinel, or
something else is unverified.

→ RFC-005 OQ-6 / Tension 5: this item is a concrete instance of the
  "stale-binary verification hygiene" class parked there. Next revision
  of RFC-005 should incorporate this as evidence that the class is not
  merely theoretical — it has produced real registry corruption.

## Current state (2026-06-28)

The specific impact window has passed: the `provenance` field (SL-154) is
landed and baked into the PATH binary, so the one-time silent-drop event is
behind us. The *class* (stale PATH binary lacking a new schema field) persists
in the jail's readonly-PATH setup but is now:

- **Known and documented** — two memories
  ([[mem.pattern.jail.stale-binary-strips-registry-field]],
  [[mem.pattern.jail.stale-test-fixture-vocabulary-change]]) + this item
- **Mitigated by procedure** — `just rebuild-stale`, or use
  `./target/debug/doctrine` for registry-touching ops
- **Deprioritized by governance** — RFC-005 classifies stale-binary hygiene as
  "project-local, low priority," not a platform defect; parked below H1–H5
- **Tagged `mitigated`** — not closed, because a future `BoundaryRow` field
  addition could re-trigger; but the procedure + documentation make the
  residual risk acceptable at current priority

A schema-version sentinel would eliminate the class structurally, but the
cost/urgency doesn't justify it while the jail setup remains the only trigger
surface.
