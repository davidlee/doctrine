# REV REV-013 — reconcile SL-158

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Reconcile narrative (SL-158)

- **[RV-167 F-2/F-3]:** SPEC-001 gained D13 (trinary `Gating` class) + REQ-239
  (records as admissible `needs`/`after` targets). SPEC-019 D7 + OQ-2 revised to
  reflect SL-158 landing — records now gate directly, no longer awaiting IMP-047.
  ADR-017 §3 prose corrected: the `is_work_like` target gate was widened (D2); the
  old "source-only" premise was false in the code.
- **[RV-167 F-1]:** `design.md` §3 design-target widened to include
  `src/knowledge.rs` and `src/relation_graph.rs` (test infrastructure).
