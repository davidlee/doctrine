# CHR-052: Anchor SPEC-002 to coverage and verification implementation sources

## Source

Incidental finding from IMP-381's specification coverage census.

## Problem

SPEC-002 governs requirement coverage, executable verification, and observed
evidence, but has no `[[source]]` anchors. Its prose therefore cannot be checked
against the implementation surfaces it claims, and future coverage assessments
cannot distinguish intentionally shared seams from ungoverned code by following
the specification itself.

## Work

- Map SPEC-002 responsibilities to the live coverage and verification modules.
- Add narrow, live `[[source]]` anchors with responsibility descriptions.
- Explicitly identify shared ownership joints, especially where the future
  phase-plan surface projects VT criteria into SPEC-002's executable seam.
- Validate the resulting specification metadata.

## Boundaries

- This is an anchoring and traceability repair, not a semantic revision of
  SPEC-002.
- Do not use bulk module anchors without first mapping the responsibility each
  anchor supports.
- Do not introduce criterion lineage or a parallel executable-command schema.
- IMP-316's source-anchor liveness checker is related infrastructure, not a
  substitute for authoring SPEC-002's missing anchors.

## Result

Done 2026-08-02. Nine narrow `[[source]]` anchors added to `spec-002.toml`, all
liveness-checked: `rec.rs` (R1), `reconcile.rs` (R6), `coverage.rs` (R2/R3/R5
pure leaf), `coverage_store.rs` (R2 write path), `coverage_scan.rs` (R3/R5 reader
shell), `coverage_view.rs` (R2 derived read), `coverage_verify.rs` (R3 VT
re-derivation), `verify.rs` (R3 executable seam), `slice.rs` (R7 closure gate).

The anchor schema (`Source { language, identifier, module }`) has no description
field, so the responsibility mapping lives in a new `## Source anchors` section
in `spec-002.md`, with a joints subsection covering: `slice.rs` shared with
SPEC-014 (SPEC-002 owns only the closure-gate drift predicate); CLI verb shapes
left to SPEC-013 per D9; `verify.rs` as the projection target for a future
phase-plan surface's VT criteria (RFC-027 H6, no parallel command schema);
`git.rs` reused for staleness but not co-owned; and the SL-170 dispatch
regression gate explicitly excluded.

R4 (two-tier separation) is deliberately left unanchored — it is an invariant
proved by the absence of a coverage→authored-status edge, and an anchor would
imply a module owns it.

No semantic revision to SPEC-002. `doctrine validate` clean.
