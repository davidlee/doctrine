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

pub(crate) use test_support::{doctrine_bin, repo_root};
