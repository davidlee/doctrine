# CHR-178: Adopt cargo 1.100 beta

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

`nix flake update` (2026-10-16) moved `rust-bin.beta.latest` (`flake.nix:49`)
from 1.99 to cargo/clippy 1.100.0-beta.5. `doctrine check gate` then fails
before tests run, on files no slice touched. Recurrence of `ISS-448`; root fix
is `IMP-273` (pin an exact toolchain). Found while driving `SL-281`.

## Scope

1. **Test binary path.** Cargo 1.100 places test executables under
   `target/debug/build/<pkg>/<hash>/`, so `test_support::doctrine_bin`
   (`src/test_support.rs:49`; current_exe, pop twice) resolves a path that does
   not exist and every e2e test fails with "not found". Make the resolution
   layout-independent (`CARGO_BIN_EXE_doctrine` where cargo sets it; otherwise
   walk up to the profile dir holding the binary).
2. **Clippy.** 41 errors: mostly `map_unwrap_or` (`.map(f).unwrap_or_default()`),
   some `redundant_clone`, plus unfulfilled `#[expect]`s
   (e.g. `clippy::double_must_use` at `src/map_server/state.rs:81`, cf. `IMP-273`).
3. **MSRV.** `Cargo.toml` declares `rust-version = "1.85"`. Do not adopt APIs
   newer than that (e.g. `Option::map_or_default`) without deciding on the
   MSRV explicitly.
4. Commit `flake.lock` with the fixes.

## Done when

`doctrine check gate` is green on cargo 1.100 with no symlink shims in `target/`.
