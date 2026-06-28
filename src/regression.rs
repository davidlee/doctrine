// SPDX-License-Identifier: GPL-3.0-only
//! `regression` — the pure failure-set diff core (ADR-001 leaf, SL-170 S1 /
//! PHASE-02). Given a BASELINE failure-set (the suite at the funnel's pre-spawn
//! HEAD `B`) and a CURRENT failure-set (the suite at the integrated `S`), it
//! partitions test failures into four buckets keyed by a target-qualified test
//! key plus a volatility-stripped signature:
//!
//! - `new` — key in current \ baseline = a REGRESSION (halts the gate);
//! - `changed` — key in both, signature differs = a NEW failure mode for an
//!   already-red test (halts — codex F-1 / INV-7);
//! - `fixed` — key in baseline \ current = an improvement (informational);
//! - `persistent` — key in both, SAME signature = a genuinely pre-existing /
//!   environmental failure that fails identically (ignored).
//!
//! The gate halts on `new ∪ changed`. `persistent` absorbs the env artifacts a
//! correctly-normalised coordination tree shares between `B` and `S` (the OQ-4
//! disambiguator), but ONLY under INV-1 (identical invocation + filter state).
//!
//! A suite run yields EITHER a well-formed [`FailureSet::Obtained`] OR a
//! [`FailureSet::Unobtainable`] marker — NEVER a silent empty set (INV-5). A
//! non-completing / unparseable run is `Unobtainable`, and [`diff`] returns
//! `Err` if either side is `Unobtainable`, so a compile error / panic / format
//! change at `S` can never read as "zero failures = green". This is the
//! load-bearing inversion against the SL-169 ship-as-env regression.
//!
//! Pure: std + `regex` + `anyhow` only. Suite invocation, file/cache IO, the sha
//! key, and the run-fingerprint all live in the impure shell
//! (`crate::commands::check`); this module receives captured suite output as
//! `&str` and emits a [`RegressionDelta`] / a rendered `String`.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Result, bail};
use regex::Regex;

/// A suite run's failure-set: EITHER a well-formed map (`key → sig`) OR an
/// unobtainable marker carrying why (INV-5). Never silently `Obtained(∅)` for a
/// run that did not complete — that is the false-green hole this kind closes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FailureSet {
    /// A parsed failure-set: test key → volatility-stripped signature.
    Obtained(BTreeMap<String, String>),
    /// The run did not complete or its output did not parse; `why` explains.
    Unobtainable { why: String },
}

/// The four-bucket partition of a baseline→current failure-set diff. `new` and
/// `changed` halt the gate (INV-7); `fixed` and `persistent` are informational.
/// Keys only — the signatures live in the input sets.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RegressionDelta {
    /// Keys in current \ baseline — regressions. Halts.
    pub(crate) new: BTreeSet<String>,
    /// Keys in both, signature differs — new failure mode. Halts (F-1).
    pub(crate) changed: BTreeSet<String>,
    /// Keys in baseline \ current — improvements. Informational.
    pub(crate) fixed: BTreeSet<String>,
    /// Keys in both, SAME signature — genuinely pre-existing. Ignored.
    pub(crate) persistent: BTreeSet<String>,
}

impl RegressionDelta {
    /// The gate's halt set: `new ∪ changed` (INV-7 — NOT `new` alone). Non-empty
    /// ⇒ the diff step must exit non-zero.
    pub(crate) fn halting(&self) -> BTreeSet<String> {
        self.new.union(&self.changed).cloned().collect()
    }
}

/// Diff a baseline against a current failure-set into the four buckets. `Ok` only
/// when BOTH sides are `Obtained`; an `Unobtainable` side is a hard `Err` (INV-5)
/// — never a silent ∅-pass. The dangerous direction is a false-green at `S`.
pub(crate) fn diff(baseline: &FailureSet, current: &FailureSet) -> Result<RegressionDelta> {
    let (base, cur) = match (baseline, current) {
        (FailureSet::Obtained(b), FailureSet::Obtained(c)) => (b, c),
        (FailureSet::Unobtainable { why }, _) | (_, FailureSet::Unobtainable { why }) => {
            bail!(
                "regression diff: a failure-set is unobtainable ({why}) — refusing a silent ∅-pass (INV-5)"
            )
        }
    };
    let mut delta = RegressionDelta::default();
    for (key, sig) in cur {
        match base.get(key) {
            None => {
                delta.new.insert(key.clone());
            }
            Some(base_sig) if base_sig == sig => {
                delta.persistent.insert(key.clone());
            }
            Some(_) => {
                delta.changed.insert(key.clone());
            }
        }
    }
    for key in base.keys() {
        if !cur.contains_key(key) {
            delta.fixed.insert(key.clone());
        }
    }
    Ok(delta)
}

/// Parse captured `cargo test` output into a [`FailureSet`]. Section-aware: each
/// `Running … (target/debug/deps/<bin>-<hash>)` line opens a section whose
/// `<bin>` (hash stripped) target-qualifies every key within, so same-named
/// tests across binaries get distinct keys (A2). The signature is the
/// volatility-stripped first meaningful line of each test's `---- <name> stdout
/// ----` panic block. Output with NO recognisable cargo test structure →
/// `Unobtainable` (INV-5), never `Obtained(∅)`.
pub(crate) fn parse_failures(suite_output: &str) -> FailureSet {
    // INV-5: output with no recognisable cargo test structure (a compile error,
    // a process killed before any test ran) is Unobtainable — NEVER Obtained(∅).
    let structured = suite_output.contains("test result:")
        || suite_output.contains("\nrunning ")
        || suite_output.starts_with("running ");
    if !structured {
        return FailureSet::Unobtainable {
            why: "suite output carries no `cargo test` structure (run did not complete?)".into(),
        };
    }
    let mut map = BTreeMap::new();
    let mut target = String::from("unknown");
    let mut lines = suite_output.lines().peekable();
    while let Some(line) = lines.next() {
        if let Some(t) = running_target(line) {
            target = t;
            continue;
        }
        let Some(name) = stdout_marker(line) else {
            continue;
        };
        // Collect the panic block until the next marker / section / summary.
        let mut body: Vec<&str> = Vec::new();
        while let Some(&peek) = lines.peek() {
            if stdout_marker(peek).is_some()
                || running_target(peek).is_some()
                || peek.trim_start().starts_with("failures:")
                || peek.trim_start().starts_with("test result:")
            {
                break;
            }
            lines.next();
            let t = peek.trim();
            if !t.is_empty() && !t.starts_with("note:") {
                body.push(t);
            }
        }
        map.insert(format!("{target}::{name}"), normalise_sig(&body.join(" ")));
    }
    FailureSet::Obtained(map)
}

/// Parse a target binary name from a cargo `Running … (target/debug/deps/<bin>-<hash>)`
/// line, stripping the trailing build hash so the key is stable across rebuilds.
fn running_target(line: &str) -> Option<String> {
    let rest = line.trim_start().strip_prefix("Running ")?;
    let open = rest.rfind('(')?;
    let inside = rest[open + 1..].trim_end().trim_end_matches(')');
    let base = inside.rsplit('/').next()?; // "doctrine-aaa111"
    // strip the trailing `-<hash>` only (crate names may contain `-`).
    let target = base.rsplit_once('-').map_or(base, |(name, _hash)| name);
    Some(target.to_string())
}

/// Parse a test name from a cargo `---- <name> stdout ----` failure-block marker.
fn stdout_marker(line: &str) -> Option<String> {
    let rest = line.trim().strip_prefix("---- ")?;
    rest.strip_suffix(" stdout ----").map(str::to_string)
}

/// Render a [`RegressionDelta`] for the diff step, labelled by the base ref `B`.
/// `new` / `changed` are surfaced as the halting buckets; `fixed` / `persistent`
/// as informational; a non-empty `persistent` (i.e. a non-empty baseline on the
/// coord tree) is flagged as a warning to fix, not silently tolerated (INV-7).
pub(crate) fn render_delta(delta: &RegressionDelta, base: &str) -> String {
    let mut lines: Vec<String> = vec![format!("regression diff vs base {base}:")];
    let halting = delta.halting();
    if halting.is_empty() {
        lines.push("  ✓ no new or changed failures".to_string());
    } else {
        lines.push(format!("  ✗ {} halting failure(s):", halting.len()));
        lines.extend(delta.new.iter().map(|k| format!("    new:     {k}")));
        lines.extend(delta.changed.iter().map(|k| format!("    changed: {k}")));
    }
    lines.extend(delta.fixed.iter().map(|k| format!("    fixed:   {k}")));
    if !delta.persistent.is_empty() {
        lines.push(format!(
            "  ⚠ {} persistent (pre-existing) failure(s) on the coord tree — fix the trunk:",
            delta.persistent.len()
        ));
        lines.extend(
            delta
                .persistent
                .iter()
                .map(|k| format!("    persistent: {k}")),
        );
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

/// Strip volatile tokens from a raw failure excerpt so a failure whose only
/// change is an address / duration / tmp path / hash / line:col normalises to a
/// STABLE signature (no flap → stays `persistent`). Coarse by design: fine
/// enough to catch a changed *message*, blunt enough not to track noise.
fn normalise_sig(raw: &str) -> String {
    // (pattern, replacement) — order matters: 0x-addresses before bare hashes, so
    // an address is not double-counted. Each is a volatile token that would
    // otherwise flap the signature across otherwise-identical failures.
    let subs: &[(&str, &str)] = &[
        (r"0x[0-9a-fA-F]+", "0xADDR"),                  // pointers
        (r"\b\d+(?:\.\d+)?(?:ns|µs|us|ms|s)\b", "DUR"), // durations
        (r"/tmp/[^\s:)]+", "/tmp/TMP"),                 // tmp paths
        (r"target/[^\s:)]+", "target/PATH"),            // build paths
        (r"\.rs:\d+:\d+", ".rs:LN:CL"),                 // source line:col
        (r"\b[0-9a-f]{7,}\b", "HASH"),                  // bare hashes
    ];
    let mut s = raw.trim().to_string();
    for (pat, rep) in subs {
        if let Ok(re) = Regex::new(pat) {
            s = re.replace_all(&s, *rep).into_owned();
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obtained(pairs: &[(&str, &str)]) -> FailureSet {
        FailureSet::Obtained(
            pairs
                .iter()
                .map(|(k, s)| ((*k).to_string(), (*s).to_string()))
                .collect(),
        )
    }
    fn keys(set: &BTreeSet<String>) -> Vec<&str> {
        set.iter().map(String::as_str).collect()
    }

    // --- diff partition (VT-1, VT-4) ---

    #[test]
    fn diff_empty_both_is_green() {
        let d = diff(&obtained(&[]), &obtained(&[])).unwrap();
        assert_eq!(d, RegressionDelta::default());
        assert!(d.halting().is_empty());
    }

    #[test]
    fn diff_full_overlap_same_sig_is_all_persistent() {
        let b = obtained(&[("doctrine::a", "s1"), ("doctrine::b", "s2")]);
        let d = diff(&b, &b).unwrap();
        assert_eq!(keys(&d.persistent), vec!["doctrine::a", "doctrine::b"]);
        assert!(d.halting().is_empty());
    }

    #[test]
    fn diff_env_mask_surfaces_only_the_new_failure() {
        // baseline shares embed_fail + X with current; current adds new_fail.
        let base = obtained(&[("t::embed_fail", "e"), ("t::X", "x")]);
        let cur = obtained(&[("t::embed_fail", "e"), ("t::X", "x"), ("t::new_fail", "n")]);
        let d = diff(&base, &cur).unwrap();
        assert_eq!(keys(&d.new), vec!["t::new_fail"]);
        assert_eq!(keys(&d.persistent), vec!["t::X", "t::embed_fail"]);
        assert!(d.changed.is_empty());
    }

    #[test]
    fn diff_changed_sig_halts_same_sig_persists() {
        let base = obtained(&[("t::flaky", "old reason")]);
        let cur = obtained(&[("t::flaky", "NEW reason")]);
        let d = diff(&base, &cur).unwrap();
        assert_eq!(keys(&d.changed), vec!["t::flaky"]);
        assert!(d.persistent.is_empty());
        assert_eq!(keys(&d.halting()), vec!["t::flaky"]);
    }

    #[test]
    fn diff_fixed_bucket_is_baseline_minus_current() {
        let base = obtained(&[("t::gone", "g")]);
        let cur = obtained(&[]);
        let d = diff(&base, &cur).unwrap();
        assert_eq!(keys(&d.fixed), vec!["t::gone"]);
        assert!(d.halting().is_empty());
    }

    // --- Unobtainable (VT-3, INV-5) ---

    #[test]
    fn diff_errs_when_baseline_unobtainable() {
        let cur = obtained(&[]);
        let unob = FailureSet::Unobtainable {
            why: "compile error".into(),
        };
        assert!(diff(&unob, &cur).is_err());
    }

    #[test]
    fn diff_errs_when_current_unobtainable() {
        let base = obtained(&[]);
        let unob = FailureSet::Unobtainable {
            why: "panic mid-run".into(),
        };
        assert!(diff(&base, &unob).is_err());
    }

    // --- parse_failures (VT-2, A2) ---

    #[test]
    fn parse_section_aware_keys_disambiguate_same_named_tests() {
        // Same test name `roundtrip` fails in two different target binaries; the
        // section header target-qualifies the key (A2).
        let out = "\
     Running unittests src/main.rs (target/debug/deps/doctrine-aaa111)

failures:

---- roundtrip stdout ----
thread 'roundtrip' panicked at src/plan.rs:10:5:
assertion failed: lhs == rhs

failures:
    roundtrip

test result: FAILED. 1 passed; 1 failed; 0 ignored

     Running tests/e2e_cli.rs (target/debug/deps/e2e_cli-bbb222)

failures:

---- roundtrip stdout ----
thread 'roundtrip' panicked at tests/e2e_cli.rs:20:5:
the cli exploded

failures:
    roundtrip

test result: FAILED. 0 passed; 1 failed; 0 ignored
";
        let FailureSet::Obtained(map) = parse_failures(out) else {
            panic!("expected Obtained");
        };
        let keys: Vec<&str> = map.keys().map(String::as_str).collect();
        assert_eq!(keys, vec!["doctrine::roundtrip", "e2e_cli::roundtrip"]);
        // signatures differ (different panic messages) → a diff would see them.
        assert_ne!(map["doctrine::roundtrip"], map["e2e_cli::roundtrip"]);
    }

    #[test]
    fn parse_clean_run_is_obtained_empty() {
        let out = "\
     Running unittests src/main.rs (target/debug/deps/doctrine-aaa111)

test result: ok. 42 passed; 0 failed; 0 ignored
";
        assert_eq!(parse_failures(out), FailureSet::Obtained(BTreeMap::new()));
    }

    #[test]
    fn parse_unstructured_output_is_unobtainable() {
        // a compile error: no `running`/`test result:` structure at all.
        let out = "error[E0432]: unresolved import `crate::nope`\n  --> src/x.rs:1:5";
        assert!(matches!(
            parse_failures(out),
            FailureSet::Unobtainable { .. }
        ));
    }

    // --- signature stability (VT-5) ---

    #[test]
    fn parse_then_diff_volatile_token_only_change_stays_persistent() {
        let mk = |addr: &str, dur: &str| {
            format!(
                "\
     Running unittests src/main.rs (target/debug/deps/doctrine-{addr})

failures:

---- t stdout ----
thread 't' panicked at src/x.rs:9:5:
boom at 0x{addr} after {dur}

failures:
    t

test result: FAILED. 0 passed; 1 failed; 0 ignored
"
            )
        };
        let base = parse_failures(&mk("deadbeef", "12ms"));
        let cur = parse_failures(&mk("cafef00d", "999ms"));
        let d = diff(&base, &cur).unwrap();
        assert_eq!(keys(&d.persistent), vec!["doctrine::t"]);
        assert!(d.halting().is_empty(), "volatile-only diff must not halt");
    }

    // --- render (informational, exercises the halting-warning path) ---

    #[test]
    fn render_names_the_base_and_the_halting_keys() {
        let mut d = RegressionDelta::default();
        d.new.insert("t::regressed".into());
        let out = render_delta(&d, "B0");
        assert!(out.contains("B0"));
        assert!(out.contains("t::regressed"));
    }
}
