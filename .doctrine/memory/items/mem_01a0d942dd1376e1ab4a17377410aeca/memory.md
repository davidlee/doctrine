Auditing or sweeping a shipped sub-corpus for repo-private citations, the obvious regex is incomplete.

`grep -rnE "\b(SL|ADR|RV|REV|PRD|SPEC|IMP|ISS|CHR|DEC|RFC|REQ|POL|STD|IDE|QUE|RSK|ASM|REC|CM)-[0-9]{3}\b"` — the ISS-309 pattern — finds the common kinds but **omits**:
- the knowledge-kind prefixes `CON`, `EVD`, `HYP`, `CPT` (`glossary.md` mints them; a `CON-006` and an `EVD-014` were live in `install/`);
- membership labels `FR-`/`NF-` (mobile, but repo-private and meaningless to a client);
- doc-local design ids `D-C8`, `D-Q3` (opaque outside the private design they come from);
- repo-private **paths** entirely — the id regex never sees them (`src/*.rs`, `install/<file>.md`, `slice-NNN`).

**Do both passes.** Ids: `\b[A-Z]{2,5}-[0-9]{2,3}\b`. Paths: `(src/[A-Za-z0-9_./-]+\.rs|install/[A-Za-z0-9_./-]+\.md|slice-[0-9]{3})`. The SL-267 `install/` sweep found five further live sites beyond the 115 the ISS-309 regex reported (review-ledger.md, design-prompts/reviewing.md, templates/review.toml, templates/rec.toml, manifest.toml).

Pair the empty result with a known-positive control (an id that IS present) so the search itself is trusted, not assumed (`DEC-314`), and classify per `mem.pattern.install.shipped-corpus-citation-illustrations` before editing — the extra hits include illustrations (`templates/plan.toml`'s `src/foo.rs`) that must be left.