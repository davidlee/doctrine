# REV REV-025 — reconcile SL-220

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

<!-- Why this revision: what authored truth needs to change and why, the scope of
     the staged delta, and (for ADR/POL/STD/prose rows) the before/after excerpts
     the structured payload only labels. Seeded at `revision new`. -->

## Reconcile narrative (SL-220)

Documentary/additive follow-through deferred to reconciliation by SL-220
design §7 / D12 (the normative amendments landed earlier via REV-024, gating
the flip). Driven by RV-277 F-10.

- **C1 introduce FR-012 → SPEC-020**: full claim-schema retention — the wire
  v3 anchor-claim row retains its complete evidential schema (`observed_at` /
  `basis` / `admission`, per-domain payload columns), parse lossless over
  absent optionals, additive within v3 (new domains add columns, never
  reshape rows). Pins the RFC-020 T2 "one judgement interface" invariant as
  an agent-checkable requirement so Phase 2 (estimate claims) lands with
  zero schema motion.
- **C2 modify PRD-011**: descent prose — the value input to the priority
  view is resolved claim/projection evidence (ADR-015 T3 ladder, REV-024),
  no longer an authored `[value]` facet; pointer at the claims model.
- **C3 modify SPEC-001**: descent prose — the policy layer's value channel
  consumes the comparison pipeline's resolved `(value, provenance)`;
  documentary pointer, no contract change.
- **RFC-020 (not a REV target — `revises` vocabulary is SPEC/PRD/REQ/ADR/
  POL/STD)**: Implementation-path Phase 0 and Phase 1 rows annotated
  delivered-by-SL-220 by direct edit, recorded here; RFC stays `open` for
  Phases 2–3.

None of this may contradict the REV-024-amended normative text (checked at
landing).
