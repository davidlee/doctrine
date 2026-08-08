// SPDX-License-Identifier: GPL-3.0-only
//! `config_file` — the project config file's location and its raw read.
//!
//! Split out of [`crate::dtoml`] (SL-248 `sec-6`) so the two items that cross the
//! crate boundary sit in an out-edge-free leaf module. `dtoml` re-exports both
//! under their existing names, so no call site moves; `STD-001`'s single source
//! is preserved rather than duplicated.
//!
//! **Layering (ADR-001).** Leaf, out=0: `std` only. That is what makes it
//! exportable from `src/lib.rs` — `dtoml` itself cannot be, because
//! [`crate::dtoml::DoctrineToml`] projects `conduct`, `verify`, `estimate`,
//! `value`, `dispatch_config` and `install_config`, and `verify` reaches
//! engine-tier `coverage`.
//!
//! **Visibility.** Both items are authored `pub` rather than `pub(crate)`:
//! `src/lib.rs` re-exports them, and `pub use` of a `pub(crate)` item is `E0364`.

use anyhow::Context;
use std::path::Path;

/// The project config filename — lives under `.doctrine/`, the single
/// canonical home for project-local config (ISS-055).
pub const DOCTRINE_TOML: &str = ".doctrine/doctrine.toml";

/// Read the raw `doctrine.toml` body at `root` (IMPURE shell seam) — `None` when
/// the file is absent (a genuine read error still surfaces). The single file-read
/// seam shared by [`crate::dtoml::load_doctrine_toml`] and any consumer that
/// projects its own section out-of-band of [`crate::dtoml::DoctrineToml`]
/// (SL-148 `reserve`: keeps `[reservation]` parsing inside the engine-tier
/// consumer so no `leaf → engine` import is forced).
///
/// # Errors
///
/// Any read failure other than "not found" — the absent file is `Ok(None)`, so
/// an error here means the file exists and could not be read.
pub fn read_doctrine_toml_text(root: &Path) -> anyhow::Result<Option<String>> {
    let path = root.join(DOCTRINE_TOML);
    match std::fs::read_to_string(&path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("Failed to read {}", path.display())),
    }
}
