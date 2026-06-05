# Authoring global orientation masters via record --global

The shipped/global orientation memory class (ADR-002, SL-018) — `repo=""` +
`anchor_kind=none`, admitted in every partition, evergreen/non-decaying — is
authored, not hand-written:

- **`doctrine memory record --global`** mints a master: it suppresses the git
  born-frame capture (`git::unanchored_frame`, identical to the unborn/non-repo
  `none_frame`) so the record carries `repo=""`/anchor `none`, and writes into the
  **repo-root `memory/`** tree (`MEMORY_MASTERS_DIR`) instead of `items/`. This is
  the declared escape hatch past the repo-anchor write gate (`memory.rs`, the
  constraint-4 bail on repo-non-empty + unanchored) — the normal record path is
  untouched; only the frame source and target dir branch on `--global`.

- **Lifecycle.** Masters under `memory/` are committed and embedded via
  `CorpusAssets` (RustEmbed). `cargo build` re-embeds; `doctrine memory sync`
  materializes them into the gitignored `.doctrine/memory/shipped/` (a second scan
  root unioned by `collect_all`), so they surface through `find`/`retrieve`/`list`
  and the boot snapshot.

- **Invariants (master-lint, `corpus::lint_master`).** Every master needs `repo=""`
  AND `anchor_kind=none`, a valid `memory_type` that is NOT the `reference` literal
  (`MemoryType::parse` bails on it — author references as `signpost`), and the scope
  floor: ≥1 of `paths`/`globs`/`commands`, never tag-only.

- **Audience = the downstream agent driving doctrine** (design D7). Doctrine-repo
  dev gotchas (rust/clippy/cargo — like this very memory) stay in `items/`, NOT the
  shipped corpus.
