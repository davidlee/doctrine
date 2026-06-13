# Notes SL-058: Finish the relation surface: fix stale scaffold templates, migrate their entity fallout, add agent guidance

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-01 — Templates & scaffold-output guard (done)

Six scaffold templates brought to post-cut shape; two complementary tests in
`tests/e2e_relation_migration_storage.rs` reusing the existing `view()` parser:
`template_source_is_post_cut_shape_kind_specific` (on-disk source guard) and
`scaffolded_entities_are_post_cut_shape_all_six_paths` (black-box golden over the
freshly-built binary, all six `new` paths). Kind-specific (F-D): slice asserts NO
`[relationships]` header; gov/backlog assert migrated keys absent + kept keys
present + `doctrine link` guidance comment present. `just gate` green.

**Carry-forward for PHASE-02/03 (template/corpus editors):** editing a template's
typed `[relationships]` axes also breaks the renderer's *unit* test beside it —
`render_{adr,policy,standard}_toml_relationships_are_preserved_and_ignored_by_meta`
in `src/{adr,policy,standard}.rs` `toml`-parse the rendered output and assert each
typed key by name (`index not found` panic when a key is dropped). The black-box
scaffold suite will NOT surface this (different suite) — run `just gate`, not just
the e2e file. Slice/backlog renderers read `tier1`/`read_block`, so dropping their
migrated keys did not break a renderer unit test (only gov did).
