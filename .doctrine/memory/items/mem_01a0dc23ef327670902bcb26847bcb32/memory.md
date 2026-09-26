The `[package] include` allow-list is the only reason a gitignored root
(`web/map/dist`) ships, so every compile-time embed root must be named in it.
`cargo package --list` enumerates that same allow-list: a root nobody added is
*absent by construction*, so a `--list` assertion can never fail on it — it
re-states Cargo.toml back at you.

cargo's verification step is the only oracle that sees every compile-time embed
(`include_str!` targets and RustEmbed `#[folder]` roots) the way rustc does: it
extracts the tarball into `target/package/<name>-<ver>/` and compiles it. A
dropped root fails there with the exact include-path error, ~35s warm.

SL-263 PHASE-04 added `templates/surface.ts` and did not extend the
`/templates/mcp.ts` entry; `web-build` + `gate` + `pkg-check` + `nix-build` all
passed and `cargo publish` died inside its own verification build (v0.45.0).
`just pkg-check` now runs both legs: the two `--list` assertions a build cannot
prove (`publication/manifest.toml` force-included; `src/lib.rs` absent per
SL-248 `sec-9` R7) plus `cargo package --allow-dirty`. `cargo publish` reuses
`target/package/`, so the second verification is incremental.