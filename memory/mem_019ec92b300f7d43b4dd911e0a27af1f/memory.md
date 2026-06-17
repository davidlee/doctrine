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

Verb shapes drift — `doctrine spec --help` is the source of truth; the forms
below were re-verified 2026-06-18 (PRD-014).

- `doctrine spec new product "Title"` — scaffold a PRD. Title is **positional**
  (prompted if omitted); `--slug` optional. There is **no `--title` flag**.
- `doctrine spec new tech "Title"` — scaffold a tech spec. The new-time flags are
  only `--slug`/`--path`; there is **no `--descends-from` flag** — the descent
  edge onto a PRD is recorded after scaffold, not at `new` time. Confirm via
  `--help` before wiring.
- `doctrine spec show <PRD-ID|SPEC-ID>` — reassembled whole.
- `doctrine spec list` — list specs per subtype.
- `doctrine spec req add --kind functional|quality <SPEC_REF> "Title"` — reserve a
  `REQ-NNN` and append it as a labelled member. The label (`FR-`/`NF-`) is
  auto-assigned next-free-by-kind; `--label` overrides. (The legacy
  `req new --spec … --label FR --title …` form is **gone**.)
- **Enrich a requirement by hand:** `req add` seeds bare fields. To render richly
  under `spec show`, edit `requirement-NNN.toml` directly — add `description`
  (one-line queryable statement) and an `acceptance_criteria` array. **No flags
  exist** for these.
- `doctrine spec validate` — FK-integrity gate; expect `validate: corpus clean`.

See [[signpost.doctrine.requirements]] for coverage and reconciliation,
[[signpost.doctrine.revisions]] for the revision change-axis,
and [[signpost.doctrine.reference-docs]] for the glossary.
