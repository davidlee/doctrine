# ISS-014: slice 067 committed without slice-067.toml breaks corpus-walk gate in clean checkouts

Slice 067 was committed (`design.md` + `slice-067.md`, commits 792ed7b/cdcd484)
**without `slice-067.toml`**. The numeric dir `.doctrine/slice/067/` therefore
carries no metadata file in the git tree.

The corpus-walk invariant tests in `tests/e2e_relation_migration_storage.rs`
(`slice_corpus_relationships_table_holds_only_dep_seq_keys` :239,
`relation_rows_of_one_label_are_contiguous` :334) iterate every numeric slice dir
via `entity_tomls` and `read_to_string(slice-NNN.toml).unwrap()` — they panic on
067's missing toml. The test reads `CARGO_MANIFEST_DIR`, not the git tree
(`mem.pattern.testing.corpus-walk-test-baked-manifest-reads-worktree-branch-corpus`),
so the break was **masked** while an untracked `slice-067.toml` sat on disk in the
authoring checkout; it surfaces RED in any clean checkout / worktree (CI, dispatch).
The untracked toml was deleted mid-session, so the break is now live on `main`.

Surfaced by the SL-066 `/dispatch` run: the funnel verify on `dispatch/066` REDs
on exactly these two tests, orthogonal to the SL-066 delta (which is green).

**Fix:** author + commit `slice-067.toml` (lifecycle metadata for SL-067), or back
out the 067 prose commits until the metadata is ready. Then the corpus gate goes
green in clean checkouts.
