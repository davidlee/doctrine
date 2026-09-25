Auditing or sweeping a shipped sub-corpus for repo-private citations, the obvious regex is incomplete.

`grep -rnE "\b(SL|ADR|RV|REV|PRD|SPEC|IMP|ISS|CHR|DEC|RFC|REQ|POL|STD|IDE|QUE|RSK|ASM|REC|CM)-[0-9]{3}\b"` — the ISS-309 pattern — finds the common kinds but **omits**:
- the knowledge-kind prefixes `CON`, `EVD`, `HYP`, `CPT` (`glossary.md` mints them; a `CON-006` and an `EVD-014` were live in `install/`);
- membership labels `FR-`/`NF-` (mobile, but repo-private and meaningless to a client);
- doc-local design ids `D-C8`, `D-Q3` (opaque outside the private design they come from);
- repo-private **paths** entirely — the id regex never sees them (`src/*.rs`, `install/<file>.md`, `slice-NNN`).

**Do both passes.** Ids: `\b[A-Z]{2,5}-[0-9]{2,3}\b`. Paths: `(src/[A-Za-z0-9_./-]+\.rs|install/[A-Za-z0-9_./-]+\.md|slice-[0-9]{3})`. The SL-267 `install/` sweep found five further live sites beyond the 115 the ISS-309 regex reported (review-ledger.md, design-prompts/reviewing.md, templates/review.toml, templates/rec.toml, manifest.toml).

Pair the empty result with a known-positive control (an id that IS present) so the search itself is trusted, not assumed (`DEC-314`), and classify per `mem.pattern.install.shipped-corpus-citation-illustrations` before editing — the extra hits include illustrations (`templates/plan.toml`'s `src/foo.rs`) that must be left.


## Two dimensions that pass is still missing (SL-267 PHASE-03)

Added after the `plugins/**` + `memory/**` sweep. Both were found *after* a first
"complete" pass had already been run and reported clean:

- **`doc/` paths.** The path regex above matches `src/*.rs` and `install/*.md`
  but not `doc/<name>.md`. Doctrine's `doc/` no longer exists in the repo, so
  those references are **dangling** — and they are typically introduced by the
  phrase *"Point of truth: `doc/entity-model.md`"*, i.e. they name a private
  document **as authoritative**. Add `doc/[A-Za-z0-9_./-]+\.md` to the path pass.
  Same for `.toml` scope fields (`paths = [...]` in a shipped `memory.toml`) —
  record those, but see note 2.
- **Doc-local design ids with no hyphen and no id shape.** `\b[A-Z]{2,5}-[0-9]{2,3}\b`
  cannot see them and neither can the ISS-309 regex: `D1`, `D18`, `F2`, `F7`,
  `S3`, `S6`, `INV-6`, `C-V`, and `§8.1`. Run
  `grep -rnE '\bF[0-9]+\b|\bC-V\b|\bINV-[0-9]+\b|\bD[0-9]+\b|\bS[0-9]\b'` as a
  third pass. In SL-267's skills this class was **~35 sites across 8 files** —
  about 7× the count PHASE-01's `install/` sweep found — so it is not a rounding
  error on the skill corpus.

**Note 1 — a doc-local id may be load-bearing as an anchor.** `worktree/SKILL.md`
used `D-NN` in *section headings* and cross-referenced them as
`[Provisioning](#provisioning-d9)`. Dropping the label silently broke the link.
After sweeping a file, re-resolve every intra-file `#anchor` it contains.

**Note 2 — a `.toml` scope field is data, not prose.** `paths`/`globs` are
matchers evaluated against the *client's* tree, so `.doctrine/slice/` etc. are
correct; a private entry (`memory/`, `src/`, `doc/*.md`) is a dead matcher rather
than a citation, and changing it changes retrieval behaviour. Record those rows
and escalate them; do not sweep them in the same pass as prose.

**Note 3 — budget for the second pass.** SL-267 PHASE-03's operand (`memory/**`)
grew from ISS-309's 11 masters to 13, and `plugins/**` from 14 files/74 sites to
16 files/~104 sites, *within* the plan's stated scope. The enumeration is always a
floor; size the phase against a live grep, not the item's count.
