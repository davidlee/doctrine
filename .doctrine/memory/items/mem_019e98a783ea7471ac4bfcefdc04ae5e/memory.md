# rust-embed assets — the footgun as recorded no longer reproduces

Assets under `install/` are embedded at build time by `rust_embed`
(`#[derive(RustEmbed)] #[folder = "install/"]`). **The embed lives in
`src/asset_source.rs:19-22`** (moved there by SL-223), *not* `src/install.rs`.

## What this memory used to say, and what is true now

It recorded that rust-embed has no `rerun-if-changed`, so a lone template edit
plus `cargo build` prints `Finished` while the binary keeps the OLD asset, and
that you must `touch src/install.rs` to force a re-embed.

**Re-probed 2026-08-08 (SL-249 PHASE-03 T5) and it does not reproduce.** At
rust-embed 8 with `debug-embed` (`Cargo.toml:155`), from a fully built tree:

    cargo build            # Finished, 0.05s — nothing to do
    <edit install/templates/knowledge-decision.toml, delete one key>
    cargo build            # Compiling doctrine ... — cargo rebuilt, no touch

and a test reading the asset through `asset_text` saw the **new** content on the
first run, with no `touch` anywhere. rust-embed's macro registers the folder's
files as build dependencies, so cargo's staleness check covers them.

Two other claims were stale as well:

- the `CARGO_TARGET_DIR` redirect to `/home/david/.cargo/doctrine-target-jail`
  was **removed in SL-156**. Each worktree builds into its own in-tree
  `./target/` — `./target/debug/doctrine` IS the live binary (AGENTS.md).
- `src/install.rs` is no longer the embed site (SL-223).

## What survives, and is the reason to keep this memory

- **`touch src/asset_source.rs && cargo build` is still the correct escape
  hatch** if you ever *do* suspect a stale asset. It is cheap and harmless; it is
  just no longer a precondition. If you reach for it, use that file — touching
  `src/install.rs` is now a no-op for the embed.
- **Never trust `Finished`; verify through the render.** A stale embed is a
  silent corpus-wide defect that FK/structural validation cannot catch. After any
  asset edit, assert on output that depends on the asset.
- **A different and still-live asset trap: the nix/crane build.**
  `craneLib.cleanCargoSource` filters the sandbox to `.rs`/`.toml`/`.lock`, so
  every non-Rust embed root is stripped and the nix binary ships hollow **with no
  compile error**. Any new embed root must be grafted into `srcWithDist` in
  `flake.nix`. Validate with `just nix-build` (AGENTS.md).

## Scope of the re-probe

`install/templates/*.toml` under the `InstallAssets` root, `cargo build`/`cargo
test --bin doctrine`, dev profile, in-tree `./target`. Not re-probed: the
`publication/`, `memory/`, `plugins/` or `web/map/dist/` roots, release profile,
or `include_str!` targets — those may still want the touch.

Source: SL-019 inquisition CHARGES I+II for the original claim; SL-249 PHASE-03
T5 for the correction. Related: [[mem.pattern.testing.stale-cargo-bin-exe]].
