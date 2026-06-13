# Transitive dep is in the graph but not on the extern prelude — declare direct to name it, default-features=false to keep weight zero

A dependency that appears in `cargo tree` / `Cargo.lock` only **transitively** is in
the dependency *graph* but is **not nameable** in your crate — it is not on the extern
prelude, so `use <crate>::...` fails to compile (`E0433: cannot find crate`). A
re-exporting parent only exposes what it chooses (e.g. comfy-table re-exports
`crossterm::style::{Attribute, Color}` but never `crossterm::terminal::size`).

To name it you MUST declare it directly in `Cargo.toml`. Two-step gotcha:

1. **Graph-presence ≠ nameability.** "It's already transitive via X, so no Cargo.toml
   edit is needed" is FALSE. A direct declaration is required.
2. **A direct declaration activates the dep's OWN default features**, which the
   transitive parent may have had OFF. `crossterm = "0.29"` (default features) drags in
   ~9 new compiled crates (mio, signal-hook, derive_more, …) via its `events` feature —
   real new weight. Use `crossterm = { version = "0.29", default-features = false }`
   when the path you need (`terminal::size()`) is not feature-gated. Pin the SAME
   version already in the lockfile so `cargo tree -i <crate>` shows one version and
   `Cargo.lock` gains **zero** new `[[package]]` blocks.

**Why:** SL-054 PHASE-03 design asserted "no new Cargo.toml dependency — crossterm
already transitive." Both halves were wrong: it wasn't nameable, and the naive direct
add would have pulled new crates. The achievable invariant is "no new *compiled
crate*", not "no Cargo.toml edit".

**How to apply:** when a design/plan claims a transitive crate is usable "for free,"
verify by actually `use`-ing it. If you must declare it direct, default-features=false +
version-pin to the lockfile, then prove zero new `[[package]]` blocks before committing.

See [[mem.pattern.lint.new-workspace-member-cargo-metadata]] for adjacent cargo-manifest
footguns.
