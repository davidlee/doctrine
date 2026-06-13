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

## PHASE-02 — Detection-gap closure & entity migration (done, commit fad9d59)

`view()` hardened (comment-stripped exact header match F-C; quote-stripped keys
F-H) → the latent fallout went visible (RED) → migrated to GREEN. Slice corpus
invariant strengthened to strict whole-table-absence (F-E); `056` hardcode + the
stray-key tolerance comment removed.

**EN-2 re-scan drifted the design (load-bearing gate).** Concurrent authoring on
shared `main` had inflated the fallout past the design's "10 backlog + SL-056":
- TWO populated migrated keys (IMP-045 AND IMP-052, both `slices=["SL-056"]`), not
  one → F-G cutover rule literally fired. User ruled: hand link+strip both (2
  trivial identical edges don't warrant a migrator tool).
- 14 backlog files total carried slices/specs/drift (only `slices` ever populated).
- **SL-054 (done/terminal) — unanticipated stray-key table.** It hand-authored
  `extends=[53]` + `adrs=[1]` (NOT migrated vocabulary, not `link`-writable). User
  ruling: convert what maps to a legal label, comment out the freestyle rest →
  `adrs=[1]` became a `governed_by ADR-001` edge; `extends=[53]` (no legal
  slice→slice label) demoted to prose; typed table dropped. This is why F-E is
  STRICT (no table at all), not "no migrated key" — see design F-E.

**Durable lesson (candidate memory):** a locked design's fallout count is stale by
execution time on shared `main`; the EN-2 re-scan + F-G cutover re-evaluation is
mandatory, not ceremonial — it caught a 2nd populated key and a whole new entity
class (stray-key table) the design never saw.

Round-trip (VT-2): IMP-045 `slices: SL-056` + SL-054 `governed_by: ADR-001` render
in `inspect`; `doctrine validate` corpus clean. backlog-048/010 carry rode-along
status closures (IMP-048 done, ISS-010 obsolete) from concurrent work. `just gate`
green; behaviour-preservation (4 original storage invariants + e2e_link_unlink)
unchanged.
