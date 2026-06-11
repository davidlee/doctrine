// SPDX-License-Identifier: GPL-3.0-only
//! `coverage_scan` — the SL-042 P3 impure reconcile-reader shell.
//!
//! This is the ONLY git/disk seam in the coverage data flow (CLAUDE.md
//! pure/imperative split; design §5.2). It corpus-walks every slice's
//! `coverage.toml`, filters to one requirement, resolves each surviving entry's
//! staleness against git ONCE per scan, and hands the pure folds
//! ([`crate::coverage::composite`] / [`crate::coverage::drift`]) in-memory
//! `(CoverageEntry, IsStale)` cells. The folds stay pure — staleness arrives
//! already resolved here, never inside a fold.
//!
//! Degradations are total, never fatal: a missing slice tree, an unreadable or
//! malformed `coverage.toml`, or an unborn HEAD all narrow the result rather than
//! erroring — a single bad file or a fresh repo must not abort a reconcile read.

// The shell is a leaf built ahead of its consumer: P3 lands the scan; the CLI
// reconcile reader that calls it is a future slice. Until then `scan_coverage`
// is dead in the bins/lib build, so the module carries the self-clearing
// `not(test)` dead_code expect (the `dead-code-self-clearing-leaf` precedent).
// Under `cfg(test)` the perf-spike/integration tests exercise it, so the lint
// would not fire there; scoping to `not(test)` fulfils the expectation exactly
// where the lint applies. Retires itself when the reader is wired.
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SL-042 P3 reconcile-reader shell is a leaf built ahead of its \
                  CLI-reader consumer — scan_coverage is dead in the bins/lib \
                  build until that reader is wired (future slice)"
    )
)]

use std::fs;
use std::path::Path;

use crate::coverage::{self, CoverageEntry, IsStale};
use crate::git;

/// Repo-relative slice tree — the dir whose `*/coverage.toml` files this shell
/// walks. Mirrors `state::SLICE_DIR`; kept local so the shell owns its one path.
const SLICE_DIR: &str = ".doctrine/slice";

/// Walk every slice's `coverage.toml`, keep the entries citing `req`, and resolve
/// each one's [`IsStale`] against git — the in-memory cells the pure folds consume.
///
/// Data flow (design §5.2): corpus-walk → parse → filter by requirement →
/// resolve `HEAD` ONCE → per-entry `commits_touching(anchor..HEAD over
/// touched_paths)`. An unborn/non-repo HEAD makes every cell [`IsStale::Unknown`].
/// A missing slice tree or any unreadable/malformed file is skipped, never fatal.
pub(crate) fn scan_coverage(root: &Path, req: &str) -> Vec<(CoverageEntry, IsStale)> {
    let matched = collect_matching_entries(root, req);

    // Resolve HEAD ONCE for the whole scan (the single git anchor for staleness).
    // None ⇒ unborn / non-repo / git failure ⇒ every cell is Unknown.
    let head = git::head_sha(root);

    matched
        .into_iter()
        .map(|entry| {
            let stale = match head.as_deref() {
                Some(head) => IsStale::from(git::commits_touching(
                    root,
                    &entry.touched_paths,
                    &entry.git_anchor,
                    head,
                )),
                None => IsStale::Unknown,
            };
            (entry, stale)
        })
        .collect()
}

/// The disk half: corpus-walk `<root>/.doctrine/slice/*/coverage.toml`, parse
/// each, and keep entries whose key requirement matches `req`. Missing dir →
/// empty; an unreadable or malformed file is skipped (degradation, not error).
fn collect_matching_entries(root: &Path, req: &str) -> Vec<CoverageEntry> {
    let slice_root = root.join(SLICE_DIR);
    let Ok(slices) = fs::read_dir(&slice_root) else {
        return Vec::new(); // absent / unreadable tree → empty
    };

    let mut out = Vec::new();
    for slice in slices.flatten() {
        let coverage_path = slice.path().join("coverage.toml");
        let Ok(body) = fs::read_to_string(&coverage_path) else {
            continue; // no coverage.toml in this slice (or unreadable) → skip
        };
        let Ok(file) = coverage::parse(&body) else {
            continue; // malformed coverage.toml → skip, never abort the scan
        };
        out.extend(file.entry.into_iter().filter(|e| e.key.requirement == req));
    }
    out
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "tests: fail-fast unwrap on disk/git setup is idiomatic"
)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::time::Instant;

    // --- helpers -------------------------------------------------------------

    /// Write one slice's `coverage.toml` under a project root.
    fn write_coverage(root: &Path, slice_num: u32, body: &str) {
        let dir = root.join(SLICE_DIR).join(format!("{slice_num:03}"));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("coverage.toml"), body).unwrap();
    }

    /// A minimal `[[entry]]` body for one requirement.
    fn one_entry_body(slice: &str, req: &str, status: &str) -> String {
        format!(
            "[[entry]]\n\
             slice = \"{slice}\"\n\
             requirement = \"{req}\"\n\
             contributing_change = \"{slice}\"\n\
             mode = \"VT\"\n\
             status = \"{status}\"\n\
             git_anchor = \"deadbeef\"\n"
        )
    }

    // --- behaviour: filter + degradations ------------------------------------

    #[test]
    fn missing_slice_tree_yields_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert!(scan_coverage(dir.path(), "REQ-110").is_empty());
    }

    #[test]
    fn filters_to_the_requested_requirement_across_slices() {
        let dir = tempfile::tempdir().unwrap();
        write_coverage(
            dir.path(),
            40,
            &one_entry_body("SL-040", "REQ-110", "verified"),
        );
        write_coverage(
            dir.path(),
            41,
            &one_entry_body("SL-041", "REQ-999", "planned"),
        );
        write_coverage(
            dir.path(),
            42,
            &one_entry_body("SL-042", "REQ-110", "planned"),
        );

        let cells = scan_coverage(dir.path(), "REQ-110");
        assert_eq!(
            cells.len(),
            2,
            "only the two REQ-110 entries survive the filter"
        );
        assert!(cells.iter().all(|(e, _)| e.key.requirement == "REQ-110"));
    }

    #[test]
    fn malformed_coverage_file_is_skipped_not_fatal() {
        let dir = tempfile::tempdir().unwrap();
        write_coverage(
            dir.path(),
            40,
            &one_entry_body("SL-040", "REQ-110", "verified"),
        );
        write_coverage(dir.path(), 41, "this is not valid toml = = =");
        let cells = scan_coverage(dir.path(), "REQ-110");
        assert_eq!(
            cells.len(),
            1,
            "the good file survives; the bad one is skipped"
        );
    }

    #[test]
    fn no_head_makes_every_cell_unknown() {
        // A tempdir that is NOT a git repo → head_sha None → all Unknown.
        let dir = tempfile::tempdir().unwrap();
        write_coverage(
            dir.path(),
            40,
            &one_entry_body("SL-040", "REQ-110", "verified"),
        );
        let cells = scan_coverage(dir.path(), "REQ-110");
        assert_eq!(cells.len(), 1);
        assert_eq!(cells.first().unwrap().1, IsStale::Unknown);
    }

    // --- VT-4 (R2 perf spike) — TWO axes, measured separately ----------------
    //
    // Axis (a): scan fan-in (walk+parse+filter), IsStale precomputed Unknown
    // (no git). Sweep N ∈ {50, 500, 2000}; the 2000 tier is #[ignore]d so it
    // never bloats the default gate — run it explicitly to confirm the cliff.
    // Axis (b): per-call git::commits_touching subprocess cost against the REAL
    // fork repo with real paths. N calls, per-call cost — NO fabricated commits.
    //
    // Bounds are DEBUG-budgeted (~10× release; mem.pattern.testing.debug-vs-
    // release-scale-timing): generous, cliff-detecting, not tight absolutes.

    /// Axis (a): build N slice coverage files, then measure ONLY walk+parse+filter.
    fn measure_scan_fanin(n: u32) -> std::time::Duration {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..n {
            // Half the files carry the target requirement, half a decoy — so the
            // filter does real work.
            let req = if i % 2 == 0 { "REQ-110" } else { "REQ-999" };
            write_coverage(dir.path(), i, &one_entry_body("SL-000", req, "planned"));
        }
        let start = Instant::now();
        let cells = scan_coverage(dir.path(), "REQ-110");
        let elapsed = start.elapsed();
        // Sanity: half matched (the non-repo tempdir resolves all to Unknown,
        // which is fine — axis (a) is the walk cost, not the git cost).
        let expected = n.div_euclid(2) + n.rem_euclid(2);
        assert_eq!(cells.len() as u32, expected, "filter kept the REQ-110 half");
        elapsed
    }

    #[test]
    fn vt4a_scan_fanin_small_tiers() {
        // Default-gate tiers: cheap, always run. Print per-tier timing.
        for n in [50_u32, 500] {
            let d = measure_scan_fanin(n);
            println!("VT-4(a) scan fan-in N={n}: {d:?}");
            // Loose debug ceiling — flags a pathological regression, not a cliff.
            assert!(
                d.as_secs() < 10,
                "scan fan-in N={n} took {d:?} — investigate (debug budget ~10x)"
            );
        }
    }

    #[test]
    #[ignore = "heavy 2000-file tier — run explicitly to confirm the scan cliff; \
                numbers recorded in the worker report"]
    fn vt4a_scan_fanin_heavy_tier() {
        let d = measure_scan_fanin(2000);
        println!("VT-4(a) scan fan-in N=2000: {d:?}");
        assert!(d.as_secs() < 30, "scan fan-in N=2000 took {d:?}");
    }

    /// Axis (b): per-call `git::commits_touching` subprocess cost against the
    /// real fork repo. Returns (total, per_call). Skips gracefully if the fork is
    /// not a usable git repo (e.g. CI without history).
    fn measure_staleness_per_call(n: u32) -> Option<(std::time::Duration, std::time::Duration)> {
        // The fork repo: CARGO_MANIFEST_DIR is the crate root = the worktree.
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let head = git::head_sha(root)?;
        // An old SHA reachable from HEAD: first commit on the branch.
        let out = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["rev-list", "--max-parents=0", "HEAD"])
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let roots = String::from_utf8(out.stdout).ok()?;
        let base = roots.lines().next()?.trim().to_owned();
        if base.is_empty() {
            return None;
        }
        let paths = vec!["src/coverage.rs".to_owned()];

        let start = Instant::now();
        for _ in 0..n {
            // Real subprocess each call — this is the cost we are measuring.
            let _ = git::commits_touching(root, &paths, &base, &head);
        }
        let total = start.elapsed();
        let per = total.checked_div(n).unwrap_or(total);
        Some((total, per))
    }

    #[test]
    fn vt4b_staleness_per_call_cost() {
        // Modest N so the gate stays fast; per-call cost is the signal, not total.
        let Some((total, per)) = measure_staleness_per_call(20) else {
            println!("VT-4(b) staleness: fork not a usable git repo — skipped");
            return;
        };
        println!("VT-4(b) staleness N=20: total {total:?}, per-call {per:?}");
        // A git subprocess pair (merge-base + rev-list) is single-digit ms to
        // low tens of ms; flag only a pathological per-call cost.
        assert!(
            per.as_millis() < 2000,
            "per-call staleness {per:?} — investigate subprocess cost"
        );
    }
}
