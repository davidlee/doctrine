# Doctrine spec authoring

Doctrine supports two tiers of specifications, plus requirements that descend
from them:

- **Product Requirements Doc (PRD)** — the *what*: product intent, user-facing
  requirements, acceptance criteria, out-of-scope boundaries. Lives under
  `.doctrine/spec/product/nnn/`.
- **Technical Specification (SPEC)** — the *how*: architecture, mechanism,
  constraints. Lives under `.doctrine/spec/tech/nnn/`. Each tech spec
  `descends_from` a product spec.
- **Requirements (REQ)** — individual requirements within a spec, labelled
  `FR-NNN` (functional) or `NF-NNN` (non-functional). The label is membership —
  cite the durable `REQ-NNN` id, not the mobile label.

This is the **entity model**, not the workflow. The skills `/spec-product` and
`/spec-tech` govern authoring and reviewing specs. The CLI provides the
commands; the skills provide the process.

## CLI

- `doctrine spec new product --title "..."` — scaffold a PRD.
- `doctrine spec new tech --title "..."` — scaffold a tech spec (requires
  `--descends-from <PRD-ID>`).
- `doctrine spec show <ID>` — full content.
- `doctrine spec list` — list specs.
- `doctrine spec req new --spec <SPEC-ID> --label FR --title "..."` — add a
  requirement to a spec.

See [[signpost.doctrine.requirements]] for coverage and reconciliation,
[[signpost.doctrine.revisions]] for the revision change-axis,
and [[signpost.doctrine.reference-docs]] for the glossary.
