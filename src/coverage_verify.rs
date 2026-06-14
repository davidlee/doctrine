// SPDX-License-Identifier: GPL-3.0-only
//! `coverage_verify` — the verifier shell: continuous re-derivation of `VT`
//! coverage status (SL-057 PHASE-04, design F-VII/F-VIII).
//!
//! This module is the IMPURE shell over the slice-side coverage store: it reads
//! `doctrine.toml`, resolves every `VT` entry's check against the project config,
//! RUNS each distinct resolved argv exactly once (global dedup across the whole
//! invocation — INV-2), folds each run's [`crate::coverage::RunOutcome`] through
//! the pure [`crate::coverage::derive_status`], re-stamps the git anchor on a fresh
//! observation, and writes the re-derived status back per slice. disk / git /
//! subprocess / monotonic-clock all live HERE — the verdict fold stays pure.
//!
//! Reuse, not parallel impl:
//! - config read parses through [`crate::dtoml::parse`] (the one `doctrine.toml`
//!   reader); absent file ⇒ [`crate::verify::VerificationConfig::default`];
//! - base resolution is [`crate::verify::resolve`] (the pure fold);
//! - matcher evaluation is [`crate::coverage::evaluate_matcher`];
//! - the verdict is [`crate::coverage::derive_status`];
//! - the store seam is [`crate::coverage_store::load`] / `save`;
//! - the within-file no-clobber fold is [`crate::coverage::upsert`];
//! - the git anchor is [`crate::git::head_sha`].
//!
//! **F-VII (Unobtainable ⇒ Blocked, never silent-Failed).** An unresolvable check,
//! a spawn failure, a wall-clock timeout, an unreadable/absent matcher file, OR an
//! unparseable hand-edited regex all collapse to [`crate::coverage::RunOutcome::Unobtainable`]
//! ⇒ `Blocked` — a config error is surfaced loud, never silently miscoloured Failed.
//!
//! **F-VIII (anchor freshness).** A `Ran` observation re-stamps `git_anchor` to the
//! current `HEAD`; an `Unobtainable` (`Blocked`) cell KEEPS its prior anchor, so a
//! never-observed cell still goes stale against later commits.

// PHASE-05 wires the CLI (`coverage verify`) onto `run`, so the verifier shell and
// its transitive references now have a live bins/lib consumer — the PHASE-04
// leaf-ahead-of-consumer dead_code blanket is retired.

use std::collections::BTreeMap;
use std::io::{Read, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;

use crate::coverage::{self, CoverageFile, CoverageKey, MatchSource, RunOutcome};
use crate::coverage_store;
use crate::git;
use crate::requirement::CoverageStatus;
use crate::verify::{self, Resolved, VerificationConfig};

/// The per-entry verification line in the [`Report`]: the cited key, the status the
/// cell carried BEFORE this invocation, and the re-derived status AFTER. `exit_code_only`
/// flags a cell whose check is a literal `command` with NO matcher (D3/A) — its
/// verdict rides the exit code alone, so it is surfaced for audit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EntryReport {
    pub(crate) key: CoverageKey,
    pub(crate) old_status: CoverageStatus,
    pub(crate) new_status: CoverageStatus,
    pub(crate) exit_code_only: bool,
}

/// A check-less `VT` entry surfaced in the [`Report`]'s backfill list: it is
/// REPORTED and LEFT UNTOUCHED (no run, no auto-`Blocked`) — its key names the cell
/// an author must backfill a check onto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BackfillEntry {
    pub(crate) key: CoverageKey,
    pub(crate) status: CoverageStatus,
}

/// The structured, testable result of a [`run`] invocation (T6). Holds the
/// per-entry verification lines, the check-less-`VT` backfill list, and the
/// exit-code-only count. Printing is PHASE-05's CLI job — `run` RETURNS this.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Report {
    /// One line per VERIFIED (runnable) entry — its key + status transition.
    pub(crate) verified: Vec<EntryReport>,
    /// Check-less `VT` entries — reported, untouched (the backfill set).
    pub(crate) backfill: Vec<BackfillEntry>,
}

impl Report {
    /// The count of check-less `VT` entries needing a backfill (the loud
    /// `"N VT entries lack a check — backfill"` figure; rendered by PHASE-05).
    pub(crate) fn backfill_count(&self) -> usize {
        self.backfill.len()
    }

    /// The count of exit-code-only cells (literal command, no matcher) — flagged
    /// for audit (D3/A).
    pub(crate) fn exit_code_only_count(&self) -> usize {
        self.verified.iter().filter(|e| e.exit_code_only).count()
    }
}

/// The cached result of running ONE distinct resolved argv: the captured streams +
/// the exit-code verdict, OR a marker that the run could not be obtained at all
/// (spawn failure / wall-clock timeout / empty argv) — the latter folds to
/// [`RunOutcome::Unobtainable`].
enum RunResult {
    /// The argv ran to completion: its exit-code verdict + captured streams.
    Ran {
        exit_ok: bool,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
    /// The argv could not be obtained (spawn failure / timeout / empty) ⇒ Blocked.
    Unobtainable,
}

/// Re-derive `VT` coverage status across a SET of slices (IMPURE: disk / git /
/// subprocess / clock). The slice SET is deliberate (F-2/INV-2): dedup of the
/// distinct resolved argvs spans EVERY entry across ALL passed slices, so a
/// `cargo test` shared by N slices runs ONCE per invocation, not once per slice.
/// WRITES are per-slice — each slice whose file changed is saved back.
///
/// Contract:
/// - config: `<root>/doctrine.toml` → [`crate::dtoml::parse`]`.verification`; an
///   ABSENT file ⇒ [`VerificationConfig::default`] (NOT a defaulted-green path —
///   a default-base check with no `command` is `NoRunnable` ⇒ Unobtainable ⇒ Blocked).
/// - For each runnable `VT` entry (`key.mode == "VT"`, `check.is_some()`):
///   [`verify::resolve`]; an `Err` ⇒ [`RunOutcome::Unobtainable`] (NOT run). The
///   resolvable ones group by their resolved `argv` and each DISTINCT argv runs
///   EXACTLY ONCE.
/// - Per-entry verdict: from the argv's cached [`RunResult`] + the resolved match
///   source, fold a [`RunOutcome`] (matcher over the picked haystack), then
///   [`coverage::derive_status`].
/// - Write-back (F-VIII): set `status`; re-stamp `git_anchor` to `HEAD` ONLY on a
///   `Ran` outcome; `upsert` + [`coverage_store::save`] each changed slice.
/// - Check-less `VT` entries are reported in the backfill list and LEFT UNTOUCHED.
pub(crate) fn run(root: &Path, slice_ids: &[u32]) -> Result<Report> {
    let cfg = coverage_store::load_config(root)?;
    let head = git::head_sha(root);

    // Load every slice's file up front (held by slice for per-slice write-back).
    let mut files: Vec<(u32, CoverageFile)> = Vec::with_capacity(slice_ids.len());
    for &slice_id in slice_ids {
        files.push((slice_id, coverage_store::load(root, slice_id)?));
    }

    // GLOBAL run cache: each DISTINCT resolved argv runs at most once across the
    // whole invocation (INV-2). Keyed by the full argv vec.
    let mut run_cache: BTreeMap<Vec<String>, RunResult> = BTreeMap::new();
    let mut report = Report::default();

    for (slice_id, file) in &mut files {
        let mut changed = false;
        for entry in &mut file.entry {
            if entry.key.mode != "VT" {
                continue;
            }
            let Some(check) = entry.check.clone() else {
                // Check-less VT entry: reported, left untouched (T6 backfill).
                report.backfill.push(BackfillEntry {
                    key: entry.key.clone(),
                    status: entry.status,
                });
                continue;
            };

            // Resolve; an unresolvable check is Unobtainable (NOT run) — F-VII.
            let outcome = match verify::resolve(&cfg, &check) {
                Err(_) => RunOutcome::Unobtainable,
                Ok(resolved) => {
                    let result = run_argv_cached(root, &cfg, &resolved.argv, &mut run_cache);
                    outcome_for(root, result, &check, &resolved)
                }
            };

            let old_status = entry.status;
            let new_status = coverage::derive_status(&outcome);

            // F-VIII: re-stamp the anchor ONLY on a fresh `Ran` observation; a
            // Blocked/Unobtainable cell KEEPS its prior anchor.
            let ran = matches!(outcome, RunOutcome::Ran { .. });
            if let (true, Some(h)) = (ran, &head) {
                entry.git_anchor.clone_from(h);
            }
            entry.status = new_status;
            changed = true;

            // Exit-code-only = a literal command with no matcher (D3/A).
            let exit_code_only = check.command.is_some() && check.matcher.is_none();
            report.verified.push(EntryReport {
                key: entry.key.clone(),
                old_status,
                new_status,
                exit_code_only,
            });
        }
        if changed {
            coverage_store::save(root, *slice_id, file)?;
        }
    }

    Ok(report)
}

/// The slice-tree dir under `<root>` enumerated for `coverage verify --all`.
const SLICE_DIR: &str = ".doctrine/slice";

/// Enumerate every slice id in `<root>/.doctrine/slice` — the `--all` set. Skips
/// the `NNN-slug` alias symlinks (ISS-006) and any non-numeric dir; an absent tree
/// is the empty set. Sorted ascending for a deterministic invocation order.
fn all_slice_ids(root: &Path) -> Result<Vec<u32>> {
    let mut ids = Vec::new();
    let dir = root.join(SLICE_DIR);
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(ids),
        Err(e) => return Err(anyhow::anyhow!("failed to read {}: {e}", dir.display())),
    };
    for entry in entries.flatten() {
        // Skip the slug-alias symlink — it re-points at a numeric dir already seen.
        if entry.file_type().is_ok_and(|t| t.is_symlink()) {
            continue;
        }
        if let Some(id) = entry
            .file_name()
            .to_str()
            .and_then(|n| n.parse::<u32>().ok())
        {
            ids.push(id);
        }
    }
    ids.sort_unstable();
    Ok(ids)
}

/// `doctrine coverage verify <slice>|--all` — the verifier CLI shell: resolve the
/// root + slice set, [`run`] the re-derivation, and PRINT the [`Report`] (per-entry
/// `key: old→new`, exit-code-only flags, and the loud backfill line). Exactly one
/// of `slice` / `--all` is required.
pub(crate) fn run_cli(path: Option<PathBuf>, slice: Option<&str>, all: bool) -> Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let slice_ids = match (slice, all) {
        (Some(_), true) => anyhow::bail!("pass a single <slice> OR --all, not both"),
        (None, false) => anyhow::bail!("pass a single <slice> or --all"),
        (Some(s), false) => vec![crate::slice::parse_ref(s)?],
        (None, true) => all_slice_ids(&root)?,
    };

    let report = run(&root, &slice_ids)?;
    print_report(&report)
}

/// The terse display token for a [`CoverageStatus`] in the report (the Debug name,
/// matching `coverage_store::withdrawal_line`'s `[Failed]` register). Routed through
/// a single `format!` so the report's status rendering has one source.
fn status_label(status: CoverageStatus) -> String {
    format!("{status:?}")
}

/// Print a verifier [`Report`]: one `key: old→new` line per re-derived entry (with
/// an `[exit-code-only]` flag where applicable), then the loud backfill line naming
/// how many `VT` entries still lack a check.
fn print_report(report: &Report) -> Result<()> {
    let mut out = std::io::stdout();
    for e in &report.verified {
        let flag = if e.exit_code_only {
            " [exit-code-only]"
        } else {
            ""
        };
        let (old, new) = (status_label(e.old_status), status_label(e.new_status));
        writeln!(
            out,
            "{}/{}/{}/{}: {old}→{new}{flag}",
            e.key.slice, e.key.requirement, e.key.contributing_change, e.key.mode,
        )?;
    }
    for b in &report.backfill {
        writeln!(
            out,
            "{}/{}/{}/{}: no check — backfill",
            b.key.slice, b.key.requirement, b.key.contributing_change, b.key.mode,
        )?;
    }
    writeln!(
        out,
        "{} VT entries lack a check — backfill",
        report.backfill_count(),
    )?;
    if report.exit_code_only_count() > 0 {
        writeln!(
            out,
            "{} exit-code-only cells (no matcher) — audit",
            report.exit_code_only_count(),
        )?;
    }
    Ok(())
}

/// Fold the cached [`RunResult`] of an entry's resolved argv into a [`RunOutcome`].
/// `Unobtainable` stays Unobtainable. A `Ran` with no matcher ⇒ exit-code-only;
/// with a matcher, pick the haystack by the resolved `source`, evaluate, and map
/// an unparseable regex / unreadable-or-absent file to `Unobtainable` (F-3/F-VII).
fn outcome_for(
    root: &Path,
    result: &RunResult,
    check: &coverage::VtCheck,
    resolved: &Resolved,
) -> RunOutcome {
    let (exit_ok, stdout, stderr) = match result {
        RunResult::Unobtainable => return RunOutcome::Unobtainable,
        RunResult::Ran {
            exit_ok,
            stdout,
            stderr,
        } => (*exit_ok, stdout, stderr),
    };

    let Some(matcher) = &check.matcher else {
        // No matcher ⇒ exit-code-only.
        return RunOutcome::Ran {
            exit_ok,
            matched: None,
        };
    };

    // Pick the haystack by the source the resolve chose (entry → default → Stdout).
    let haystack = match &resolved.source {
        MatchSource::Stdout => String::from_utf8_lossy(stdout).into_owned(),
        MatchSource::Stderr => String::from_utf8_lossy(stderr).into_owned(),
        MatchSource::File(glob) => match read_file_haystack(root, glob) {
            Some(text) => text,
            // Empty match-set OR any unreadable file ⇒ Unobtainable (skip evaluate).
            None => return RunOutcome::Unobtainable,
        },
    };

    match coverage::evaluate_matcher(&matcher.pattern, matcher.regex, &haystack) {
        Some(matched) => RunOutcome::Ran {
            exit_ok,
            matched: Some(matched),
        },
        // Unparseable hand-edited regex ⇒ Unobtainable (config error ⇒ Blocked, F-3).
        None => RunOutcome::Unobtainable,
    }
}

/// Resolve a `File` matcher glob UNDER `root` and concatenate every match's
/// contents (ANY-match over the concatenation). The glob is `root.join`ed (so an
/// ABSOLUTE hand-edited glob is discarded onto root by join) AND every matched
/// path is canonically CONFINED to root — a `..`-ascending match that resolves
/// outside the (canonicalized) root tree is dropped, so a hand-edited escaping
/// glob finds nothing (the F-III confinement, restated at the run seam where
/// `coverage::valid` cannot guard a hand-edited store). `None` ⇒ an empty
/// match-set OR any unreadable match (⇒ the caller maps to Unobtainable).
fn read_file_haystack(root: &Path, glob: &str) -> Option<String> {
    // Canonical root: the containment fence. If root itself cannot canonicalize
    // (shouldn't happen — it exists), nothing is reachable.
    let root_canon = std::fs::canonicalize(root).ok()?;
    let pattern = root.join(glob);
    let pattern_str = pattern.to_str()?;
    let paths = glob::glob(pattern_str).ok()?;

    let mut concatenated = String::new();
    let mut any = false;
    for entry in paths {
        let path = entry.ok()?;
        // Confine to root: a match whose canonical path escapes the root tree is
        // dropped (absolute / `..`-escaping hand-edited glob ⇒ finds nothing).
        let canon = std::fs::canonicalize(&path).ok()?;
        if !canon.starts_with(&root_canon) {
            return None;
        }
        // A glob can surface a dir; reading it errors ⇒ Unobtainable (None).
        let body = std::fs::read_to_string(&canon).ok()?;
        concatenated.push_str(&body);
        any = true;
    }
    if any { Some(concatenated) } else { None }
}

/// Run ONE distinct resolved argv at most once (the GLOBAL dedup cache, INV-2),
/// returning a borrow of the cached [`RunResult`]. A cache hit re-borrows; a miss
/// runs [`run_argv`] and stores it.
fn run_argv_cached<'cache>(
    root: &Path,
    cfg: &VerificationConfig,
    argv: &[String],
    cache: &'cache mut BTreeMap<Vec<String>, RunResult>,
) -> &'cache RunResult {
    cache
        .entry(argv.to_vec())
        .or_insert_with(|| run_argv(root, cfg, argv))
}

/// Run one argv with `cwd == root` under a wall-clock cap (IMPURE; std-only). Each
/// pipe is drained on its OWN thread (mandatory — a single-threaded read of one
/// pipe deadlocks when the child fills the other). The cap is a poll loop over
/// `try_wait` against an `Instant` deadline; on expiry the child is killed and the
/// run is `Unobtainable`. A spawn failure (bad argv) or an empty argv is also
/// `Unobtainable`.
fn run_argv(root: &Path, cfg: &VerificationConfig, argv: &[String]) -> RunResult {
    let Some((program, args)) = argv.split_first() else {
        return RunResult::Unobtainable; // empty argv (guarded; shouldn't happen post-resolve)
    };

    let spawned = Command::new(program)
        .args(args)
        .current_dir(root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    // A spawn failure (bad argv / missing binary) ⇒ Unobtainable.
    let Ok(mut child) = spawned else {
        return RunResult::Unobtainable;
    };

    // Drain EACH pipe on its own thread (avoids the pipe-buffer deadlock). If a
    // handle is missing the drainer yields an empty buffer.
    let stdout_handle = child.stdout.take();
    let stderr_handle = child.stderr.take();
    let oh = thread::spawn(move || drain(stdout_handle));
    let eh = thread::spawn(move || drain(stderr_handle));

    let deadline = Instant::now() + Duration::from_secs(cfg.timeout_secs());
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = oh.join().unwrap_or_default();
                let stderr = eh.join().unwrap_or_default();
                return RunResult::Ran {
                    exit_ok: status.success(),
                    stdout,
                    stderr,
                };
            }
            // Still running: kill on deadline (timeout), else poll again.
            Ok(None) if Instant::now() >= deadline => {
                reap(&mut child, oh, eh);
                return RunResult::Unobtainable; // wall-clock timeout
            }
            Ok(None) => thread::sleep(Duration::from_millis(50)),
            // A `try_wait` failure is itself unobtainable — reap and bail.
            Err(_) => {
                reap(&mut child, oh, eh);
                return RunResult::Unobtainable;
            }
        }
    }
}

/// Kill the child and join its drain threads (the timeout / wait-failure cleanup).
/// Every discarded `Result` goes through [`drop`] (the repo idiom for a `must_use`
/// value that is genuinely best-effort cleanup).
fn reap(
    child: &mut std::process::Child,
    oh: thread::JoinHandle<Vec<u8>>,
    eh: thread::JoinHandle<Vec<u8>>,
) {
    drop(child.kill());
    drop(child.wait());
    // Let the drain threads observe the closed pipes and finish.
    drop(oh.join());
    drop(eh.join());
}

/// Drain a child pipe to a byte buffer on its own thread; a missing handle or a
/// read error yields what was read so far (empty on a missing handle).
fn drain<R: Read + Send + 'static>(handle: Option<R>) -> Vec<u8> {
    let mut buf = Vec::new();
    if let Some(mut h) = handle {
        drop(h.read_to_end(&mut buf));
    }
    buf
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "tests: fail-fast unwrap on disk round-trip / spawn is idiomatic"
)]
mod tests {
    use super::*;
    use crate::coverage::{CoverageEntry, Matcher, VtCheck};

    fn key(slice: &str, req: &str, change: &str, mode: &str) -> CoverageKey {
        CoverageKey {
            slice: slice.to_owned(),
            requirement: req.to_owned(),
            contributing_change: change.to_owned(),
            mode: mode.to_owned(),
        }
    }

    /// Build a `CoverageEntry` with a chosen status + check (the verifier's input).
    fn entry(k: CoverageKey, status: CoverageStatus, check: Option<VtCheck>) -> CoverageEntry {
        CoverageEntry {
            key: k,
            status,
            git_anchor: "prior-anchor".to_owned(),
            attested_date: None,
            touched_paths: vec!["src/x.rs".to_owned()],
            check,
        }
    }

    /// A literal-command check with an optional matcher.
    fn cmd_check(command: Vec<&str>, matcher: Option<Matcher>) -> VtCheck {
        VtCheck {
            alias: None,
            command: Some(command.into_iter().map(str::to_owned).collect()),
            extra_args: Vec::new(),
            matcher,
        }
    }

    fn matcher(source: Option<MatchSource>, pattern: &str, regex: bool) -> Matcher {
        Matcher {
            source,
            pattern: pattern.to_owned(),
            regex,
        }
    }

    /// Seed a slice's coverage.toml on disk with the given entries (via the store).
    fn seed(root: &Path, slice_id: u32, entries: Vec<CoverageEntry>) {
        let file = CoverageFile { entry: entries };
        coverage_store::save(root, slice_id, &file).unwrap();
    }

    /// Read a slice's coverage.toml back from disk.
    fn reload(root: &Path, slice_id: u32) -> CoverageFile {
        coverage_store::load(root, slice_id).unwrap()
    }

    /// Write a `doctrine.toml` with a 1-second timeout so the timeout VT is fast.
    fn write_doctrine_toml(root: &Path, body: &str) {
        std::fs::write(root.join("doctrine.toml"), body).unwrap();
    }

    fn status_of(file: &CoverageFile, req: &str) -> CoverageStatus {
        file.entry
            .iter()
            .find(|e| e.key.requirement == req)
            .unwrap()
            .status
    }

    // --- VT-1: GLOBAL dedup — one resolved argv runs ONCE -------------------

    #[test]
    fn distinct_argv_runs_exactly_once_across_entries() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // A command that APPENDS a marker line to a root-relative counter file each
        // time it runs. Three entries share the SAME resolved argv ⇒ it must run
        // ONCE (INV-2): the counter file must hold exactly one line.
        let argv = vec!["sh", "-c", "printf 'x\\n' >> counter.txt"];
        let entries = vec![
            entry(
                key("SL-057", "REQ-200", "SL-057", "VT"),
                CoverageStatus::Planned,
                Some(cmd_check(argv.clone(), None)),
            ),
            entry(
                key("SL-057", "REQ-201", "SL-057", "VT"),
                CoverageStatus::Planned,
                Some(cmd_check(argv.clone(), None)),
            ),
            entry(
                key("SL-057", "REQ-202", "SL-057", "VT"),
                CoverageStatus::Planned,
                Some(cmd_check(argv.clone(), None)),
            ),
        ];
        seed(root, 57, entries);

        run(root, &[57]).unwrap();

        let counter = std::fs::read_to_string(root.join("counter.txt")).unwrap();
        assert_eq!(
            counter.lines().count(),
            1,
            "the shared argv ran exactly once across all three entries (INV-2)"
        );
    }

    #[test]
    fn dedup_spans_multiple_slices() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let argv = vec!["sh", "-c", "printf 'x\\n' >> counter.txt"];
        // Same resolved argv across TWO slices ⇒ still ONE run (global dedup).
        seed(
            root,
            57,
            vec![entry(
                key("SL-057", "REQ-200", "SL-057", "VT"),
                CoverageStatus::Planned,
                Some(cmd_check(argv.clone(), None)),
            )],
        );
        seed(
            root,
            58,
            vec![entry(
                key("SL-058", "REQ-300", "SL-058", "VT"),
                CoverageStatus::Planned,
                Some(cmd_check(argv.clone(), None)),
            )],
        );

        run(root, &[57, 58]).unwrap();

        let counter = std::fs::read_to_string(root.join("counter.txt")).unwrap();
        assert_eq!(
            counter.lines().count(),
            1,
            "dedup spans every slice in the invocation (F-2/INV-2)"
        );
    }

    // --- VT-2: the per-entry verdict matrix --------------------------------

    #[test]
    fn exit_zero_with_matcher_hit_is_verified() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        seed(
            root,
            57,
            vec![entry(
                key("SL-057", "REQ-200", "SL-057", "VT"),
                CoverageStatus::Planned,
                Some(cmd_check(
                    vec!["sh", "-c", "printf MARKER"],
                    Some(matcher(Some(MatchSource::Stdout), "MARKER", false)),
                )),
            )],
        );
        run(root, &[57]).unwrap();
        assert_eq!(
            status_of(&reload(root, 57), "REQ-200"),
            CoverageStatus::Verified
        );
    }

    #[test]
    fn nonzero_exit_is_failed() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        seed(
            root,
            57,
            vec![entry(
                key("SL-057", "REQ-200", "SL-057", "VT"),
                CoverageStatus::Planned,
                Some(cmd_check(vec!["false"], None)),
            )],
        );
        run(root, &[57]).unwrap();
        assert_eq!(
            status_of(&reload(root, 57), "REQ-200"),
            CoverageStatus::Failed
        );
    }

    #[test]
    fn matcher_miss_is_failed() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        seed(
            root,
            57,
            vec![entry(
                key("SL-057", "REQ-200", "SL-057", "VT"),
                CoverageStatus::Planned,
                Some(cmd_check(
                    vec!["true"],
                    Some(matcher(Some(MatchSource::Stdout), "ABSENT", false)),
                )),
            )],
        );
        run(root, &[57]).unwrap();
        assert_eq!(
            status_of(&reload(root, 57), "REQ-200"),
            CoverageStatus::Failed,
            "exit 0 but matcher miss ⇒ Failed"
        );
    }

    #[test]
    fn unknown_alias_is_blocked() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // No doctrine.toml ⇒ default config ⇒ the alias is unknown ⇒ Unobtainable.
        let check = VtCheck {
            alias: Some("missing".to_owned()),
            command: None,
            extra_args: Vec::new(),
            matcher: Some(matcher(Some(MatchSource::Stdout), "ok", false)),
        };
        seed(
            root,
            57,
            vec![entry(
                key("SL-057", "REQ-200", "SL-057", "VT"),
                CoverageStatus::Planned,
                Some(check),
            )],
        );
        run(root, &[57]).unwrap();
        assert_eq!(
            status_of(&reload(root, 57), "REQ-200"),
            CoverageStatus::Blocked,
            "unresolvable alias ⇒ Blocked (never run)"
        );
    }

    #[test]
    fn spawn_failure_is_blocked() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        seed(
            root,
            57,
            vec![entry(
                key("SL-057", "REQ-200", "SL-057", "VT"),
                CoverageStatus::Planned,
                Some(cmd_check(vec!["this-binary-does-not-exist-xyzzy"], None)),
            )],
        );
        run(root, &[57]).unwrap();
        assert_eq!(
            status_of(&reload(root, 57), "REQ-200"),
            CoverageStatus::Blocked,
            "a spawn failure ⇒ Blocked, never silent-Failed (F-VII)"
        );
    }

    #[test]
    fn wall_clock_timeout_is_blocked() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        write_doctrine_toml(root, "[verification]\ntimeout-secs = 1\n");
        seed(
            root,
            57,
            vec![entry(
                key("SL-057", "REQ-200", "SL-057", "VT"),
                CoverageStatus::Planned,
                Some(cmd_check(vec!["sleep", "5"], None)),
            )],
        );
        run(root, &[57]).unwrap();
        assert_eq!(
            status_of(&reload(root, 57), "REQ-200"),
            CoverageStatus::Blocked,
            "a wall-clock timeout ⇒ Blocked (F-VII)"
        );
    }

    #[test]
    fn absent_doctrine_toml_default_base_is_blocked_not_green() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // A default-base check (neither alias nor command) with NO doctrine.toml ⇒
        // default config has no `command` ⇒ NoRunnable ⇒ Unobtainable ⇒ Blocked.
        // Never defaulted-green.
        let check = VtCheck {
            alias: None,
            command: None,
            extra_args: Vec::new(),
            matcher: Some(matcher(Some(MatchSource::Stdout), "ok", false)),
        };
        seed(
            root,
            57,
            vec![entry(
                key("SL-057", "REQ-200", "SL-057", "VT"),
                CoverageStatus::Planned,
                Some(check),
            )],
        );
        run(root, &[57]).unwrap();
        assert_eq!(
            status_of(&reload(root, 57), "REQ-200"),
            CoverageStatus::Blocked,
            "absent config + default-base ⇒ Blocked, never green"
        );
    }

    #[test]
    fn stderr_source_matcher_reads_stderr() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        seed(
            root,
            57,
            vec![entry(
                key("SL-057", "REQ-200", "SL-057", "VT"),
                CoverageStatus::Planned,
                Some(cmd_check(
                    vec!["sh", "-c", "printf ERRMARK >&2"],
                    Some(matcher(Some(MatchSource::Stderr), "ERRMARK", false)),
                )),
            )],
        );
        run(root, &[57]).unwrap();
        assert_eq!(
            status_of(&reload(root, 57), "REQ-200"),
            CoverageStatus::Verified,
            "the matcher reads the captured stderr"
        );
    }

    // --- VT-3: git_anchor freshness + cwd + File-glob confinement ----------

    #[test]
    fn ran_entry_restamps_anchor_blocked_keeps_prior() {
        // A git repo so head_sha resolves to a real SHA.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        init_repo(root);
        let head = git::head_sha(root).expect("repo has a HEAD");

        seed(
            root,
            57,
            vec![
                // Ran (Verified) ⇒ anchor re-stamped to head.
                entry(
                    key("SL-057", "REQ-200", "SL-057", "VT"),
                    CoverageStatus::Planned,
                    Some(cmd_check(vec!["true"], None)),
                ),
                // Unresolvable ⇒ Blocked ⇒ anchor KEPT.
                entry(
                    key("SL-057", "REQ-201", "SL-057", "VT"),
                    CoverageStatus::Planned,
                    Some(VtCheck {
                        alias: Some("missing".to_owned()),
                        command: None,
                        extra_args: Vec::new(),
                        matcher: Some(matcher(Some(MatchSource::Stdout), "ok", false)),
                    }),
                ),
            ],
        );
        run(root, &[57]).unwrap();

        let file = reload(root, 57);
        let ran = file
            .entry
            .iter()
            .find(|e| e.key.requirement == "REQ-200")
            .unwrap();
        let blocked = file
            .entry
            .iter()
            .find(|e| e.key.requirement == "REQ-201")
            .unwrap();
        assert_eq!(ran.git_anchor, head, "a Ran observation re-stamps to HEAD");
        assert_eq!(ran.status, CoverageStatus::Verified);
        assert_eq!(
            blocked.git_anchor, "prior-anchor",
            "a Blocked cell KEEPS its prior anchor (F-VIII)"
        );
        assert_eq!(blocked.status, CoverageStatus::Blocked);
    }

    #[test]
    fn command_runs_with_cwd_at_root() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // A root-relative file the command reads — proving cwd == root. `cat`
        // exits 0 and prints MARK iff the file is found relative to cwd.
        std::fs::write(root.join("present.txt"), "MARK").unwrap();
        seed(
            root,
            57,
            vec![entry(
                key("SL-057", "REQ-200", "SL-057", "VT"),
                CoverageStatus::Planned,
                Some(cmd_check(
                    vec!["cat", "present.txt"],
                    Some(matcher(Some(MatchSource::Stdout), "MARK", false)),
                )),
            )],
        );
        run(root, &[57]).unwrap();
        assert_eq!(
            status_of(&reload(root, 57), "REQ-200"),
            CoverageStatus::Verified,
            "the command ran with cwd == root (read a root-relative file)"
        );
    }

    #[test]
    fn file_source_matcher_reads_under_root() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::write(root.join("report.txt"), "PASS here").unwrap();
        seed(
            root,
            57,
            vec![entry(
                key("SL-057", "REQ-200", "SL-057", "VT"),
                CoverageStatus::Planned,
                Some(cmd_check(
                    vec!["true"],
                    Some(matcher(
                        Some(MatchSource::File("report.txt".to_owned())),
                        "PASS",
                        false,
                    )),
                )),
            )],
        );
        run(root, &[57]).unwrap();
        assert_eq!(
            status_of(&reload(root, 57), "REQ-200"),
            CoverageStatus::Verified,
            "a File matcher reads the file under root and matches"
        );
    }

    #[test]
    fn ascending_file_glob_finds_nothing_is_blocked() {
        // A hand-edited `..`-escaping glob: joined under root, it finds nothing ⇒
        // empty match-set ⇒ Unobtainable ⇒ Blocked. Seed a file OUTSIDE root to
        // prove it is NOT reachable.
        let outer = tempfile::tempdir().unwrap();
        std::fs::write(outer.path().join("secret.txt"), "PASS").unwrap();
        let root = outer.path().join("inner");
        std::fs::create_dir_all(&root).unwrap();
        seed(
            &root,
            57,
            vec![entry(
                key("SL-057", "REQ-200", "SL-057", "VT"),
                CoverageStatus::Planned,
                Some(cmd_check(
                    vec!["true"],
                    Some(matcher(
                        Some(MatchSource::File("../secret.txt".to_owned())),
                        "PASS",
                        false,
                    )),
                )),
            )],
        );
        run(&root, &[57]).unwrap();
        assert_eq!(
            status_of(&reload(&root, 57), "REQ-200"),
            CoverageStatus::Blocked,
            "a ..-escaping glob finds nothing under root ⇒ Blocked"
        );
    }

    #[test]
    fn unparseable_regex_matcher_is_blocked() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        seed(
            root,
            57,
            vec![entry(
                key("SL-057", "REQ-200", "SL-057", "VT"),
                CoverageStatus::Planned,
                Some(cmd_check(
                    vec!["true"],
                    // An unparseable regex (hand-edited): `(` ⇒ None ⇒ Unobtainable.
                    Some(matcher(Some(MatchSource::Stdout), "(", true)),
                )),
            )],
        );
        run(root, &[57]).unwrap();
        assert_eq!(
            status_of(&reload(root, 57), "REQ-200"),
            CoverageStatus::Blocked,
            "an unparseable regex ⇒ Blocked, never a silent Failed (F-3)"
        );
    }

    // --- VT-4: Report — exit-code-only flag, backfill count, field preservation ---

    #[test]
    fn report_flags_exit_code_only_and_backfill_untouched() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        seed(
            root,
            57,
            vec![
                // An exit-code-only cell (literal command, no matcher).
                entry(
                    key("SL-057", "REQ-200", "SL-057", "VT"),
                    CoverageStatus::Planned,
                    Some(cmd_check(vec!["true"], None)),
                ),
                // A check-less VT entry: reported in backfill, left untouched.
                entry(
                    key("SL-057", "REQ-201", "SL-057", "VT"),
                    CoverageStatus::InProgress,
                    None,
                ),
            ],
        );
        let report = run(root, &[57]).unwrap();

        assert_eq!(
            report.exit_code_only_count(),
            1,
            "the literal-command cell is flagged"
        );
        let flagged = report
            .verified
            .iter()
            .find(|e| e.key.requirement == "REQ-200")
            .unwrap();
        assert!(flagged.exit_code_only);
        assert_eq!(flagged.old_status, CoverageStatus::Planned);
        assert_eq!(flagged.new_status, CoverageStatus::Verified);

        assert_eq!(
            report.backfill_count(),
            1,
            "the check-less VT is counted for backfill"
        );
        assert_eq!(report.backfill.first().unwrap().key.requirement, "REQ-201");

        // The check-less entry's status is UNTOUCHED on disk.
        let file = reload(root, 57);
        assert_eq!(
            status_of(&file, "REQ-201"),
            CoverageStatus::InProgress,
            "the check-less VT entry is left untouched"
        );
    }

    #[test]
    fn write_back_preserves_key_touched_paths_and_check() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let check = cmd_check(vec!["true"], None);
        seed(
            root,
            57,
            vec![entry(
                key("SL-057", "REQ-200", "SL-057", "VT"),
                CoverageStatus::Planned,
                Some(check.clone()),
            )],
        );
        run(root, &[57]).unwrap();

        let file = reload(root, 57);
        let e = file.entry.first().unwrap();
        assert_eq!(
            e.key,
            key("SL-057", "REQ-200", "SL-057", "VT"),
            "key preserved"
        );
        assert_eq!(
            e.touched_paths,
            vec!["src/x.rs".to_owned()],
            "touched_paths preserved"
        );
        assert_eq!(e.check.as_ref(), Some(&check), "check preserved");
        assert_eq!(e.status, CoverageStatus::Verified, "status re-derived");
    }

    #[test]
    fn non_vt_entries_are_left_untouched() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        seed(
            root,
            57,
            vec![entry(
                // A VH attestation — not a VT entry; the verifier ignores it.
                key("SL-057", "REQ-200", "SL-057", "VH"),
                CoverageStatus::Verified,
                None,
            )],
        );
        let report = run(root, &[57]).unwrap();
        assert!(report.verified.is_empty(), "no VT entry to verify");
        assert!(
            report.backfill.is_empty(),
            "a VH entry is not a backfill candidate"
        );
        assert_eq!(
            status_of(&reload(root, 57), "REQ-200"),
            CoverageStatus::Verified,
            "the VH entry's status is untouched"
        );
    }

    // --- VT-5 (NF-001 / INV-1): run() touches ONLY coverage.toml -------------

    #[test]
    fn run_mutates_only_coverage_toml_not_a_sibling_entity_file() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // A sibling requirement entity file in the doctrine tree.
        let req_dir = root.join(".doctrine/requirement/200");
        std::fs::create_dir_all(&req_dir).unwrap();
        let req_file = req_dir.join("requirement-200.toml");
        let req_body =
            "id = 200\ntitle = \"x\"\nslug = \"x\"\nstatus = \"active\"\nkind = \"functional\"\n";
        std::fs::write(&req_file, req_body).unwrap();

        // Drive run() end-to-end over varied outcomes (Verified + Failed + Blocked).
        seed(
            root,
            57,
            vec![
                entry(
                    key("SL-057", "REQ-200", "SL-057", "VT"),
                    CoverageStatus::Planned,
                    Some(cmd_check(vec!["true"], None)),
                ),
                entry(
                    key("SL-057", "REQ-201", "SL-057", "VT"),
                    CoverageStatus::Planned,
                    Some(cmd_check(vec!["false"], None)),
                ),
                entry(
                    key("SL-057", "REQ-202", "SL-057", "VT"),
                    CoverageStatus::Planned,
                    Some(VtCheck {
                        alias: Some("missing".to_owned()),
                        command: None,
                        extra_args: Vec::new(),
                        matcher: Some(matcher(Some(MatchSource::Stdout), "ok", false)),
                    }),
                ),
            ],
        );

        run(root, &[57]).unwrap();

        // The requirement entity file is BYTE-UNCHANGED.
        assert_eq!(
            std::fs::read_to_string(&req_file).unwrap(),
            req_body,
            "run() drove the write seam and left the requirement entity byte-identical (NF-001/INV-1)"
        );
        // And coverage.toml DID change (the three statuses re-derived).
        let file = reload(root, 57);
        assert_eq!(status_of(&file, "REQ-200"), CoverageStatus::Verified);
        assert_eq!(status_of(&file, "REQ-201"), CoverageStatus::Failed);
        assert_eq!(status_of(&file, "REQ-202"), CoverageStatus::Blocked);
    }

    // --- helpers: a minimal real git repo so head_sha resolves ---------------

    fn init_repo(root: &Path) {
        let run_git = |args: &[&str]| {
            std::process::Command::new("git")
                .arg("-C")
                .arg(root)
                .args(args)
                .output()
                .unwrap();
        };
        run_git(&["init", "-q"]);
        run_git(&["config", "user.email", "t@t.t"]);
        run_git(&["config", "user.name", "t"]);
        std::fs::write(root.join("seed"), "x").unwrap();
        run_git(&["add", "-A"]);
        run_git(&["commit", "-q", "-m", "seed"]);
    }
}
