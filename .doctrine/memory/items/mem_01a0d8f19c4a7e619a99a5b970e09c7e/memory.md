Auditing the shipped corpus (`install/`, the published reference docs, and the shipped memory corpus `memory/`) for repo-private citations, an id grep returns ~164 sites in two groups. Only the second is a defect.

**Do NOT sweep — illustrations and client-structure references (the majority):**

- **Reference-form illustrations.** The `glossary.md` kind↔abbr table, `routing-process.md`'s `SL-023`/`ADR-005`/`REQ-059` list, the reference-forms headers in `install/templates/*.md`, and the commented payload examples in `templates/{spec-product,spec-tech,members,interactions}.toml`. These docs *define* what an id looks like; the correct referent is the client's own `PRD-001`.
- **Client-structure references.** `.doctrine/spec/` and `.doctrine/adr/` cited as directory conventions, and client filenames cited inside templates (`adr-nnn.md`, `phase-01.md`, `handover.md`, `research.md`).
- **Fill-in-the-blank scaffolding.** e.g. `install/harvest.md`'s `IMP-241 — <one clause>`.

The distinction is *citing the client's structure* versus *citing this repo's contents*. A careless sweep corrupts the id-vocabulary docs and every projected template.

**DO sweep — the defect:** citations that only resolve in doctrine's own repo — `ADR-005`, `SL-250`, `install/<file>.md`, `src/*.rs`. These do not dangle in a client repo; they silently resolve to the *client's* unrelated record. Full site-by-site ledger in ISS-309.
