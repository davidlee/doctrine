# REV REV-031 — Rename base orientation surface to project-orientation.md

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

The RFC-021 minimal-projection base names its orientation surface `boot-project.md`
(a "candidate" name — the RFC explicitly deferred "exact names … downstream design
work"). REV-028 committed that candidate into SPEC-009 prose. During SL-227 design
we renamed it to **`project-orientation.md`** for two reasons:

1. **Consistency.** SL-227 introduces the distinct standing-governance surface
   (FR-010, NF-005) and names it `project-governance.md`. A parallel `project-`
   prefix on the orientation surface makes the two base surfaces read as a set.
2. **Semantic precision.** The spec's own role word is "orientation surface"
   (NF-005); `boot-project.md`'s `boot-` prefix over-implies boot-snapshot
   machinery. `project-orientation.md` names the role, not the mechanism.
   (`project-onboarding.md` was considered and rejected — "onboarding" narrows
   the standing-orientation role to first-run.)

Nothing is shipped under the old name — FR-007/FR-010/NF-005 are all `pending`
(forward-intent), and no `boot-project.md` asset exists (SL-227 authors it
net-new). This revision refines unrealised forward-intent so spec and SL-227
design agree from the start, rather than shipping a deliberate known-divergence
truable only at reconcile.

## Staged delta — the string edit

Across SPEC-009, replace `boot-project.md` → `project-orientation.md`:

- **FR-007 (REQ-353)** statement — the three-file base list:
  before: `(.gitignore, doctrine.toml, boot-project.md)`
  after:  `(.gitignore, doctrine.toml, project-orientation.md)`
- **FR-010 (REQ-356)** statement — "physically distinct from boot-project.md" →
  "physically distinct from project-orientation.md".
- **NF-005 (REQ-358)** statement — "never folded into the boot-project.md
  orientation surface" → "never folded into the project-orientation.md
  orientation surface".
- **spec-009.md body** — lines ~188 and ~200, same literal replacement.

Provenance: RFC-021 (C1 minimal projection); realised by SL-227.
