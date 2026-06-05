# rust-embed re-embed footgun — lone template edit is invisible until the embedding crate recompiles

Templates/assets under `install/` are embedded into the binary at build time by
`rust_embed` (`#[folder = "install/"]`, src/install.rs). There is **no
`build.rs` / `rerun-if-changed` for `install/`**, and rust-embed builds with
`debug-embed`. So editing a template alone and running `cargo build` prints
`Finished` but does **NOT** re-embed — the binary keeps the OLD asset.

Force the embedding crate to recompile:

    touch src/install.rs && cargo build      # or: cargo clean -p doctrine

Companion path trap: the cargo `target_directory` is redirected out of the repo
(`/home/david/.cargo/doctrine-target-jail`). `./target`, `./target-jail/`, and
`~/.cargo/bin/doctrine` are STALE copies carrying the old asset. Resolve the
fresh binary via `cargo metadata --format-version=1` (`target_directory`), never
a hardcoded `./target` path.

Proof gate (never trust `Finished`): after rebuild, render output that depends
on the asset and grep for a new marker (e.g. SL-019: `spec show` must contain
`## 1. Intent`, not the old `## Problem`). A stale embed is a SILENT corpus-wide
defect — `spec validate` checks FK integrity, not prose, so it will NOT catch it.

Source: SL-019 inquisition CHARGES I+II (both verified). Stray `target-jail/` and
a literal-path target dir were found in the working tree from botched
binary-resolution attempts — the trap recurs. Related: [[mem.pattern.testing.stale-cargo-bin-exe]].
