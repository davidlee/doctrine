# CHR-041: Comparison surface polish (VH-1)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Cosmetic polish candidates from SL-213's audit (RV-266 F-7/F-8), harvested
from worker notes plus an agent VH-1 pre-screen on a synthesized mini-ledger.
None functional; human eyeball decides which to take.

- `compare list`: a tombstoned row renders both `status=tombstoned` and the
  legacy `[withdrawn]` marker on one line — drop one.
- explain gauge line says "no anchor in component" for both the P7 case
  (anchors exist, none bracket this entity) and the P8 case (no anchors at
  all) — distinguishable if desired.
- pluralization: "1 prefer-first judgements recorded" (count=1 renders the
  plural); same template family as the gauge line's "ordered by N judgements".
- priority-domain disclosure line is a global corpus count rendered on every
  entity's explain — per-entity scoping would be quieter (design D2 does not
  pin the scope).
- AnchorConflict finding phrasing "anchors X=1.0 vs Y=2.0 conflict" — terse;
  acceptable, revisit only if users stumble.
