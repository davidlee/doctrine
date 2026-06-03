//! Project-root detection, shared by `install` and `skills`.

use std::path::PathBuf;

use anyhow::{Context, bail};

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

    for ancestor in cwd.ancestors() {
        for marker in markers {
            if ancestor.join(marker).exists() {
                return Ok(ancestor.to_path_buf());
            }
        }
    }

    bail!(
        "No project root found. Walked up from '{}' looking for any of: {:?}",
        cwd.display(),
        markers,
    )
}
