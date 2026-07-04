# IMP-262: Supersede reversal verb

## Problem

`doctrine supersede` is one-way: it refuses an already-superseded entity, and
`unlink` only touches tier-1 `link` edges, not the lifecycle `superseded_by` /
`supersedes` pair. Reversing a bad supersede edge requires **hand-editing two
TOML files** — the superseder's `supersedes` field and the superseded's
`status` + `superseded_by` — with no CLI guard against partial/inconsistent
edits.

Witnessed once but high-impact (no safe undo path):
- RV-236 follow-up (REV-020): ADR-004 was falsely superseded by ADR-012
  (unrelated topic, fixture supersede from SL-155). Reverting required
  hand-editing ADR-004.toml + ADR-012.toml.

The governance tier is especially vulnerable — a bad ADR supersede can
incorrectly mark live authority as dead, and the fix path is raw TOML surgery
with no guardrails.

## Fix direction

- **`doctrine supersede --undo <OLD> <NEW>`** or a dedicated
  **`doctrine revision unsupersede <OLD>`**: reverses a supersede edge by
  restoring the superseded entity's original status, clearing both
  `superseded_by` and `supersedes` fields, and validating the reversal is
  consistent (both sides agree on the edge before reversal).
- **Or generalise `unlink`**: coverage for supersede edges so `unlink` can
  remove a supersede relationship (treating it as a typed edge rather than a
  lifecycle flag).
- Either approach should be transactional — both TOMLs updated or neither.

## Related

- RFC-011 case-notes: `[revision (REV-020 enacting RV-236 penance)]`
- ADR-004 (relation storage model)
- mem.system.governance.no-fixture-supersede-on-live-adr — the captured gotcha
  from the RV-236 false-positive investigation
