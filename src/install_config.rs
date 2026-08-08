// SPDX-License-Identifier: GPL-3.0-only
//! `install_config` — the `[install]` section of `doctrine.toml` (SL-152 PHASE-06).
//!
//! Parameterises two install-time choices: the git repo slug the universal npx
//! skills delegation (`npx skills add <repo> …`) resolves against, and — since
//! SL-250 — which Claude settings file `boot install` / `memory sync install`
//! writes its direct-written hooks into (`claude_settings_scope`). A pure leaf
//! (ADR-001): serde defaults only, no IO and no domain knowledge — mirrors the
//! `dispatch_config` precedent.

use serde::Deserialize;

const DEFAULT_REPO: &str = "davidlee/doctrine";

fn default_repo() -> String {
    DEFAULT_REPO.to_string()
}

/// Which Claude settings file doctrine writes its hooks and `worktree.baseRef`
/// into (SL-250 `DEC-163`). A sticky project key rather than a per-invocation
/// flag: the routine, flagless installs (`memory sync install`) must inherit the
/// choice rather than re-elect it.
///
/// Config vocabulary only — the mapping onto a path (`settings_rel`) and onto a
/// command form (`command_form`) lives in `boot`, which owns both. Giving a pure
/// leaf that domain knowledge would invert the tiers and close a cycle, since
/// `boot` already reaches here through `dtoml` (ADR-001; SL-250 `plan.md` § *The
/// one departure from design `sec-2`*).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ClaudeSettingsScope {
    /// `.claude/settings.json` — committed, reviewable, travels with the repo.
    #[default]
    Project,
    /// `.claude/settings.local.json` — private to one checkout.
    Local,
}

/// The documented spelling of the scope key, for the operator-facing
/// announcement (STD-001). `DEC-163` drops the `--scope` flag, so the installer
/// is the only place an operator learns the choice exists — and a message that
/// misnames the key is worse than no message.
pub(crate) const CLAUDE_SETTINGS_SCOPE_KEY: &str = "claude-settings-scope";

impl ClaudeSettingsScope {
    /// The other member of the pair — the scope this one abandons, and so the
    /// one the `DEC-164` sweep targets.
    ///
    /// Pure vocabulary, which is why it sits here rather than in `boot`: it maps
    /// a scope to a scope, never to a path. The path mapping (`settings_rel`) and
    /// the command-form mapping (`command_form`) stay in `boot`, which owns both
    /// — this leaf is still out=0 (ADR-001).
    pub(crate) const fn sibling(self) -> Self {
        match self {
            Self::Project => Self::Local,
            Self::Local => Self::Project,
        }
    }
}

/// The `[install]` table from `doctrine.toml`.
///
/// The container carries `rename_all = "kebab-case"` and NOT only the enum:
/// the enum attribute governs the *values* (`project` / `local`), the container
/// one governs the *field name*. Without it `claude-settings-scope` binds
/// nothing and is silently discarded — neither this struct nor `DoctrineToml`
/// sets `deny_unknown_fields` — leaving the operator's chosen scope inert while
/// doctrine writes (and, from SL-250 PHASE-03, sweeps) the other file. The
/// precedent is `DispatchConfig`, which carries exactly this pair.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub(crate) struct InstallConfig {
    /// The git repo slug the universal npx skills delegation resolves against.
    /// Defaults to `davidlee/doctrine`.
    #[serde(default = "default_repo")]
    pub(crate) repo: String,
    /// Which Claude settings file `boot install` and `memory sync install`
    /// write. Defaults to `Project` (SL-250 `OQ-1`).
    #[serde(default)]
    pub(crate) claude_settings_scope: ClaudeSettingsScope,
}

impl Default for InstallConfig {
    fn default() -> Self {
        Self {
            repo: default_repo(),
            claude_settings_scope: ClaudeSettingsScope::default(),
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

    /// SL-250 PHASE-02 VT-1. Driven through `dtoml::parse` — the real path — so
    /// this asserts the CONTAINER `rename_all` as well as the enum's, which a
    /// constructed-enum test cannot see.
    #[test]
    fn scope_key_binds_from_its_documented_spelling() {
        let doc = crate::dtoml::parse("[install]\nclaude-settings-scope = \"local\"\n").unwrap();
        assert_eq!(
            doc.install.claude_settings_scope,
            ClaudeSettingsScope::Local
        );
        let doc = crate::dtoml::parse("[install]\nclaude-settings-scope = \"project\"\n").unwrap();
        assert_eq!(
            doc.install.claude_settings_scope,
            ClaudeSettingsScope::Project
        );
    }

    /// SL-250 PHASE-02 VT-1 — the flip `OQ-1` settled: an absent key and an
    /// absent `[install]` table both land on `Project`.
    #[test]
    fn absent_key_defaults_to_project_scope() {
        let doc = crate::dtoml::parse("[install]\nrepo = \"acme/doctrine\"\n").unwrap();
        assert_eq!(
            doc.install.claude_settings_scope,
            ClaudeSettingsScope::Project
        );
        let doc = crate::dtoml::parse("").unwrap();
        assert_eq!(
            doc.install.claude_settings_scope,
            ClaudeSettingsScope::Project
        );
    }

    /// SL-250 PHASE-02 VT-1, and the guard on this slice's worst failure mode:
    /// nothing sets `deny_unknown_fields`, so the underscore spelling is
    /// silently discarded and the field falls to `Project` — after which the
    /// PHASE-03 sweep evicts the operator's working local entries while the
    /// announcement names a target they did not choose. A `serde(alias)` would
    /// make this test pass for the wrong reason: the point is that the
    /// documented spelling binds through the container `rename_all`.
    #[test]
    fn the_underscore_spelling_does_not_bind() {
        let doc = crate::dtoml::parse("[install]\nclaude_settings_scope = \"local\"\n").unwrap();
        assert_eq!(
            doc.install.claude_settings_scope,
            ClaudeSettingsScope::Project,
            "an unknown key must not bind — if this fails, the documented \
             spelling is being served by an alias rather than rename_all"
        );
    }
}
