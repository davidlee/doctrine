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

// The base-resolution config + fold are now consumed by the PHASE-04 verifier and
// the PHASE-05 record handler (through `coverage_store::load_config` + `resolve`),
// so the PHASE-02 leaf-ahead-of-consumer dead_code blanket is retired.

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
    /// Override argv for `doctrine check quick` (per-edit cadence). Absent ⇒ an
    /// OWNED no-op ([`CheckPlan::Noop`]) — never a host binary (CR-F3, POL-002).
    /// Read ONLY by [`resolve_check`], never by [`resolve`] (INV-1).
    quick: Option<Vec<String>>,
    /// Override argv for `doctrine check commit` (per-commit cadence). Absent ⇒
    /// [`DEFAULT_COMMIT`]. Read ONLY by [`resolve_check`] (INV-1).
    commit: Option<Vec<String>>,
    /// Override argv for `doctrine check gate` (end-of-phase cadence). Absent ⇒
    /// [`DEFAULT_GATE`]. Read ONLY by [`resolve_check`] (INV-1).
    gate: Option<Vec<String>>,
    /// Override argv for the S1 regression suite (`doctrine check regression`).
    /// Absent ⇒ [`DEFAULT_REGRESSION`]. MUST be a per-test runner (the `cargo
    /// test` family), NOT the coarse `just gate` aggregate (SL-170 design D4) —
    /// the gate parses per-test failure keys.
    regression: Option<Vec<String>>,
}

impl VerificationConfig {
    /// The effective run timeout: the configured `timeout-secs`, else the baked
    /// [`DEFAULT_TIMEOUT_SECS`] (`300`).
    pub(crate) fn timeout_secs(&self) -> u64 {
        self.timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS)
    }

    /// The S1 regression suite argv: the configured `[verification].regression`,
    /// else the baked [`DEFAULT_REGRESSION`]. Like the other baked defaults this
    /// *informs* a host convention, never gates it (POL-002 / client-overridable).
    pub(crate) fn regression_argv(&self) -> Vec<String> {
        self.regression.clone().unwrap_or_else(|| {
            DEFAULT_REGRESSION
                .iter()
                .map(|s| (*s).to_string())
                .collect()
        })
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

// --- `doctrine check` cadence resolution (SL-163) ----------------------------
//
// A SEPARATE concern from VT-evidence [`resolve`] above: the dev-check verb reads
// the three override fields and NONE of the VT machinery (INV-1). Named defaults
// are pure data (STD-001) — they INFORM, they never gate (POL-002).

/// The baked argv for `doctrine check commit` when `[verification].commit` is
/// absent. Pure data — a host convention that *informs* (POL-002), client-overridable.
const DEFAULT_COMMIT: &[&str] = &["just", "check"];
/// The baked argv for `doctrine check gate` when `[verification].gate` is absent.
const DEFAULT_GATE: &[&str] = &["just", "gate"];
/// The baked S1 regression suite argv when `[verification].regression` is absent.
/// A per-test runner (NOT `just gate`) so the gate can parse per-test failure
/// keys (design D4). Pure data — informs, never gates (POL-002), overridable.
const DEFAULT_REGRESSION: &[&str] = &["cargo", "test", "--no-fail-fast"];
/// What the `quick` shell prints on the owned no-op path (unconfigured quick).
const QUICK_UNSET_NOTE: &str = "doctrine check quick: no [verification].quick set — skipping";

/// The three check cadences. clap-free (ADR-001 / A2) — the CLI `CheckCommand`
/// bridges to this leaf via `From` in the shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CheckKind {
    Quick,
    Commit,
    Gate,
}

impl CheckKind {
    /// The owned `[verification]` config key for this cadence — the SINGLE source
    /// of the kind's spelling (STD-001), used by the `Empty` keyed error.
    pub(crate) fn key(self) -> &'static str {
        match self {
            CheckKind::Quick => "quick",
            CheckKind::Commit => "commit",
            CheckKind::Gate => "gate",
        }
    }
}

/// What the shell should do for a cadence. The `Noop` arm keeps the unconfigured
/// `quick` path OWNED — doctrine prints + exits 0 itself, never proxies a host
/// `echo` (CR-F3, POL-002). `Empty` carries a configured-but-empty override (CR-F2)
/// so the shell errors toward the key instead of spawning nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CheckPlan {
    /// Spawn this argv (non-empty by construction — INV-2).
    Run(Vec<String>),
    /// Print this note, exit 0 — NO spawn (unconfigured quick).
    Noop(&'static str),
    /// The override is `[]` — keyed error, never an empty spawn.
    Empty(CheckKind),
}

/// Resolve a [`CheckKind`] against a [`VerificationConfig`] into a [`CheckPlan`]
/// (PURE, total over `(cfg, kind)` — INV-2). Precedence:
///   override `Some(v)` non-empty → `Run(v)`
///   override `Some([])`          → `Empty(kind)`            (CR-F2)
///   override `None`, Quick       → `Noop(QUICK_UNSET_NOTE)` (CR-F3, owned)
///   override `None`, Commit      → `Run(DEFAULT_COMMIT)`
///   override `None`, Gate        → `Run(DEFAULT_GATE)`
pub(crate) fn resolve_check(cfg: &VerificationConfig, kind: CheckKind) -> CheckPlan {
    let override_argv = match kind {
        CheckKind::Quick => &cfg.quick,
        CheckKind::Commit => &cfg.commit,
        CheckKind::Gate => &cfg.gate,
    };
    match override_argv {
        Some(argv) if argv.is_empty() => CheckPlan::Empty(kind),
        Some(argv) => CheckPlan::Run(argv.clone()),
        None => match kind {
            CheckKind::Quick => CheckPlan::Noop(QUICK_UNSET_NOTE),
            CheckKind::Commit => CheckPlan::Run(owned(DEFAULT_COMMIT)),
            CheckKind::Gate => CheckPlan::Run(owned(DEFAULT_GATE)),
        },
    }
}

/// `&[&str]` default literal → an owned argv.
fn owned(argv: &[&str]) -> Vec<String> {
    argv.iter().map(|s| (*s).to_owned()).collect()
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

    // --- SL-163 VT-2: the three check keys deserialize; INV-1 untouched --------

    #[test]
    fn check_override_keys_deserialize_on_verification_config() {
        let cfg = crate::dtoml::parse(
            "[verification]\n\
             quick  = [\"echo\", \"q\"]\n\
             commit = [\"just\", \"check\"]\n\
             gate   = [\"just\", \"gate\"]\n",
        )
        .unwrap()
        .verification;
        assert_eq!(cfg.quick, Some(vec!["echo".to_owned(), "q".to_owned()]));
        assert_eq!(
            cfg.commit,
            Some(vec!["just".to_owned(), "check".to_owned()])
        );
        assert_eq!(cfg.gate, Some(vec!["just".to_owned(), "gate".to_owned()]));
    }

    #[test]
    fn absent_table_yields_all_none_check_overrides() {
        // INV-1: an absent [verification] still defaults — the three new fields
        // are None, the existing `command` path is unperturbed.
        let cfg = crate::dtoml::parse("title = \"unrelated\"\n")
            .unwrap()
            .verification;
        assert_eq!(cfg.quick, None);
        assert_eq!(cfg.commit, None);
        assert_eq!(cfg.gate, None);
        assert_eq!(cfg.command, None);
    }

    // --- SL-163 VT-1: resolve_check truth table -------------------------------

    /// A `VerificationConfig` carrying just the three override fields under test.
    fn cfg_with(
        quick: Option<Vec<&str>>,
        commit: Option<Vec<&str>>,
        gate: Option<Vec<&str>>,
    ) -> VerificationConfig {
        let own = |o: Option<Vec<&str>>| o.map(|v| v.into_iter().map(str::to_owned).collect());
        VerificationConfig {
            quick: own(quick),
            commit: own(commit),
            gate: own(gate),
            ..Default::default()
        }
    }

    fn run(argv: &[&str]) -> CheckPlan {
        CheckPlan::Run(argv.iter().map(|s| (*s).to_owned()).collect())
    }

    #[test]
    fn resolve_check_override_present_runs_it_verbatim() {
        let cfg = cfg_with(
            Some(vec!["cargo", "test"]),
            Some(vec!["make", "ci"]),
            Some(vec!["nix", "flake", "check"]),
        );
        assert_eq!(
            resolve_check(&cfg, CheckKind::Quick),
            run(&["cargo", "test"])
        );
        assert_eq!(resolve_check(&cfg, CheckKind::Commit), run(&["make", "ci"]));
        assert_eq!(
            resolve_check(&cfg, CheckKind::Gate),
            run(&["nix", "flake", "check"])
        );
    }

    #[test]
    fn resolve_check_unconfigured_quick_is_owned_noop() {
        let cfg = cfg_with(None, None, None);
        assert_eq!(
            resolve_check(&cfg, CheckKind::Quick),
            CheckPlan::Noop(QUICK_UNSET_NOTE)
        );
    }

    #[test]
    fn resolve_check_unconfigured_commit_and_gate_use_defaults() {
        let cfg = cfg_with(None, None, None);
        assert_eq!(resolve_check(&cfg, CheckKind::Commit), run(DEFAULT_COMMIT));
        assert_eq!(resolve_check(&cfg, CheckKind::Gate), run(DEFAULT_GATE));
    }

    #[test]
    fn resolve_check_empty_override_routes_to_keyed_error_not_run() {
        // CR-F2 / EDGE: a configured `[]` is Empty(kind), never Run([]) — for
        // every cadence, including quick (whose unconfigured path is Noop).
        let cfg = cfg_with(Some(vec![]), Some(vec![]), Some(vec![]));
        assert_eq!(
            resolve_check(&cfg, CheckKind::Quick),
            CheckPlan::Empty(CheckKind::Quick)
        );
        assert_eq!(
            resolve_check(&cfg, CheckKind::Commit),
            CheckPlan::Empty(CheckKind::Commit)
        );
        assert_eq!(
            resolve_check(&cfg, CheckKind::Gate),
            CheckPlan::Empty(CheckKind::Gate)
        );
    }

    #[test]
    fn check_kind_key_is_the_config_key_spelling() {
        assert_eq!(CheckKind::Quick.key(), "quick");
        assert_eq!(CheckKind::Commit.key(), "commit");
        assert_eq!(CheckKind::Gate.key(), "gate");
    }
}
