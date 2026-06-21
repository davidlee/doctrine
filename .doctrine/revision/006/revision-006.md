# REV REV-006 — reconcile SL-136: root-level governance tags

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

<!-- Why this revision: what authored truth needs to change and why, the scope of
     the staged delta, and (for ADR/POL/STD/prose rows) the before/after excerpts
     the structured payload only labels. Seeded at `revision new`. -->

## Reconcile narrative (SL-136)

- [RV-133, D6 REV obligation]: SL-136 migrated governance/RFC tags from typed
  `[relationships].tags` to root-level `tags`. SPEC-005 D2, SPEC-016
  (responsibility text), and SPEC-018 §relations all pinned governance tags as
  typed — they are now intentionally non-canonical. This REV amends all three
  specs to reflect root-level tags, bringing the specs back into coherence with
  the implemented corpus.

### SPEC-005
- D2: "tags remain in the typed [relationships] table" → "tags moved to root-level"
- Concerns: "carries only tags now" → "carries no data fields now"  
- Responsibilities: update description of relationships seam

### SPEC-016
- Responsibility text describing the governance `[relationships]` seam as carrying
  `tags` → updated to root-level

### SPEC-018
- §relations: "tags … stays typed" → updated to root-level uniform storage
