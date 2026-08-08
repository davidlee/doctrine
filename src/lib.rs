// SPDX-License-Identifier: GPL-3.0-only
//! The `doctrine` library target — the entire public surface of the root package.
//!
//! It exists for exactly one consumer: `crates/doctrine-control`, the second
//! binary in this workspace (SL-248 `sec-6`, `DEC-153`). Nothing else links it —
//! `src/lib.rs` is excluded from the published crate's `include` allow-list
//! (`sec-9` `R7`), so these items never become crates.io semver surface.
//!
//! **The export list is the whole crossing** (`sec-6` invariant 1).
//! `doctrine-control` reaches the root package through the items below and
//! through nothing else, and every one of them belongs to a `leaf`-tier module
//! in `.doctrine/adr/001/layering.toml`. That is the cross-crate half of the
//! `ADR-001` gate: a cross-crate import is not a `crate::` path, so the layering
//! extractor sees no edge either way, and bounding this list is what bounds
//! every edge instead. `tests/architecture_layering.rs` asserts the set.
//!
//! **`main.rs` keeps its own module tree** (`sec-6`), so `clock`, `config_file`,
//! `git` and `kinds` compile once here and once in the binary. The two copies
//! never meet only while the binary names no `doctrine::` path — `sec-9` `R6`,
//! whose mitigation is that rule rather than a test.
//!
//! **`kinds` exports nothing and is declared anyway.** `git.rs`'s `#[cfg(test)]`
//! module reaches `crate::kinds`, and this target compiles it under `cargo test`,
//! so the export set has to be transitively closed over `crate::` paths in test
//! code as well as production code. `cargo build` stays green while this
//! declaration is missing; `E0432` surfaces only when the library's own tests
//! first compile.
//!
//! **`dtoml` is deliberately absent.** Its `DoctrineToml` projects `conduct`,
//! `verify`, `estimate`, `value`, `dispatch_config` and `install_config`, and
//! `verify` reaches engine-tier `coverage` — declaring it would pull that
//! cascade into a library whose export set is meant to be leaf-only. The two
//! items that do cross live in `config_file`, which `dtoml` re-exports.
#![expect(
    clippy::pub_use,
    reason = "a curated re-export set from private modules is what this target IS; \
              the set is bounded by the EXPORTED assertion in \
              tests/architecture_layering.rs, which is stricter than the lint"
)]
#![expect(
    dead_code,
    unused_imports,
    reason = "this target compiles a subset view of modules the binary owns: \
              everything outside the export set below is live in the bin target \
              and dead here, by construction. The modules are declared for their \
              `crate::` paths, not their contents"
)]

mod clock;
mod config_file;
mod fsutil;
mod git;
mod kinds;

/// The `[interpretation]` policy (`sec-4`, `REQ-449`) — the one export that is
/// a module rather than an item. It has its own refusal vocabulary and four
/// public types, so re-exporting each at the crate root would put `sec-4`'s
/// namespace into this one's; `EXPORTED` names the module and the boundary is
/// the same size.
pub mod interpretation;

pub use clock::today;
pub use config_file::{DOCTRINE_TOML, read_doctrine_toml_text};
pub use git::{CaptureError, read_path_at};
