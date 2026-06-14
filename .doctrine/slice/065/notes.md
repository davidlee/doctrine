# SL-065 — implementation notes

## PHASE-01 — author product intent (FR-005, FR-006 on PRD-002)

- FR-005 → REQ-259 "Label a product spec with its product level".
- FR-006 → REQ-260 "Decompose a product spec into a single-parent acyclic
  hierarchy" (mirror of REQ-083).
- `doctrine spec req add` reserves identity + label + kind + title only.
  `description` + `acceptance_criteria[]` are **hand-authored** into
  `requirement-NNN.toml` afterwards (edit-preserving). Criteria text lifted from
  design §6 verbatim.
- Pure entity authoring, no source change. `spec validate PRD-002` clean.
  REQ-082/083 (PRD-012, tech-only) untouched.
- Done in solo worktree fork `sl-065-p01`; landed `--no-ff` (e057760) onto main
  (merge 2bf8254), fork gc'd.
