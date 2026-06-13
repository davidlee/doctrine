# ISS-009: slice new scaffolder emits the stale [relationships] reserved comment, not the post-SL-048 [[relation]] row format

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`doctrine slice new` scaffolds `slice-NNN.toml` with a reserved
`[relationships]` comment block describing the **pre-SL-048** typed-table shape
(`specs = [...]`, `requirements = [...]`, `supersedes = [2]`) and the now-false
note "Empty in v1 — no spec/requirement registry to point at yet." Since SL-048
PHASE-04 ("the cut") tier-1 relations are uniform `[[relation]]` rows
(`label`/`target`), read generically by the SL-046 reader. The stale scaffold
misleads the author into prose relations (observed during SL-057 scoping).

Fix: scaffold the `[[relation]]` idiom (commented exemplar rows + the legal label
set) instead of the legacy `[relationships]` table. Check the other entity
scaffolders (spec, backlog) for the same drift. Pairs with IMP-048/IMP-049.
