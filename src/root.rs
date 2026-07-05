// SPDX-License-Identifier: GPL-3.0-only
//! Project-root detection, shared by `install` and `skills`.

use std::path::{Path, PathBuf};

use anyhow::Context;

/// Default markers that identify a project root when walking up from CWD.
pub(crate) fn default_markers() -> Vec<String> {
    vec![
        ".git".to_string(),
        ".jj".to_string(),
        ".project".to_string(),
        "Cargo.toml".to_string(),
    ]
}

/// Resolve the project root.
///
/// An `explicit` path is used as-is. Otherwise walk up from CWD until a
/// directory contains any of `markers`.
pub(crate) fn find(explicit: Option<PathBuf>, markers: &[String]) -> anyhow::Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path);
    }

    let cwd = std::env::current_dir().context("Failed to get current working directory")?;

    find_from(&cwd, markers).ok_or_else(|| {
        anyhow::anyhow!(
            "No project root found. Walked up from '{}' looking for any of: {:?}",
            cwd.display(),
            markers,
        )
    })
}

/// Walk up from `start` (inclusive) to the first ancestor containing any of
/// `markers`. Unlike [`find`], this takes an explicit start directory — the
/// ambient-surfacing hook (SL-205) discovers the root from the stdin `cwd`, not
/// the process cwd — and returns `None` (rather than an error) when no marker is
/// found, so the caller can fold a missing root into a fail-open no-op.
pub(crate) fn find_from(start: &Path, markers: &[String]) -> Option<PathBuf> {
    for ancestor in start.ancestors() {
        for marker in markers {
            if ancestor.join(marker).exists() {
                return Some(ancestor.to_path_buf());
            }
        }
    }
    None
}
