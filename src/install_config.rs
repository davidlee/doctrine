// SPDX-License-Identifier: GPL-3.0-only
//! `install_config` — the `[install]` section of `doctrine.toml` (SL-152 PHASE-06).
//!
//! Parameterises the printed post-install delegation instructions: the git repo
//! slug the Claude plugin marketplace (`/plugin marketplace add <repo>`) and the
//! universal npx skills command (`npx skills add <repo> …`) resolve against.
//! A pure leaf (ADR-001): serde defaults only, no IO and no domain knowledge —
//! mirrors the `dispatch_config` precedent.

use serde::Deserialize;

const DEFAULT_REPO: &str = "davidlee/doctrine";

fn default_repo() -> String {
    DEFAULT_REPO.to_string()
}

/// The `[install]` table from `doctrine.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub(crate) struct InstallConfig {
    /// The git repo slug for the plugin marketplace / npx skills commands.
    /// Defaults to `davidlee/doctrine`.
    #[serde(default = "default_repo")]
    pub(crate) repo: String,
}

impl Default for InstallConfig {
    fn default() -> Self {
        Self {
            repo: default_repo(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_repo_defaults() {
        let cfg: InstallConfig = toml::from_str("").unwrap();
        assert_eq!(cfg.repo, "davidlee/doctrine");
    }

    #[test]
    fn explicit_repo_overrides_default() {
        let cfg: InstallConfig = toml::from_str("repo = \"acme/doctrine\"").unwrap();
        assert_eq!(cfg.repo, "acme/doctrine");
    }
}
