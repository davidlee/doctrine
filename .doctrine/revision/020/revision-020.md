# REV REV-020 — RV-236 relation-canon corrections

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

Enacts the surviving penance of RV-236 (inquisition of SPEC-003) plus the root
cause it exposed. RV-236 raised three findings; F-1 was rejected on ground truth
(see below) and replaced by the real defect it mistook — a spurious supersede
edge. Origin: IMP-237.

### ADR-012 / ADR-004 — revert the fixture supersede (RV-236 F-1 root cause)

RV-236 F-1 claimed SPEC-003 cites *superseded* ADR-004 as live relation
authority. Ground truth: ADR-004 (outbound-only relations) is the live relation
principle, cited as authority across the active corpus (SPEC-017, SPEC-018,
ADR-010 §5 carve-out). Its `superseded_by = ["ADR-012"]` was authored by SL-155
G5b **purely as a CLI fixture** — the slice doc states no governance entity had a
`supersedes` row and picked ADR-012→ADR-004 arbitrarily to manufacture one.
ADR-012 (dispatch integration topology) supersedes nothing about relations; its
body never mentions ADR-004. The fake status is what tricked the inquisitor.

A governance corpus is not a CLI test fixture. Reverting restores ADR-004 to live
canon and clears the false-superseded taint corpus-wide (~40 citations become
lawful without touching any of them). No replacement fixture — a `supersedes` row
must reflect a real supersession or not exist.

- `adr-004.toml`: `status = "superseded"` → `"accepted"`; `superseded_by =
  ["ADR-012"]` → `[]`.
- `adr-012.toml`: `supersedes = ["ADR-004"]` → `[]`.

### SPEC-003 — container ids (RV-236 F-2)

Two child containers were named in prose without their durable ids while already
parenting to SPEC-003 structurally. Added:

- `- **Dispatch & worktree**` → `**Dispatch & worktree** (SPEC-012)`.
- `- **CLI surface**` → `**CLI surface** (SPEC-013)`.

### SPEC-017 — sanction the root-context lineage exception (RV-236 F-3)

SPEC-003 is `active` with `descends_from = null` and `parent = null`. F-3 held
that silence must not masquerade as doctrine. Confirmed a lawful root-context
exception (not a mis-set field): a `c4_level = "context"` tech spec is the
whole-system synthesis, above the product capabilities, so it descends from no
single PRD and has no decomposition parent. Recorded the exception in SPEC-017's
"Outbound descent and decomposition" section, scoped to the context altitude —
container/component specs with null lineage remain unplaced, not roots.
