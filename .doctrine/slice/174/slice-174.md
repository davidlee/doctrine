# Prebuilt binary distribution

## Context

`cargo install doctrine` fails on macOS with `ld: library not found for -liconv`
— a transitive crate links `iconv` at build time and the user's toolchain/SDK
lib path doesn't resolve it. Source compilation is fragile across host
environments generally; the only outcome that guarantees a clean install for
**all** macOS users regardless of their local toolchain is to stop requiring a
local compile. We ship prebuilt binaries instead.

A `build.rs`/source-link-hardening path (emit `rustc-link-search`) was
deliberately rejected: it cannot repair an arbitrarily broken local SDK and so
does not meet the "all users" bar.

The binary is not standalone source — it embeds generated/authored assets via
`rust-embed` and `include_str!`: `web/map/dist/` (built by `just web-build`),
`install/`, `plugins/`, `memory/`, and `.pi/extensions/doctrine/mcp.ts`. Any
release artifact MUST contain these. See [[crane-strips-non-rust-embeds]]:
git-based source listing drops gitignored `web/map/dist`; the existing nix flake
(crane) already grafts `dist` into the source tree and produces a correct
hermetic binary.

## Scope & Objectives

- Produce prebuilt `doctrine` binaries for macOS — `aarch64-apple-darwin` and
  `x86_64-apple-darwin` at minimum — with all embedded assets present.
- Establish a release pipeline that builds and publishes those artifacts on a
  version tag (macOS build can't run in-jail / on Linux → needs macOS CI).
- Provide at least one frictionless install channel that requires no local Rust
  toolchain.
- Update install documentation (README) to lead with the prebuilt path.

## Non-Goals

- `build.rs` / source link-flag hardening (explicitly rejected).
- Diagnosing/fixing the specific iconv-emitting crate.
- Removing `cargo install` / crates.io publish as an option — it stays for
  toolchain-equipped users; prebuilt becomes the default recommendation.
- Windows support (not in motivation; revisit later if asked).

## Affected surface (coarse, scope-relevant)

- `justfile` — release/publish recipes.
- `Cargo.toml` — possible `[package.metadata]` (e.g. binstall) and `include`.
- `.github/workflows/**` — new release CI (does not exist yet).
- `README.md` — install instructions.
- New distribution assets — installer script and/or brew formula (location TBD
  at design).

## Risks / Assumptions

- **Asset embedding on the release build** — the CI build must replicate the
  flake's dist-graft (or invoke the flake) or the shipped binary's map server is
  broken. Primary correctness risk.
- macOS CI runners are available (GitHub-hosted) and can produce both arches
  (native arm64 runner + x86_64, or cross via target).
- Codesigning/notarization: unsigned macOS binaries trigger Gatekeeper quarantine
  on download. May need an install path that strips quarantine, or accept the
  one-time user override. Decide at design.

## Open Questions

- OQ-1 Channel: GH Release + `curl|sh` installer, a Homebrew tap, `cargo-binstall`
  metadata, or a combination? (design decision)
- OQ-2 Build vehicle for release artifacts: drive via the existing nix flake on
  macOS CI, or a plain `cargo build --release` after `just web-build`? Embedding
  correctness rides on this.
- OQ-3 Target breadth: macOS-only now, or also publish Linux (and which libc) to
  make the pipeline general?
- OQ-4 Gatekeeper/notarization posture for unsigned binaries.

## Verification / closure intent

- A tagged release produces downloadable macOS arm64 + x86_64 binaries.
- A fresh macOS environment (no Rust toolchain, or the broken-iconv toolchain)
  can install and run `doctrine` via the documented channel.
- The installed binary serves the embedded map and exposes embedded
  install/plugins/memory assets (embed integrity verified, not just `--version`).

## Summary

## Follow-Ups
