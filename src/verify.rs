// SPDX-License-Identifier: GPL-3.0-only
//! `verify` — the project verification config + pure base resolution (SL-057
//! PHASE-02, design F-1).
//!
//! A project's `doctrine.toml` `[verification]` table declares how `VT` evidence
//! is produced: a project-default base argv (`command`), a default matcher
//! `source`, a run `timeout-secs`, and named `aliases` a [`crate::coverage::VtCheck`]
//! may reference. This module owns the parsed [`VerificationConfig`] and the pure
//! [`resolve`] that folds a config + a single check into the runnable [`Resolved`]
//! (base argv ++ extra args, and the effective match source).
//!
//! **Pure leaf (ADR-001).** No clock / disk / rng / git / process here — the
//! `doctrine.toml` *read* lives in the shell (PHASE-04+), and *running* the
//! resolved argv is the verifier's job. [`resolve`] takes owned/borrowed data
//! only and is total over its inputs.

// `VerificationConfig`, `resolve`, `Resolved`, `ResolveError`, and `timeout_secs`
// are a base-resolution leaf built ahead of their consumer: PHASE-02 lands the
// config + the pure fold; the PHASE-04 verifier that reads the file and runs the
// resolved argv is the dependent consumer. Until then every item here is dead in
// the bins/lib build, so the module carries a self-clearing `not(test)` dead_code
// expect (the `coverage.rs` precedent). It scopes to `not(test)` because the VTs
// below exercise every item under `cfg(test)`, where `dead_code` would not fire;
// the gate runs plain `cargo clippy` (bins/lib, no test cfg) where the items are
// genuinely dead. The expectation is fulfilled exactly where the lint applies, and
// retires itself the moment PHASE-04 wires the verifier.
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "verify base-resolution leaf (SL-057 PHASE-02) is built ahead of \
                  its PHASE-04 verifier consumer"
    )
)]

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::coverage::{MatchSource, VtCheck};

/// The baked run timeout (seconds) when `[verification] timeout-secs` is absent.
const DEFAULT_TIMEOUT_SECS: u64 = 300;

/// The parsed `[verification]` table. Every field optional / defaulting, so an
/// ABSENT `[verification]` table yields [`VerificationConfig::default`] (tolerant
/// parse, the conduct precedent). `kebab-case` so the documented `default-source`
/// / `timeout-secs` keys parse; `[verification.aliases]` collects into `aliases`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub(crate) struct VerificationConfig {
    /// The project-default base argv a default-base check resolves to.
    command: Option<Vec<String>>,
    /// The default matcher source when a check's matcher names none.
    default_source: Option<MatchSource>,
    /// The run timeout in seconds; [`timeout_secs`](Self::timeout_secs) bakes the
    /// `300` default when absent.
    timeout_secs: Option<u64>,
    /// Named base argvs a [`VtCheck::alias`] resolves against.
    aliases: BTreeMap<String, Vec<String>>,
}

impl VerificationConfig {
    /// The effective run timeout: the configured `timeout-secs`, else the baked
    /// [`DEFAULT_TIMEOUT_SECS`] (`300`).
    pub(crate) fn timeout_secs(&self) -> u64 {
        self.timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS)
    }
}

/// A resolved runnable check: the full argv to spawn and the effective match
/// source. Produced by [`resolve`]; consumed by the PHASE-04 verifier shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Resolved {
    pub(crate) argv: Vec<String>,
    pub(crate) source: MatchSource,
}

/// Why [`resolve`] could not produce a [`Resolved`] — one variant per reason so
/// callers assert the REASON, not merely `is_err()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolveError {
    /// The check sets BOTH `alias` and `command` — mutually exclusive (the
    /// [`crate::coverage::valid`] (a) XOR, restated at the resolve seam).
    BothAliasAndCommand,
    /// The check names an `alias` the config's `[verification.aliases]` lacks.
    UnknownAlias,
    /// No base argv could be obtained: the default-base path with no
    /// `[verification] command` declared.
    NoRunnable,
}

/// Resolve a [`VtCheck`] against a [`VerificationConfig`] into a runnable
/// [`Resolved`] (PURE). Base argv precedence: an `alias` resolves through the
/// config's alias table; a literal `command` is taken verbatim; otherwise the
/// project-default `command` is used. `extra_args` always append to the base.
/// Match-source precedence: the check's own matcher source, else the config
/// `default-source`, else [`MatchSource::Stdout`].
pub(crate) fn resolve(cfg: &VerificationConfig, check: &VtCheck) -> Result<Resolved, ResolveError> {
    if check.alias.is_some() && check.command.is_some() {
        return Err(ResolveError::BothAliasAndCommand);
    }

    let mut argv = match (&check.alias, &check.command) {
        (Some(alias), _) => cfg
            .aliases
            .get(alias)
            .cloned()
            .ok_or(ResolveError::UnknownAlias)?,
        (None, Some(command)) => command.clone(),
        (None, None) => cfg.command.clone().ok_or(ResolveError::NoRunnable)?,
    };
    argv.extend(check.extra_args.iter().cloned());

    let source = check
        .matcher
        .as_ref()
        .and_then(|m| m.source.clone())
        .or_else(|| cfg.default_source.clone())
        .unwrap_or(MatchSource::Stdout);

    Ok(Resolved { argv, source })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "tests: fail-fast unwrap on parse/resolve is idiomatic"
)]
mod tests {
    use super::*;
    use crate::coverage::Matcher;

    /// Build a [`VtCheck`] from the parts a test cares about; the rest default.
    fn vtcheck(
        alias: Option<&str>,
        command: Option<Vec<&str>>,
        extra_args: Vec<&str>,
        matcher: Option<Matcher>,
    ) -> VtCheck {
        VtCheck {
            alias: alias.map(str::to_owned),
            command: command.map(|c| c.into_iter().map(str::to_owned).collect()),
            extra_args: extra_args.into_iter().map(str::to_owned).collect(),
            matcher,
        }
    }

    fn matcher(source: Option<MatchSource>, pattern: &str) -> Matcher {
        Matcher {
            source,
            pattern: pattern.to_owned(),
            regex: false,
        }
    }

    // --- VT-1: tolerant parse of [verification] ------------------------------

    #[test]
    fn full_verification_table_parses() {
        let cfg = crate::dtoml::parse(
            "[verification]\n\
             command = [\"just\", \"check\"]\n\
             default-source = \"stdout\"\n\
             timeout-secs = 120\n\
             [verification.aliases]\n\
             unit = [\"cargo\", \"test\"]\n",
        )
        .unwrap()
        .verification;
        assert_eq!(
            cfg.command,
            Some(vec!["just".to_owned(), "check".to_owned()])
        );
        assert_eq!(cfg.default_source, Some(MatchSource::Stdout));
        assert_eq!(cfg.timeout_secs(), 120);
        assert_eq!(
            cfg.aliases.get("unit"),
            Some(&vec!["cargo".to_owned(), "test".to_owned()])
        );
    }

    #[test]
    fn absent_verification_table_yields_default_and_baked_timeout() {
        // An ABSENT [verification] block parses to the default config; the baked
        // 300s timeout applies.
        let cfg = crate::dtoml::parse("title = \"unrelated\"\n")
            .unwrap()
            .verification;
        assert_eq!(cfg, VerificationConfig::default());
        assert_eq!(cfg.timeout_secs(), 300);
    }

    #[test]
    fn absent_conduct_still_yields_conduct_defaults_through_dtoml() {
        // The R2 path: dtoml carries conduct too — an absent [conduct] yields the
        // default ConductConfig through the shared reader.
        let doc = crate::dtoml::parse("[verification]\ncommand = [\"x\"]\n").unwrap();
        assert_eq!(doc.conduct, crate::conduct::ConductConfig::default());
    }

    // --- VT-2: resolve base-argv + source precedence -------------------------

    #[test]
    fn known_alias_resolves_to_its_base_argv() {
        let mut aliases = BTreeMap::new();
        aliases.insert(
            "unit".to_owned(),
            vec!["cargo".to_owned(), "test".to_owned()],
        );
        let cfg = VerificationConfig {
            aliases,
            ..Default::default()
        };
        let check = vtcheck(Some("unit"), None, vec![], Some(matcher(None, "ok")));
        let resolved = resolve(&cfg, &check).unwrap();
        assert_eq!(resolved.argv, vec!["cargo".to_owned(), "test".to_owned()]);
        assert_eq!(resolved.source, MatchSource::Stdout);
    }

    #[test]
    fn unknown_alias_errors() {
        let cfg = VerificationConfig::default();
        let check = vtcheck(Some("missing"), None, vec![], Some(matcher(None, "ok")));
        assert_eq!(resolve(&cfg, &check), Err(ResolveError::UnknownAlias));
    }

    #[test]
    fn both_alias_and_command_errors() {
        let cfg = VerificationConfig::default();
        let check = vtcheck(
            Some("unit"),
            Some(vec!["cargo", "test"]),
            vec![],
            Some(matcher(None, "ok")),
        );
        assert_eq!(
            resolve(&cfg, &check),
            Err(ResolveError::BothAliasAndCommand)
        );
    }

    #[test]
    fn default_base_uses_config_command() {
        let cfg = VerificationConfig {
            command: Some(vec!["just".to_owned(), "check".to_owned()]),
            ..Default::default()
        };
        // Neither alias nor command on the check ⇒ the project-default base.
        let check = vtcheck(None, None, vec![], Some(matcher(None, "ok")));
        let resolved = resolve(&cfg, &check).unwrap();
        assert_eq!(resolved.argv, vec!["just".to_owned(), "check".to_owned()]);
    }

    #[test]
    fn default_base_with_no_config_command_errors() {
        let cfg = VerificationConfig::default();
        let check = vtcheck(None, None, vec![], Some(matcher(None, "ok")));
        assert_eq!(resolve(&cfg, &check), Err(ResolveError::NoRunnable));
    }

    #[test]
    fn literal_command_is_taken_verbatim_and_extra_args_append() {
        let cfg = VerificationConfig::default();
        let check = vtcheck(
            None,
            Some(vec!["cargo", "test"]),
            vec!["--quiet", "--", "mymod"],
            None,
        );
        let resolved = resolve(&cfg, &check).unwrap();
        assert_eq!(
            resolved.argv,
            vec![
                "cargo".to_owned(),
                "test".to_owned(),
                "--quiet".to_owned(),
                "--".to_owned(),
                "mymod".to_owned(),
            ],
            "argv == base ++ extra_args"
        );
    }

    #[test]
    fn extra_args_append_to_alias_base() {
        let mut aliases = BTreeMap::new();
        aliases.insert(
            "unit".to_owned(),
            vec!["cargo".to_owned(), "test".to_owned()],
        );
        let cfg = VerificationConfig {
            aliases,
            ..Default::default()
        };
        let check = vtcheck(
            Some("unit"),
            None,
            vec!["--release"],
            Some(matcher(None, "ok")),
        );
        let resolved = resolve(&cfg, &check).unwrap();
        assert_eq!(
            resolved.argv,
            vec![
                "cargo".to_owned(),
                "test".to_owned(),
                "--release".to_owned()
            ]
        );
    }

    #[test]
    fn source_precedence_entry_matcher_wins() {
        // Entry matcher source beats config default-source.
        let cfg = VerificationConfig {
            command: Some(vec!["x".to_owned()]),
            default_source: Some(MatchSource::Stderr),
            ..Default::default()
        };
        let check = vtcheck(
            None,
            None,
            vec![],
            Some(matcher(Some(MatchSource::Stdout), "ok")),
        );
        assert_eq!(resolve(&cfg, &check).unwrap().source, MatchSource::Stdout);
    }

    #[test]
    fn source_precedence_falls_to_default_source() {
        // No matcher source ⇒ config default-source.
        let cfg = VerificationConfig {
            command: Some(vec!["x".to_owned()]),
            default_source: Some(MatchSource::Stderr),
            ..Default::default()
        };
        let check = vtcheck(None, None, vec![], Some(matcher(None, "ok")));
        assert_eq!(resolve(&cfg, &check).unwrap().source, MatchSource::Stderr);
    }

    #[test]
    fn source_precedence_falls_to_stdout() {
        // Neither matcher source nor default-source ⇒ Stdout.
        let cfg = VerificationConfig {
            command: Some(vec!["x".to_owned()]),
            ..Default::default()
        };
        let check = vtcheck(None, None, vec![], None);
        assert_eq!(resolve(&cfg, &check).unwrap().source, MatchSource::Stdout);
    }
}
