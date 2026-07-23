# DEC-005: Harvest output — single maintained freshness-stamped section

The harvest's canonical output is one maintained `## Harvest` section in the
governing slice's `notes.md`: single-copy, freshness-stamped (date + phase +
head commit), pointer-only (ids + one clause, never restated content), swept at
each pass. Chosen over append-only per-event blocks (relocates staleness) and
no-manifest (fails "consumed, not re-derived"). Satisfies RFC-011 L2'
properties. Rationale: SL-215 `design.md` D2.
