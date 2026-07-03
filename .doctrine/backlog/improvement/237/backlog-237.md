# IMP-237: Align SPEC-003 root context with active relation and container references

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Raised from RV-236, an inquisition of SPEC-003.

Penance:

- Replace SPEC-003's live root citation to superseded ADR-004 with active
  relation canon, or explicitly frame ADR-004 as historical under ADR-010 /
  SPEC-018.
- Add canonical ids for the Dispatch & worktree and CLI surface containers in
  SPEC-003's prose container list, matching the parent tree (`SPEC-012`,
  `SPEC-013`).
- Decide whether SPEC-003's null `descends_from` is a sanctioned root-context
  exception or should point to a product spec; record the rationale in canon.

Evidence: RV-236 findings and synthesis.

## Resolution — enacted via REV-020

- **Penance 1 (F-1) — rejected as authored.** ADR-004 is *not* dead authority;
  it is live relation canon cited across the active corpus (SPEC-017, SPEC-018,
  ADR-010 §5). Its `superseded` status was a **SL-155 G5b CLI fixture** —
  ADR-012→ADR-004 authored arbitrarily to manufacture the corpus's only
  `supersedes` row. That false status was the actual defect and the cause of the
  F-1 misfire. Root cause reverted: ADR-004 → `accepted`, edge removed. SPEC-003's
  citation was correct and left untouched.
- **Penance 2 (F-2) — done.** `SPEC-012` / `SPEC-013` ids added to SPEC-003's
  container prose.
- **Penance 3 (F-3) — done.** Confirmed root-context lineage exception; sanctioned
  null `descends_from`/`parent` for `c4_level = context` specs in SPEC-017.

All three landed under REV-020 (approved, applied). Corpus validates clean.
