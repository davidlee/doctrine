# ISS-227: stale CLI help — needs and supersede predate SL-158/SL-097 target widening

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Two `--help` surfaces contradict the shipped behaviour:

- `doctrine needs --help`: "TGT must resolve AND be work-like (slice or
  backlog) — a free-text or non-work-like target is refused". Stale since
  SL-158 D2 widened `is_admissible_dep_target` to work-like ∪ RECORD (all
  seven kinds incl. CPT; see src/kinds.rs RECORD). Also the doc comment at
  src/commands/dep_seq.rs:73 lists only six kinds (predates CPT, SL-159).
- `doctrine supersede --help` (src/commands/cli.rs:727): "Refuses … a non-ADR
  kind". Stale since SL-095 (POL/STD arms) and SL-097 (record arms +
  validate_matrix in src/supersede.rs).

Surfaced during SL-214 PHASE-01 while grounding the /knowledge skill's
intent→verb table — the skill teaches "CLI is the source of truth", so these
actively mislead. Fix is doc-only: rewrite the two help strings (and the
dep_seq six-kind comment) against current gates.
