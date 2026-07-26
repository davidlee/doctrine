# REV REV-036 — Reconcile observation ledger contracts after adversarial review

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

<!-- Why this revision: what authored truth needs to change and why, the scope of
     the staged delta, and (for ADR/POL/STD/prose rows) the before/after excerpts
     the structured payload only labels. Seeded at `revision new`. -->

RV-310 found that the first active SPEC-028 contract could not uphold several
PRD-018 invariants: its kind/date-derived path did not provide global UUID
identity, component-wide correction inertness allowed invalid controls to undo
valid corrections, and role-blind dogfood plus generic measurement writers
crossed capability and trust boundaries.

The revised contract adopts the SL-231 decisions recorded in DEC-044 through
DEC-052:

- identity-only UUID-sharded authoritative paths, with optional derived
  chronological symlinks;
- shared complete-content atomic no-clobber filesystem publication;
- per-control correction resolution with idempotent repeats and deterministic
  conflict precedence;
- friction-only public CLI/MCP capture and a closed registered-source seam for
  machine measurements;
- a named adapter-known enrichment allowlist;
- bounded hostile content, safe rendering, and untrusted agent framing;
- a UUID-native, keyset-paginated read surface outside the entity-list spine;
- explicit authored-store disposition and ignore tradeoffs; and
- the authoritative ADR-001 `leaf` classification.

The product intent in PRD-018 remains unchanged. This revision tightens
SPEC-028 and REQ-405 through REQ-413 so the evergreen technical contract and
its acceptance criteria match the reviewed slice design before Plan.
