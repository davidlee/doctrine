# IMP-135: CLI help text consistency pass

## Source

IMP-133 UX review, first pass (F-2, F-4, F-5, F-6).
See `.doctrine/backlog/improvement/133/ux-review-findings.md`.

## Scope

Four low-hanging help text improvements:

- **F-2** — `supersede` help: add directional phrasing to `<NEW> <OLD>`
  arg descriptions ("The superseding entity (newer)", "The superseded
  entity (older)")
- **F-4** — `concept-map add`/`remove`: add arg descriptions for `<ID>`,
  `<SOURCE>`, `<REL>`, `<TARGET>`, and `--force`. Currently zero.
- **F-5** — Verbosity drift: `backlog needs` has terse help vs rich
  `link`/`unlink` help. Add cycle-detection explanation, §§ refs.
- **F-6** — `after` target: `[TARGET]` bracketed as optional but required
  unless `--prune`. Clarify in arg description.

All changes are in clap `#[arg(help = "...")]` attributes — no logic
changes.
