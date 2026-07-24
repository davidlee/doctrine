// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine check {quick|commit|gate}` — the cadence proxy verb (SL-163).
//!
//! Resolves a project-declared check command from the OWNED `[verification]`
//! contract and proxy-executes it: inherit stdio, no timeout, forward the exit
//! code (incl. `128+signo` on signal death). The pure cadence resolution lives in
//! the `verify` leaf ([`crate::verify::resolve_check`]); this module is the impure
//! shell (ADR-001) — root detection, the config read, spawn, and exit forwarding.
//!
//! The OPPOSITE posture to `coverage_verify::run_argv` (pipe + capture + cap): a
//! dev gate streams live and may legitimately run long (design §5.4 / D5).

use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use anyhow::bail;
use clap::Subcommand;

use crate::verify::{CheckKind, CheckPlan};

/// The three check cadences as a clap subcommand. Each carries `-p/--path` (CR-F6),
/// threaded to root detection (aids e2e temp-root). clap-owned (ADR-001 / A2) —
/// bridges to the leaf [`CheckKind`] via [`From`], keeping the leaf clap-free.
#[derive(Debug, Subcommand)]
pub(crate) enum CheckCommand {
    /// Per-edit cadence. Unconfigured ⇒ an owned no-op (exit 0; never fails a hook).
    Quick {
        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
    /// Per-commit cadence. Unconfigured ⇒ `just check`.
    Commit {
        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
    /// End-of-phase cadence. Unconfigured ⇒ `just gate`.
    Gate {
        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
    /// Non-mutating prove-clean cadence — asserts fmt+lint clean, never fixes.
    /// Unconfigured ⇒ `just prove`.
    Prove {
        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
    /// S1 regression baseline-diff (SL-170) — capture a failure-set baseline at a
    /// base ref, then diff a later run against it. NOT a cadence proxy: this verb
    /// runs the per-test suite itself and partitions failures (new/changed/fixed/
    /// persistent), halting on a regression regardless of which env it surfaces under.
    Regression {
        #[command(subcommand)]
        command: RegressionCommand,
    },
    /// Check a slice plan's VT rows carry structured mandates (IMP-209).
    ///
    /// Reads the authored `plan.toml`, flags every non-waived VT without
    /// `test_file` (`BareTestFile`) or with empty `keywords` (`BareKeywords`),
    /// and warns on waived VTs with no `waived_reason` (`MissingWaiverReason`).
    /// VA/VH rows are skipped; exit non-zero iff any bare VT exists.
    Plan {
        /// Slice id, e.g. 209. Accepts `SL-209` or bare `209`.
        #[arg(value_parser = crate::slice::parse_cli_id)]
        id: u32,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

/// `doctrine check regression {capture|diff}` — the S1 gate verbs. `--base <B>` is
/// the funnel's live pre-spawn HEAD (INV-2), the cache key + render label.
#[derive(Debug, Subcommand)]
pub(crate) enum RegressionCommand {
    /// Run the suite on the coord tree (at `B`) and record `baseline-<B>`; no-op
    /// on a cache hit.
    Capture {
        /// The base ref `B` (the funnel's pre-spawn HEAD) — cache key + label.
        #[arg(long)]
        base: String,
        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
    /// Run the suite at `S`, diff against `baseline-<B>`, and exit non-zero iff a
    /// new or changed failure surfaced (INV-7), or a run was unobtainable (INV-5).
    Diff {
        /// The base ref `B` whose baseline to diff against.
        #[arg(long)]
        base: String,
        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

/// `doctrine check <kind>` — resolve the cadence's plan from the owned config and
/// act: print + exit-0 the owned no-op, error toward the key on an empty override,
/// or proxy-spawn the resolved argv (diverging via [`run_proxy`]).
pub(crate) fn dispatch(cmd: CheckCommand) -> anyhow::Result<()> {
    use std::io::Write;
    // `regression` and `plan` are real handlers, not cadence proxies.
    let (kind, path) = match cmd {
        CheckCommand::Regression { command } => return dispatch_regression(command),
        CheckCommand::Plan { id, path } => return run_check_plan(path, id),
        CheckCommand::Quick { path } => (CheckKind::Quick, path),
        CheckCommand::Commit { path } => (CheckKind::Commit, path),
        CheckCommand::Gate { path } => (CheckKind::Gate, path),
        CheckCommand::Prove { path } => (CheckKind::Prove, path),
    };
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let cfg = crate::coverage_store::load_config(&root)?;
    match crate::verify::resolve_check(&cfg, kind) {
        CheckPlan::Noop(note) => {
            writeln!(std::io::stdout(), "{note}")?;
            // Owned no-op: doctrine exits 0 itself — no child spawned (CR-F3).
            #[expect(
                clippy::disallowed_methods,
                reason = "the check verb forwards a terminal exit status; an owned no-op exits 0 (design §5.4)"
            )]
            {
                std::process::exit(0);
            }
        }
        CheckPlan::Empty(k) => bail!(
            "[verification].{} is empty — set a non-empty argv in {}",
            k.key(),
            crate::dtoml::DOCTRINE_TOML
        ),
        CheckPlan::Run(argv) => run_proxy(&root, &argv, kind),
    }
}

/// Route a `check regression {capture|diff}` invocation: resolve the root, then
/// capture (write a baseline) or diff (exit non-zero on a regression — INV-7).
/// `diff` forwards its computed exit code as the verb's terminal status.
fn dispatch_regression(cmd: RegressionCommand) -> anyhow::Result<()> {
    match cmd {
        RegressionCommand::Capture { base, path } => {
            let root = crate::root::find(path, &crate::root::default_markers())?;
            crate::regression_run::run_capture(&root, &base)
        }
        RegressionCommand::Diff { base, path } => {
            let root = crate::root::find(path, &crate::root::default_markers())?;
            let code = crate::regression_run::run_diff(&root, &base)?;
            #[expect(
                clippy::disallowed_methods,
                reason = "the regression diff verb forwards its gate verdict as the terminal exit code (INV-7)"
            )]
            {
                std::process::exit(code);
            }
        }
    }
}

/// `doctrine check plan <id>` — read the authored `plan.toml` + the slice's
/// design-target selectors and flag both bare VT rows (IMP-209) and VTs whose
/// `test_file` is declared by no design-target selector (SL-224). The thin exit
/// shell: [`plan_check_report`] does the read+render+decide; this diverges via
/// `process::exit(1)` on the decision (`MissingWaiverReason` is a soft warning).
fn run_check_plan(path: Option<PathBuf>, id: u32) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (report, fail) = plan_check_report(&root, id)?;
    write!(std::io::stdout(), "{report}")?;
    if fail {
        #[expect(
            clippy::disallowed_methods,
            reason = "check plan exits non-zero on bare VTs (IMP-209) or undeclared VT test_files (SL-224)"
        )]
        {
            std::process::exit(1);
        }
    }
    Ok(())
}

/// Read the authored plan + the slice's design-target selectors, render the
/// check-plan report, and return `(rendered_output, should_exit_nonzero)`. Split
/// from [`run_check_plan`] so the read+render+decide is exercisable without the
/// process-exit shell (SL-224). Fails iff any non-waived VT lacks `test_file` /
/// has empty `keywords`, OR any non-waived VT's `test_file` is declared by no
/// design-target selector (the import belt would refuse it at integrate time).
fn plan_check_report(root: &Path, id: u32) -> anyhow::Result<(String, bool)> {
    use std::fmt::Write as _;
    let slice_root = root.join(".doctrine/slice");
    let plan = crate::slice::read_plan(&slice_root, id)?;
    let findings = crate::plan::check_vt_shape(&plan);

    // Selector-completeness (SL-224): read the SAME design-target selectors the
    // dispatch scope belt reads (dispatch.rs). Computed ABOVE the clean short-circuit
    // so a plan with no shape problems (obj-2's primary case) is still gated on it.
    let selectors =
        crate::slice::selectors(root, id, Some(crate::slice::SelectorIntent::DesignTarget))?;
    let coverage = crate::plan::undeclared_test_files(&plan, &selectors);

    let mut out = String::new();
    if findings.is_empty() && coverage.is_empty() {
        writeln!(out, "SL-{id:03}: all VT rows carry structured mandates")?;
        return Ok((out, false));
    }

    let bare_count = findings
        .iter()
        .filter(|f| !matches!(f.problem, crate::plan::VtShapeProblem::MissingWaiverReason))
        .count();
    let opaque_count = findings.len() - bare_count;

    if bare_count > 0 {
        writeln!(out, "SL-{id:03}: {bare_count} bare VT(s) found")?;
    }
    if opaque_count > 0 {
        writeln!(
            out,
            "  ({opaque_count} opaque waiver(s) — add waived_reason)"
        )?;
    }
    if !coverage.is_empty() {
        writeln!(
            out,
            "SL-{id:03}: {} undeclared VT test_file(s) found",
            coverage.len()
        )?;
    }
    writeln!(out)?;

    let mut lines: Vec<String> = Vec::new();
    for f in &findings {
        let (label, detail) = match f.problem {
            crate::plan::VtShapeProblem::BareTestFile => (
                "BareTestFile",
                "no test_file — VT is UNCHECKABLE at runtime",
            ),
            crate::plan::VtShapeProblem::BareKeywords => {
                ("BareKeywords", "keywords empty — vacuous pass at runtime")
            }
            crate::plan::VtShapeProblem::MissingWaiverReason => (
                "MissingWaiverReason",
                "waived but no reason recorded — opaque",
            ),
        };
        lines.push(format!(
            "  {:<12} {:<8} {:<24} — {detail}",
            f.phase_id, f.vt_id, label
        ));
    }
    for f in &coverage {
        lines.push(format!(
            "  {:<12} {:<8} {:<24} — test_file `{}` declared by no design-target selector",
            f.phase_id, f.vt_id, "UndeclaredTestFile", f.path
        ));
    }
    // Sort by phase then VT id for stable output.
    lines.sort();
    for line in &lines {
        writeln!(out, "{line}")?;
    }

    if bare_count > 0 {
        writeln!(
            out,
            "\nAdd `test_file` + `keywords` to each bare VT row. Re-run until clean."
        )?;
    }
    if !coverage.is_empty() {
        // The SAME id-bearing remediation formatter the import belt emits (SL-224
        // PHASE-01) — one runnable `slice selector add` line per undeclared path.
        let paths: Vec<String> = coverage.iter().map(|f| f.path.clone()).collect();
        write!(
            out,
            "\n{}",
            crate::conformance::undeclared_detail(id, &paths)
        )?;
    }

    let fail = bare_count > 0 || !coverage.is_empty();
    Ok((out, fail))
}

/// Proxy-spawn `argv` with `cwd == root`, INHERITING stdio (live stream; not piped)
/// and NO timeout (design §5.4 / D5). Reached only with a non-empty `argv` (INV-2).
/// Diverges via [`std::process::exit`] on a completed child — safe, stdio is
/// inherited so nothing is buffered/owned to flush (R2). A missing program
/// (`ENOENT`) yields an actionable error naming the owned config key (D3).
fn run_proxy(root: &Path, argv: &[String], kind: CheckKind) -> anyhow::Result<()> {
    let Some((program, args)) = argv.split_first() else {
        // INV-2: resolve_check never yields Run([]) — defend rather than panic.
        bail!("internal: empty check argv (resolve_check INV-2 violated)");
    };
    let status = Command::new(program)
        .args(args)
        .current_dir(root)
        .status()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!(
                    "`{program}` not found — set [verification].{} in {}",
                    kind.key(),
                    crate::dtoml::DOCTRINE_TOML
                )
            } else {
                anyhow::Error::new(e).context(format!("failed to spawn `{program}`"))
            }
        })?;
    #[expect(
        clippy::disallowed_methods,
        reason = "true exit forwarding (CR-F5): the proxied child's code/signal is the verb's terminal status (design §5.4)"
    )]
    {
        std::process::exit(exit_code(status));
    }
}

/// True exit forwarding (CR-F5): a normal exit yields its code; a signal-killed
/// child re-exits `128 + signo` (shell convention), not a flattened `1`. Unix-only
/// branch; doctrine targets linux/nixos.
fn exit_code(status: ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        status.signal().map_or(1, |s| 128 + s)
    }
    #[cfg(not(unix))]
    {
        1
    }
}

#[cfg(test)]
mod check_plan_report_tests {
    use super::*;

    /// Scaffold a COMPLETE slice at `root`: `plan.toml`, a full `slice-NNN.toml`
    /// carrying the given design-target `[[selector]]` line (EX-5 — a bare
    /// `plan.toml` alone would error at `slice::selectors`), and the `slice-NNN.md`
    /// body `read_slice` reads. `test_file` is the VT-1 `test_file` under test.
    fn scaffold(id: u32, selector: &str, test_file: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let slice_dir = dir.path().join(".doctrine/slice").join(format!("{id:03}"));
        std::fs::create_dir_all(&slice_dir).unwrap();
        std::fs::write(
            slice_dir.join("plan.toml"),
            format!(
                r#"
[[phase]]
id = "PHASE-01"
verification = [
  {{ id = "VT-1", test_file = "{test_file}", keywords = ["fn"] }},
]
"#
            ),
        )
        .unwrap();
        std::fs::write(
            slice_dir.join(format!("slice-{id:03}.toml")),
            format!(
                r#"id = {id}
slug = "test-slice"
title = "Test Slice"
status = "started"
created = "2026-07-24"
updated = "2026-07-24"
tags = []

[[selector]]
selector = "{selector}"
intent = "design-target"
"#
            ),
        )
        .unwrap();
        std::fs::write(
            slice_dir.join(format!("slice-{id:03}.md")),
            "# Test Slice\n",
        )
        .unwrap();
        dir
    }

    #[test]
    fn covered_test_file_reports_clean_no_fail() {
        // Design-target selector covers VT-1's test_file: undeclared_test_files is
        // empty, so the clean short-circuit fires and the report does NOT fail.
        let dir = scaffold(42, "src/plan.rs", "src/plan.rs");
        let (report, fail) = plan_check_report(dir.path(), 42).unwrap();
        assert!(!fail);
        assert!(report.contains("all VT rows carry structured mandates"));
    }

    #[test]
    fn undeclared_test_file_folds_into_fail_and_names_path() {
        // The design-target selector (read via slice::selectors with
        // SelectorIntent::DesignTarget) does NOT cover VT-1's test_file, so
        // undeclared_test_files flags it and the report FAILS naming the path.
        let dir = scaffold(42, "src/other.rs", "src/plan.rs");
        let (report, fail) = plan_check_report(dir.path(), 42).unwrap();
        assert!(fail);
        // The finding line names the undeclared test_file path.
        assert!(report.contains("UndeclaredTestFile"));
        assert!(report.contains("src/plan.rs"));
        // The shared id-bearing remediation footer is emitted.
        assert!(
            report
                .contains("doctrine slice selector add SL-042 src/plan.rs --intent design-target")
        );
    }
}
