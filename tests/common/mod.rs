// SPDX-License-Identifier: GPL-3.0-only
//! Shared integration-test helpers.
//!
//! Single source: `src/test_support.rs`, `#[path]`-included here so integration tests
//! (separate compilation units from the lib unit tests) reuse the same runtime
//! `repo_root()` resolver. See CHR-014.
//!
//! SL-162: added `doctrine_bin()` runtime binary-path resolver.

#![allow(dead_code, unused_imports)] // shared helpers: not every includer uses every fn (SL-162 D5)

#[path = "../../src/test_support.rs"]
mod test_support;

pub(crate) use test_support::{doctrine_bin, repo_root, under_worker_marker};

/// Canonical config path — mirrors `src/dtoml::DOCTRINE_TOML` (which
/// integration tests can't import from a binary-only crate).
pub(crate) const DOCTRINE_TOML: &str = ".doctrine/doctrine.toml";

/// A temp base whose ancestry to `/` carries no project marker
/// (`.git`/`.jj`/`.project`/`Cargo.toml`), for tests exercising the **no-root**
/// path: `root::find` (src/root.rs) walks CWD up to `/`, so a stray marker above
/// the system tempdir (e.g. a leftover `/tmp/.git`) would resolve an incidental
/// root and mis-fire the assertion. Panics loudly if no clean base exists — a
/// missed assertion is worse than a failed test. (Tests needing a root instead
/// plant one: `create_dir(dir/".git")`.)
pub(crate) fn marker_free_base() -> std::path::PathBuf {
    let markers = [".git", ".jj", ".project", "Cargo.toml"];
    let candidates = [
        std::path::PathBuf::from("/dev/shm"),
        std::path::PathBuf::from("/var/tmp"),
        std::env::temp_dir(),
    ];
    for base in candidates {
        if base.is_dir()
            && base
                .ancestors()
                .all(|a| markers.iter().all(|m| !a.join(m).exists()))
        {
            return base;
        }
    }
    panic!("no marker-free temp base available to exercise the no-root path");
}
