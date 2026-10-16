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
    /// The doctrine version that last installed this repo (SL-282 `DEC-378`).
    /// Raw text: the currency classifier parses it so a bad value is reported
    /// rather than failing the whole config read.
    #[serde(default)]
    pub(crate) installed_version: Option<String>,
}

impl Default for InstallConfig {
    fn default() -> Self {
        Self {
            repo: default_repo(),
            claude_settings_scope: ClaudeSettingsScope::default(),
            installed_version: None,
        }
    }
}

/// The `[install]` table name and the stamp key within it (STD-001).
const INSTALL_TABLE: &str = "install";
pub(crate) const INSTALLED_VERSION_KEY: &str = "installed-version";

/// `text` with `[install] installed-version` set to `version` (PURE, text to
/// text). A single-key `toml_edit` edit: every other byte is preserved, an
/// existing value keeps its trailing comment, the table is created (appended)
/// only when absent, and the input is returned unchanged when already current.
/// The key is inserted through the `[install]` table, never at the document
/// root — a root tail-insert lands inside the last subtable.
///
/// # Errors
///
/// Fails if `text` is not valid TOML, or `install` is not a table.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "SL-282 PHASE-03 wires the install stamp write")
)]
pub(crate) fn write_installed_version(text: &str, version: &str) -> anyhow::Result<String> {
    use toml_edit::{DocumentMut, Item, Table, Value};

    let mut doc: DocumentMut = text.parse()?;
    let mut stamp = Value::from(version);
    match doc.get_mut(INSTALL_TABLE) {
        None => {
            let mut table = Table::new();
            table.insert(INSTALLED_VERSION_KEY, Item::Value(stamp));
            if !text.is_empty() {
                table.decor_mut().set_prefix("\n");
            }
            doc.insert(INSTALL_TABLE, Item::Table(table));
        }
        Some(item) => {
            let table = item.as_table_like_mut().ok_or_else(|| {
                anyhow::anyhow!("`{INSTALL_TABLE}` in doctrine.toml is not a table")
            })?;
            match table.get(INSTALLED_VERSION_KEY) {
                Some(old) if old.as_str() == Some(version) => return Ok(text.to_string()),
                Some(Item::Value(old)) => *stamp.decor_mut() = old.decor().clone(),
                Some(_) | None => {}
            }
            table.insert(INSTALLED_VERSION_KEY, Item::Value(stamp));
        }
    }
    Ok(doc.to_string())
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

    #[test]
    fn installed_version_key_binds_from_its_documented_spelling() {
        let doc = crate::dtoml::parse("[install]\ninstalled-version = \"0.47.0\"\n").unwrap();
        assert_eq!(doc.install.installed_version.as_deref(), Some("0.47.0"));
    }

    #[test]
    fn absent_installed_version_is_none() {
        let doc = crate::dtoml::parse("[install]\nrepo = \"acme/doctrine\"\n").unwrap();
        assert_eq!(doc.install.installed_version, None);
        let doc = crate::dtoml::parse("").unwrap();
        assert_eq!(doc.install.installed_version, None);
    }

    #[test]
    fn installed_version_underscore_spelling_does_not_bind() {
        let doc = crate::dtoml::parse("[install]\ninstalled_version = \"0.47.0\"\n").unwrap();
        assert_eq!(doc.install.installed_version, None);
    }

    const STAMP_LINE: &str = "installed-version = \"0.47.0\"\n";

    fn stamped(text: &str) -> String {
        write_installed_version(text, "0.47.0").unwrap()
    }

    fn reads_back(text: &str) -> Option<String> {
        crate::dtoml::parse(text).unwrap().install.installed_version
    }

    /// This repo's own file: heavily commented, no `[install]` today. Structural
    /// assertions only, so editing the file does not break the test.
    #[test]
    fn stamping_the_live_file_only_appends() {
        let original = include_str!("../.doctrine/doctrine.toml");
        let out = stamped(original);
        if original.contains("\n[install]") {
            assert_eq!(out.replace(STAMP_LINE, ""), original);
        } else {
            assert!(out.starts_with(original), "existing bytes must be a prefix");
            assert_eq!(
                out[original.len()..].trim_start_matches('\n'),
                format!("[install]\n{STAMP_LINE}")
            );
        }
        assert_eq!(reads_back(&out).as_deref(), Some("0.47.0"));
    }

    #[test]
    fn stamping_an_existing_install_table_changes_only_the_inserted_line() {
        let input = "# head\n[install]\n# which repo\nrepo = \"acme/doctrine\" # trailing\n\n# next table\n[other]\nx = 1\n";
        let out = stamped(input);
        assert_eq!(out.replace(STAMP_LINE, ""), input);
        assert!(out.contains(STAMP_LINE));
        assert_eq!(reads_back(&out).as_deref(), Some("0.47.0"));
    }

    #[test]
    fn restamping_changes_only_the_value_bytes() {
        let input = "[install]\ninstalled-version = \"0.46.0\" # pinned\nrepo = \"a/b\"\n";
        let out = stamped(input);
        assert_eq!(
            out,
            "[install]\ninstalled-version = \"0.47.0\" # pinned\nrepo = \"a/b\"\n"
        );
    }

    #[test]
    fn stamping_an_already_current_file_is_a_byte_identical_no_op() {
        let input = "[install]\n  installed-version =   \"0.47.0\"\n";
        assert_eq!(stamped(input), input);
    }

    #[test]
    fn stamping_empty_input_makes_a_valid_install_table() {
        let out = stamped("");
        assert_eq!(out, format!("[install]\n{STAMP_LINE}"));
        assert_eq!(reads_back(&out).as_deref(), Some("0.47.0"));
    }

    #[test]
    fn stamping_inline_and_dotted_install_tables_stays_valid() {
        for input in ["install = { repo = \"a/b\" }\n", "install.repo = \"a/b\"\n"] {
            let out = stamped(input);
            assert_eq!(reads_back(&out).as_deref(), Some("0.47.0"), "{input}");
            assert!(out.contains("a/b"), "{input}");
        }
    }

    #[test]
    fn stamping_a_non_table_install_refuses_rather_than_corrupts() {
        assert!(write_installed_version("install = 3\n", "0.47.0").is_err());
    }
}
