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

## Outcome

Landed in `5512d9a61`. `doctrine check gate` green on 1.100.0-beta.5
(9313 tests passed, 0 failed); the 464 symlink shims in
`target/debug/build/` were deleted first.

- **Binary path.** `doctrine_bin` walks up from `current_exe` to the first
  ancestor dir holding a `doctrine` file. That works for both the old
  (`deps/`) and new (`build/<pkg>/<hash>/`) layouts. Not
  `CARGO_BIN_EXE_doctrine`: SL-162/CHR-014 forbid baking the path, and cargo
  doesn't set it for unit tests in `src/`.
- **MSRV.** The root `[package]` never inherited `rust-version`, so clippy
  assumed the toolchain's own version as MSRV. It now inherits `1.85`. That
  silenced the 26 `map_or_default` (stable since 1.98) suggestions. It also
  exposed four real post-1.85 uses, now rewritten: `is_multiple_of` → `%`,
  and two `const fn`s (const `Vec::is_empty` / `char::is_whitespace`,
  1.87) made non-const.
- **Clippy.** Removed 13 redundant clones and one `needless_bool`.
  Deleted two unfulfilled `#[expect]`s: `collapsible_if` in `install.rs`,
  and `double_must_use` in `map_server/state.rs`, which cited `IMP-273`.
  `IMP-273` (pin an exact toolchain) is still the root fix for this kind of
  drift.
