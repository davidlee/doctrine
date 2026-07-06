# IMP-273: Pin one exact Rust toolchain across dev + CI

## Problem

Dev and CI resolve **different, independently-floating** Rust toolchains:

- Dev/jail: `flake.nix` → `rust-bin.beta.latest` (e.g. 1.97.0-beta.5).
- CI: `.github/workflows/release.yml` → `dtolnay/rust-toolchain@beta`
  (was `@stable` until `a324a514`).

The codebase leans on a `#[cfg_attr(not(test), expect(dead_code/unused, …))]`
self-clearing idiom (~19 sites). `expect` is a **hard error when unfulfilled**
(`warnings = "deny"`, `Cargo.toml:161`), and whether the `dead_code`/`unused`
lint fires is rustc-version-sensitive. So an item can read *dead* on one
toolchain (expect satisfied) and *live* on another (expect unfulfilled → build
fails). This broke every release build v0.11.0..v0.17.1 — no binaries shipped
since v0.10.0 — while dev stayed green because dev *was* the beta compiler.

`a324a514` aligned CI to `@beta`, shrinking the skew from channel-wide
(beta↔stable) to snapshot-drift (two independent "latest beta" pickups). That
unblocks releases but does not eliminate the class: two floating pickups still
drift, and beta rolls to a new number every ~6 weeks.

## Fix

Make **one exact toolchain the single source of truth**, honored by both dev
and CI — the channel (stable/beta/nightly) then becomes a free choice;
reproducibility comes from the pin, not the channel.

- Add `rust-toolchain.toml` with an exact pin (`channel = "1.97.0-beta.5"` or a
  dated nightly, per whatever features the codebase needs).
- CI: drop the `@beta` channel ref; use `dtolnay/rust-toolchain@master` (which
  reads `rust-toolchain.toml`) or an equivalent that honors the file.
- Flake: point `rust-overlay` at the same exact pin instead of `beta.latest`,
  so the jail and CI build byte-identical.

Then the `expect(not(test))` idiom stops being version-roulette across
environments, and lint calibration (clippy/gate) is genuinely shared.

## Open decision

Whether shipped **release binaries** should build on beta/nightly at all, or on
stable. Pinned nightly is a legitimate, common pattern *when pinned*; beta as a
release channel is unusual. Decide the channel when scoping — the pin mechanism
is orthogonal and correct either way.

## Provenance

Surfaced diagnosing broken releases (2026-07-06). Fix commit for the immediate
unblock: `a324a514` (`ci(SL-174): pin release toolchain to @beta`). Related
slice: SL-174 (tag-triggered prebuilt-binary release workflow, owns
`release.yml`).
