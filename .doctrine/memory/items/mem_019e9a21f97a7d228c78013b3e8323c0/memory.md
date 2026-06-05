# RustEmbed re-embeds at compile time and follows symlinks

Two `#[derive(RustEmbed)]` gotchas that bit SL-018's `CorpusAssets`
(`#[folder = "memory/"]`), and apply to any embed (`install/`, `plugins/`):

- **Compile-time embed.** The folder is snapshotted into the binary at *build*
  time. Editing a master under `memory/` (or any embedded source) does NOT change
  a previously-built binary. Any e2e that spawns the binary must `cargo build`
  first, or it tests stale embedded bytes. This compounds the stale-`CARGO_BIN_EXE`
  footgun (`mem.pattern.testing.stale-cargo-bin-exe`): the path can be fresh while
  the *contents* are stale.

- **Symlinks are followed.** The authored corpus keeps a `mem.<key>` alias symlink
  beside each `mem_<uid>` dir. RustEmbed traverses the symlink, so each master is
  emitted TWICE — once under its uid dir, once under the alias name. The
  materializer must admit only canonical uid dirs (`gather_assets` filters via
  `memory::is_uid`, mirroring `memory::scan_named`), else the alias ships a
  duplicate master. Test: `corpus::tests::gather_assets_skips_key_symlink_aliases`.
