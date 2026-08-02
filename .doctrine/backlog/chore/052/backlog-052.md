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
