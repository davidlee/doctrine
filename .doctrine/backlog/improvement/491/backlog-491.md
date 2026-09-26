# IMP-491: No spec governs doctor

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`doctrine doctor` (the corpus-health checks, `src/doctor_checks.rs`, finding type
`src/finding.rs`) has no governing product or tech spec: `rg -i doctor` over every
tech-spec TOML returns nothing (positive control: the same glob matches `review`).

Surfaced by IMP-481's coverage census. It bites now because RFC-032
`decision-frontier.md` D13 proposes a doctor check — every `doctrine <verb>` /
`--flag` a shipped install doc names must exist in the clap surface — and
SL-242 proposes another (reference-doc currency). New checks land with no
contract to state what doctor guarantees, its categories, or its STD-003
disclosure rules.

Shape: run `/spec-coverage-assessment` on the doctor surface. Likely a tech spec
at container or component level under SPEC-003 / SPEC-013, with no new PRD, but
the assessment decides. Not RV-specific; not part of the RFC-032 programme.
