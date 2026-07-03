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
