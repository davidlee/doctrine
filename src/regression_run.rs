// SPDX-License-Identifier: GPL-3.0-only
//! `regression_run` — the IMPURE shell for the S1 regression gate (SL-170
//! PHASE-02). Runs the project's per-test suite, parses it through the pure
//! [`crate::regression`] leaf, and memoises a baseline failure-set keyed by
//! `(base-sha, run-fingerprint)` under the disposable
//! `.doctrine/state/regression/` tier.
//!
//! Two CLI verbs (`doctrine check regression {capture,diff}`):
//!
//! - `capture --base <B>` — run the suite on the coord tree positioned at `B`,
//!   parse, and write `baseline-<B>-<fp>`. No-op on a cache hit (carry-forward).
//! - `diff --base <B>` — run the suite at the integrated `S`, load `baseline-<B>`,
//!   [`regression::diff`], print [`regression::render_delta`], and exit non-zero
//!   iff `new ∪ changed ≠ ∅` (INV-7) OR a side is `Unobtainable` (INV-5).
//!
//! `B` comes SOLELY from `--base` — the funnel's live pre-spawn HEAD — never an
//! internal registry / `code_start_oid` read (INV-2). The run-fingerprint binds
//! the suite argv + filter state (`DOCTRINE_WORKER`, worker marker) + doctrine-bin
//! provenance, so a tainted run can never poison a later baseline (INV-8): a
//! fingerprint mismatch is a cache MISS → honest re-capture.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::git;
use crate::regression::{self, FailureSet};

/// Directory holding sha-keyed baseline failure-sets (disposable runtime tier).
const CACHE_DIR: &str = ".doctrine/state/regression";

/// Run the configured suite at `root`, returning a parsed [`FailureSet`]. A spawn
/// failure (bad argv / missing program) is `Unobtainable` (INV-5) — never a silent
/// clean ∅. stdout and stderr are concatenated so a panic on either stream is
/// seen by the section-aware parser.
fn run_suite(root: &Path, argv: &[String]) -> FailureSet {
    let Some((program, args)) = argv.split_first() else {
        return FailureSet::Unobtainable {
            why: "empty [verification].regression argv".into(),
        };
    };
    let output = Command::new(program).args(args).current_dir(root).output();
    let Ok(output) = output else {
        return FailureSet::Unobtainable {
            why: format!("failed to spawn suite `{program}`"),
        };
    };
    let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
    combined.push('\n');
    combined.push_str(&String::from_utf8_lossy(&output.stderr));
    regression::parse_failures(&combined)
}

/// The run-fingerprint (INV-8): a stable hash over the suite argv, the
/// test-selection / filter state (`DOCTRINE_WORKER` env, worker-marker presence)
/// and the doctrine-bin provenance (`current_exe`). Capture and diff that share
/// an identical environment compute an identical fingerprint; any drift in filter
/// state (a leaked marker / env, a swapped binary) changes it → cache miss.
fn fingerprint(root: &Path, argv: &[String]) -> String {
    let exe = std::env::current_exe()
        .ok()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let material = format!(
        "argv={argv:?}\nenv_worker={}\nmarker={}\nbin={exe}\n",
        crate::worktree::env_worker_set(),
        crate::worktree::marker_present(root),
    );
    git::sha256(material.as_bytes())
}

/// Path of the baseline cache file for `(base, fingerprint)`. The fingerprint is
/// truncated for a readable filename; collisions are negligible and a stale file
/// is at worst a cache miss (re-capture), never a correctness hazard (A3).
fn baseline_path(root: &Path, base: &str, fp: &str) -> PathBuf {
    let short_fp = &fp[..fp.len().min(16)];
    root.join(CACHE_DIR)
        .join(format!("baseline-{base}-{short_fp}"))
}

/// Serialize an `Obtained` failure-set to JSON for the baseline cache. Only an
/// `Obtained` set is ever persisted — an `Unobtainable` capture is a hard error
/// (INV-5), never written.
fn serialize_baseline(set: &FailureSet) -> Result<String> {
    match set {
        FailureSet::Obtained(map) => {
            serde_json::to_string_pretty(map).context("serialize baseline failure-set")
        }
        FailureSet::Unobtainable { why } => {
            bail!("refusing to persist an unobtainable suite run ({why}) — INV-5")
        }
    }
}

/// Load a baseline `Obtained` failure-set from its JSON cache file.
fn load_baseline(path: &Path) -> Result<FailureSet> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read baseline {}", path.display()))?;
    let map: BTreeMap<String, String> =
        serde_json::from_str(&raw).with_context(|| format!("parse baseline {}", path.display()))?;
    Ok(FailureSet::Obtained(map))
}

/// `doctrine check regression capture --base <B>` — run the suite on the coord
/// tree (positioned at `B` by the orchestrator) and write `baseline-<B>-<fp>`.
/// No-op on a cache hit (the carry-forward steady state — one suite run per batch).
pub(crate) fn run_capture(root: &Path, base: &str) -> Result<()> {
    use std::io::Write as _;
    let cfg = crate::coverage_store::load_config(root)?;
    let argv = cfg.regression_argv();
    let fp = fingerprint(root, &argv);
    let path = baseline_path(root, base, &fp);
    if path.exists() {
        writeln!(
            std::io::stdout(),
            "regression capture: cache hit for base {base} — no-op"
        )?;
        return Ok(());
    }
    let set = run_suite(root, &argv);
    let body = serialize_baseline(&set)?; // errors honestly on Unobtainable (INV-5)
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
    }
    #[expect(
        clippy::disallowed_methods,
        reason = "disposable runtime baseline cache under .doctrine/state (design §5.3)"
    )]
    std::fs::write(&path, body).with_context(|| format!("write baseline {}", path.display()))?;
    let n = match &set {
        FailureSet::Obtained(m) => m.len(),
        FailureSet::Unobtainable { .. } => 0,
    };
    writeln!(
        std::io::stdout(),
        "regression capture: base {base} — {n} baseline failure(s) recorded"
    )?;
    Ok(())
}

/// `doctrine check regression diff --base <B>` — run the suite at `S`, load
/// `baseline-<B>`, diff, render, and return the process exit code: `0` iff the
/// diff is `Ok` AND `new ∪ changed = ∅`, else `1`. An `Unobtainable` side or a
/// missing baseline is a hard non-zero (INV-5) — NEVER a silent green ∅.
pub(crate) fn run_diff(root: &Path, base: &str) -> Result<i32> {
    use std::io::Write as _;
    let cfg = crate::coverage_store::load_config(root)?;
    let argv = cfg.regression_argv();
    let fp = fingerprint(root, &argv);
    let path = baseline_path(root, base, &fp);
    if !path.exists() {
        bail!(
            "no baseline for base {base} under the current run-fingerprint — run `doctrine check regression capture --base {base}` first (INV-8 cache miss is honest, not a green ∅)"
        );
    }
    let baseline = load_baseline(&path)?;
    let current = run_suite(root, &argv);
    match regression::diff(&baseline, &current) {
        Ok(delta) => {
            write!(
                std::io::stdout(),
                "{}",
                regression::render_delta(&delta, base)
            )?;
            Ok(i32::from(!delta.halting().is_empty()))
        }
        Err(e) => {
            // INV-5: an unobtainable side is a hard failure, not a green ∅-pass.
            writeln!(std::io::stdout(), "regression diff: {e}")?;
            Ok(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn write_config(root: &Path, argv: &str) {
        let body = format!("[verification]\nregression = {argv}\n");
        fs::create_dir_all(root.join(".doctrine")).unwrap();
        fs::write(root.join(".doctrine/doctrine.toml"), body).unwrap();
    }

    #[test]
    fn fingerprint_is_stable_under_identical_env() {
        let root = tmp();
        let argv = vec!["cargo".to_string(), "test".to_string()];
        assert_eq!(
            fingerprint(root.path(), &argv),
            fingerprint(root.path(), &argv),
            "same env + argv ⇒ identical fingerprint (INV-1/INV-8)"
        );
    }

    #[test]
    fn fingerprint_differs_under_changed_filter_state() {
        let root = tmp();
        let a = vec!["cargo".to_string(), "test".to_string()];
        let b = vec!["cargo".to_string(), "nextest".to_string()];
        assert_ne!(
            fingerprint(root.path(), &a),
            fingerprint(root.path(), &b),
            "a changed suite argv ⇒ a different fingerprint"
        );
    }

    #[test]
    fn capture_writes_then_hits_cache() {
        let root = tmp();
        // `true` exits 0 with no output → parse → Unobtainable (no cargo
        // structure), which capture refuses to persist. Use `printf` of a clean
        // run instead so the suite "passes" with parseable structure.
        write_config(
            root.path(),
            r#"["printf", "running 1 tests\ntest result: ok. 1 passed; 0 failed;\n"]"#,
        );
        run_capture(root.path(), "BASE0").unwrap();
        // a baseline file now exists.
        let dir = root.path().join(CACHE_DIR);
        let n = fs::read_dir(&dir).unwrap().count();
        assert_eq!(n, 1, "one baseline written");
        // second capture is a cache-hit no-op (still one file).
        run_capture(root.path(), "BASE0").unwrap();
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
    }

    #[test]
    fn capture_refuses_to_persist_unobtainable() {
        let root = tmp();
        // `false` exits non-zero, no test structure → Unobtainable → INV-5 refusal.
        write_config(root.path(), r#"["false"]"#);
        assert!(
            run_capture(root.path(), "BASE0").is_err(),
            "an unobtainable run must not be cached as a green baseline"
        );
    }

    #[test]
    fn diff_without_baseline_is_hard_error() {
        let root = tmp();
        write_config(root.path(), r#"["printf", "test result: ok.\n"]"#);
        assert!(
            run_diff(root.path(), "MISSING").is_err(),
            "diff with no baseline halts (never a silent green ∅)"
        );
    }

    #[test]
    fn diff_green_when_current_matches_baseline() {
        let root = tmp();
        write_config(
            root.path(),
            r#"["printf", "running 1 tests\ntest result: ok.\n"]"#,
        );
        run_capture(root.path(), "B").unwrap();
        assert_eq!(
            run_diff(root.path(), "B").unwrap(),
            0,
            "no new failures ⇒ exit 0"
        );
    }
}
