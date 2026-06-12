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

// SL-044 B·P2 wired the consumer: `crate::reconcile::run` calls `scan_coverage`,
// so the shell is live in the bins/lib build — the self-clearing `not(test)`
// dead_code expect this module carried has retired itself, as its reason foretold.

use std::collections::{BTreeMap, BTreeSet};
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
    // Q1 DELEGATE (SL-045 PHASE-01, design §5.2): the single-req path rides the
    // batched walker — one shared corpus walk, no parallel impl (F3). The
    // single-elem wanted-set yields a one-key dense map; the bucket accumulates in
    // the identical read_dir order, so the result is byte-identical to the former
    // dedicated walk. The existing single-req suite (unchanged) is that proof.
    scan_coverage_batch(root, &BTreeSet::from([req.to_owned()]))
        .remove(req)
        .unwrap_or_default()
}

/// Walk every slice's `coverage.toml` ONCE and fan the entries into a DENSE
/// per-requirement bucketing over `wanted` — every wanted req keyed (empty `Vec`
/// if uncovered), each matched cell's [`IsStale`] resolved against a single
/// `HEAD`. The batched generalization of [`scan_coverage`] (design §5.2, D4):
/// it bounds a spec's whole requirement fan to ONE walk + ONE git anchor (INV-2,
/// RSK-006). Same degradations as the single-req path — a missing tree, an
/// unreadable or malformed file, or an unborn HEAD narrows the result, never
/// errors.
pub(crate) fn scan_coverage_batch(
    root: &Path,
    wanted: &BTreeSet<String>,
) -> BTreeMap<String, Vec<(CoverageEntry, IsStale)>> {
    let buckets = collect_matching_entries_batch(root, wanted);

    // Resolve HEAD ONCE for the whole batch (the single git anchor for the entire
    // requirement fan — INV-2). None ⇒ unborn / non-repo / git failure ⇒ Unknown.
    let head = git::head_sha(root);

    buckets
        .into_iter()
        .map(|(req, entries)| (req, stale_each(root, head.as_deref(), entries)))
        .collect()
}

/// Resolve each entry's [`IsStale`] against an already-resolved `head` (`None` ⇒
/// every cell [`IsStale::Unknown`]). Lifted from the former per-cell closure so the
/// single- and batched-scan paths share one staleness mapping (F3) — staleness is
/// resolved here, in the shell, never inside a pure fold.
fn stale_each(
    root: &Path,
    head: Option<&str>,
    entries: Vec<CoverageEntry>,
) -> Vec<(CoverageEntry, IsStale)> {
    entries
        .into_iter()
        .map(|entry| {
            let stale = match head {
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

/// The DISTINCT requirements physically covered by ONE slice's OWN
/// `.doctrine/slice/<NNN>/coverage.toml` (R-B4, the closure-gate `covered` term).
///
/// Distinct from [`scan_coverage`], which is per-requirement ACROSS all slices and
/// never enumerates a single slice's reqs — this reads exactly that one file and
/// collects the distinct `entry.key.requirement` (first-seen order; ISS-006's
/// slug-symlink double-walk does not apply — we read one concrete path, not a
/// dir-walk). A missing/empty `coverage.toml` ⇒ `covered = ∅` (NOT an error — a
/// slice may close having authored/reconciled reqs but covered none).
///
/// **Integrity (codex finding 6):** every entry's `key.slice` MUST equal `canonical`
/// (`"SL-<NNN>"`) — a FOREIGN `slice =` in S's own coverage.toml is an authoring
/// error, REFUSED here rather than silently swept in-or-out of the gate set.
pub(crate) fn slice_local_covered_reqs(
    root: &Path,
    slice_id: u32,
    canonical: &str,
) -> anyhow::Result<Vec<String>> {
    let path = root
        .join(SLICE_DIR)
        .join(format!("{slice_id:03}"))
        .join("coverage.toml");
    let body = match fs::read_to_string(&path) {
        Ok(b) => b,
        // Absent ⇒ covered = ∅ (not an error).
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => {
            return Err(e).map_err(|e| anyhow::anyhow!("failed to read {}: {e}", path.display()));
        }
    };
    let file = coverage::parse(&body)
        .map_err(|e| anyhow::anyhow!("failed to parse {}: {e}", path.display()))?;
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for entry in file.entry {
        anyhow::ensure!(
            entry.key.slice == canonical,
            "integrity error: {} carries a foreign coverage entry slice = \"{}\" \
             (expected \"{canonical}\") for requirement {} — a slice's own \
             coverage.toml must cite only itself",
            path.display(),
            entry.key.slice,
            entry.key.requirement,
        );
        if seen.insert(entry.key.requirement.clone()) {
            out.push(entry.key.requirement);
        }
    }
    Ok(out)
}

/// The disk half: corpus-walk `<root>/.doctrine/slice/*/coverage.toml` ONCE and
/// fan every entry whose key requirement ∈ `wanted` into a DENSE bucketing —
/// pre-seeded so every wanted req is present (empty `Vec` if uncovered). Missing
/// dir → the dense-empty seed; an unreadable or malformed file is skipped
/// (degradation, not error). The membership test is the dense map's own
/// `get_mut`, so an unwanted req's entries are dropped in one lookup.
fn collect_matching_entries_batch(
    root: &Path,
    wanted: &BTreeSet<String>,
) -> BTreeMap<String, Vec<CoverageEntry>> {
    // Dense seed: every wanted req keyed up front, so callers always find a row.
    let mut out: BTreeMap<String, Vec<CoverageEntry>> =
        wanted.iter().map(|r| (r.clone(), Vec::new())).collect();

    let slice_root = root.join(SLICE_DIR);
    let Ok(slices) = fs::read_dir(&slice_root) else {
        return out; // absent / unreadable tree → dense-empty
    };

    for slice in slices.flatten() {
        // Skip the slug-alias symlink (`NNN-slug -> NNN`): it re-walks the same
        // coverage.toml the numeric dir already yielded, double-counting every
        // entry (ISS-006). The numeric canonical dir is never a symlink.
        if slice.file_type().is_ok_and(|t| t.is_symlink()) {
            continue;
        }
        let coverage_path = slice.path().join("coverage.toml");
        let Ok(body) = fs::read_to_string(&coverage_path) else {
            continue; // no coverage.toml in this slice (or unreadable) → skip
        };
        let Ok(file) = coverage::parse(&body) else {
            continue; // malformed coverage.toml → skip, never abort the scan
        };
        for entry in file.entry {
            // Keep only wanted reqs; the dense seed's get_mut IS the filter.
            if let Some(bucket) = out.get_mut(&entry.key.requirement) {
                bucket.push(entry);
            }
        }
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
    fn slug_alias_symlink_does_not_double_count() {
        // ISS-006: doctrine seats a `NNN-slug -> NNN` symlink beside every numeric
        // slice dir. The corpus walk must NOT re-read the aliased coverage.toml, or
        // each entry is yielded twice.
        let dir = tempfile::tempdir().unwrap();
        write_coverage(
            dir.path(),
            42,
            &one_entry_body("SL-042", "REQ-110", "planned"),
        );
        let slice_root = dir.path().join(SLICE_DIR);
        std::os::unix::fs::symlink(
            slice_root.join("042"),
            slice_root.join("042-reconciliation-observe-substrate"),
        )
        .unwrap();

        let cells = scan_coverage(dir.path(), "REQ-110");
        assert_eq!(
            cells.len(),
            1,
            "the slug-alias symlink must not re-yield the same coverage entry (ISS-006)"
        );
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

    // --- PHASE-04 temp-git-repo helper (R-e seam-fit) ------------------------
    //
    // A throwaway born git repo (NEVER the doctrine repo's own .doctrine/): init,
    // pin identity, write+commit files. Mirrors git.rs's `ScratchRepo` shape; kept
    // local because that helper is private to git.rs's test module.

    /// Run `git -C <root> <args>` with pinned identity (no machine config needed),
    /// asserting success; returns trimmed stdout.
    fn git_at(root: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .arg("-C")
            .arg(root)
            .args([
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
        String::from_utf8(out.stdout).unwrap().trim().to_owned()
    }

    /// Write `rel` under `root` (creating parents), stage and commit it, return the
    /// resulting HEAD SHA.
    fn write_commit(root: &Path, rel: &str, contents: &str, msg: &str) -> String {
        let full = root.join(rel);
        fs::create_dir_all(full.parent().unwrap()).unwrap();
        fs::write(&full, contents).unwrap();
        git_at(root, &["add", rel]);
        git_at(root, &["commit", "-q", "-m", msg]);
        git_at(root, &["rev-parse", "HEAD"])
    }

    // --- T1 (R-e): the staleness seam fits a coverage entry's own granularity --
    //
    // H1 ("git::commits_touching fits coverage's (git_anchor, touched_paths)
    // granularity") turned from hypothesis into a test-backed fact: drive the seam
    // with the EXACT field types a CoverageEntry carries (a String anchor, a
    // Vec<String> of repo-relative paths) against a real temp git repo. No leaf
    // widening was needed — the existing signature consumed coverage's granularity
    // verbatim.

    #[test]
    fn seam_fits_coverage_entry_granularity_stale_and_fresh() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        git_at(root, &["init", "-q", "-b", "main"]);

        // A coverage entry's anchor is the SHA after the path was first seen.
        let anchor = write_commit(root, "src/foo.rs", "fn a() {}\n", "add foo");
        // A second, unrelated path moves HEAD without touching foo.rs.
        let _ = write_commit(root, "src/bar.rs", "fn b() {}\n", "add bar");
        let head_fresh = git_at(root, &["rev-parse", "HEAD"]);

        // Use a CoverageEntry's ACTUAL field shapes as the seam inputs.
        let entry = entry_with(&anchor, &["src/foo.rs"]);
        // foo.rs was NOT touched between anchor and head_fresh → Fresh.
        let fresh = IsStale::from(git::commits_touching(
            root,
            &entry.touched_paths,
            &entry.git_anchor,
            &head_fresh,
        ));
        assert_eq!(
            fresh,
            IsStale::Fresh,
            "anchor..HEAD over an untouched path resolves Fresh through the seam"
        );

        // Now modify foo.rs and commit — HEAD moves PAST the anchor over that path.
        let head_stale = write_commit(root, "src/foo.rs", "fn a() { 1; }\n", "edit foo");
        let stale = IsStale::from(git::commits_touching(
            root,
            &entry.touched_paths,
            &entry.git_anchor,
            &head_stale,
        ));
        assert_eq!(
            stale,
            IsStale::Stale,
            "a commit touching the path since the anchor resolves Stale through the seam"
        );
    }

    /// Build a `CoverageEntry` carrying the given anchor + touched paths (the two
    /// fields the staleness seam consumes), Verified VH evidence.
    fn entry_with(anchor: &str, paths: &[&str]) -> CoverageEntry {
        use crate::requirement::CoverageStatus;
        CoverageEntry {
            key: coverage::CoverageKey {
                slice: "SL-042".to_owned(),
                requirement: "REQ-115".to_owned(),
                contributing_change: "SL-042".to_owned(),
                mode: "VH".to_owned(),
            },
            status: CoverageStatus::Verified,
            git_anchor: anchor.to_owned(),
            attested_date: Some("2026-06-12".to_owned()),
            touched_paths: paths.iter().map(|p| (*p).to_owned()).collect(),
        }
    }

    // --- T2 (VT-1 / NF-002): VH/VA Verified evidence is FLAGGED stale, never ---
    //     auto-demoted. The core decay lock, end-to-end through scan_coverage.
    //
    // Layout a temp git repo with a committed source file (the anchor), a slice
    // coverage.toml carrying a VH and a VA Verified entry over that file, then move
    // HEAD past the anchor by editing the file. scan_coverage must mark both cells
    // Stale while their `status` stays Verified — staleness is a SEPARATE axis from
    // the observed status; nothing demotes Verified to Failed/Blocked/etc.

    /// Render a two-entry (VH + VA) coverage.toml body, both `Verified`, anchored at
    /// `anchor` over `path`. The mode is the only field that differs between them.
    fn vh_va_coverage_body(anchor: &str, path: &str) -> String {
        let entry = |mode: &str| {
            format!(
                "[[entry]]\n\
                 slice = \"SL-042\"\n\
                 requirement = \"REQ-115\"\n\
                 contributing_change = \"SL-042\"\n\
                 mode = \"{mode}\"\n\
                 status = \"verified\"\n\
                 git_anchor = \"{anchor}\"\n\
                 attested_date = \"2026-06-12\"\n\
                 touched_paths = [\"{path}\"]\n"
            )
        };
        format!("{}{}", entry("VH"), entry("VA"))
    }

    #[test]
    fn vh_va_verified_evidence_is_flagged_stale_never_demoted() {
        use crate::requirement::CoverageStatus;

        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        git_at(root, &["init", "-q", "-b", "main"]);

        // 1. A committed source file → its commit SHA is the coverage anchor.
        let anchor = write_commit(root, "src/foo.rs", "fn a() {}\n", "add foo");

        // 2. Lay the VH + VA Verified coverage entries over src/foo.rs at `anchor`,
        //    and commit them INSIDE the temp repo so the temp repo's own HEAD is
        //    valid for head_sha / commits_touching (NOT the doctrine repo's HEAD).
        let cov_rel = ".doctrine/slice/042/coverage.toml";
        write_commit(
            root,
            cov_rel,
            &vh_va_coverage_body(&anchor, "src/foo.rs"),
            "coverage",
        );

        // 3. Move HEAD PAST the anchor over src/foo.rs (edit + commit). `anchor` is
        //    now a strict ancestor of HEAD (the merge-base gate passes) and a commit
        //    has touched the path since.
        write_commit(root, "src/foo.rs", "fn a() { 1; }\n", "edit foo");

        // 4. Scan. Both VH and VA cells must be Stale, status still Verified.
        let cells = scan_coverage(root, "REQ-115");
        assert_eq!(
            cells.len(),
            2,
            "the VH and VA entries both survive the filter"
        );

        for (entry, stale) in &cells {
            assert_eq!(
                *stale,
                IsStale::Stale,
                "{} evidence over an edited path is flagged stale",
                entry.key.mode
            );
            assert_eq!(
                entry.status,
                CoverageStatus::Verified,
                "{} status stays Verified — staleness NEVER auto-demotes (NF-002)",
                entry.key.mode
            );
        }
        // Spell the mode coverage out: both attestation kinds are present.
        assert!(cells.iter().any(|(e, _)| e.key.mode == "VH"));
        assert!(cells.iter().any(|(e, _)| e.key.mode == "VA"));
    }

    #[test]
    fn vh_va_verified_evidence_untouched_since_anchor_is_fresh() {
        use crate::requirement::CoverageStatus;

        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        git_at(root, &["init", "-q", "-b", "main"]);

        // The anchor is the SHA at which foo.rs was last touched.
        let anchor = write_commit(root, "src/foo.rs", "fn a() {}\n", "add foo");
        // Commit the coverage store and an UNRELATED file — HEAD advances but never
        // touches src/foo.rs after the anchor.
        write_commit(
            root,
            ".doctrine/slice/042/coverage.toml",
            &vh_va_coverage_body(&anchor, "src/foo.rs"),
            "coverage",
        );
        write_commit(root, "src/bar.rs", "fn b() {}\n", "add bar");

        let cells = scan_coverage(root, "REQ-115");
        assert_eq!(cells.len(), 2);
        for (entry, stale) in &cells {
            assert_eq!(
                *stale,
                IsStale::Fresh,
                "{} evidence over an untouched path is Fresh — the contrast case",
                entry.key.mode
            );
            assert_eq!(entry.status, CoverageStatus::Verified, "status unchanged");
        }
    }

    // --- T3 (VT-2 / EX-3): no parallel staleness impl ------------------------
    //
    // Coverage staleness flows ONLY through git::commits_touching — there is no
    // second staleness leaf. Structurally assert that the coverage modules
    // reference the seam and carry NO code path into the memory-side staleness leaf
    // (the cs leaf below — a DISTINCT, unrelated module that must not bleed in
    // here). We match the `::`-path FORM of that module, never the bare word, so
    // prose may name it without tripping the guard.

    /// The memory-side staleness leaf's module path form. Spelled by concatenation
    /// so this very assertion's source carries no literal `<name>::` token to
    /// false-positive on (the guard reads its own file).
    fn rival_staleness_path() -> String {
        format!("{}::", "contentset")
    }

    #[test]
    fn coverage_staleness_flows_only_through_commits_touching() {
        let scan_src = include_str!("coverage_scan.rs");
        let cov_src = include_str!("coverage.rs");
        let rival = rival_staleness_path();

        assert!(
            scan_src.contains("commits_touching"),
            "the scan shell resolves staleness through git::commits_touching"
        );
        // The single staleness seam — no parallel impl, no path into the rival leaf.
        for (name, src) in [("coverage_scan.rs", scan_src), ("coverage.rs", cov_src)] {
            assert!(
                !src.contains(&rival),
                "{name} must not path into the memory-side staleness leaf — coverage \
                 staleness has its own single seam (git::commits_touching), no \
                 parallel impl"
            );
        }
    }

    // --- SL-045 PHASE-01: batched corpus scanner (EX-1/2/3, VT-1/2/3) --------
    //
    // `scan_coverage_batch(root, &wanted)` does ONE corpus walk and returns a
    // DENSE per-requirement bucketing: every wanted req keyed (empty Vec if
    // uncovered). Equivalence is pinned at the `composite` seam (E4) — read_dir
    // order is incidental, so we never assert raw Vec order.

    use std::collections::BTreeSet;

    /// The wanted-set as the shell consumes it.
    fn wanted(reqs: &[&str]) -> BTreeSet<String> {
        reqs.iter().map(|r| (*r).to_owned()).collect()
    }

    /// VT-1 — partition + per-req equivalence at the composite seam + density.
    /// Fixture: ≥3 slices, ≥2 reqs, REQ-110 covered in TWO slices (40, 42).
    #[test]
    fn batch_partitions_each_req_and_matches_single_scan_at_composite_seam() {
        let dir = tempfile::tempdir().unwrap();
        write_coverage(
            dir.path(),
            40,
            &one_entry_body("SL-040", "REQ-110", "verified"),
        );
        write_coverage(
            dir.path(),
            41,
            &one_entry_body("SL-041", "REQ-200", "planned"),
        );
        write_coverage(
            dir.path(),
            42,
            &one_entry_body("SL-042", "REQ-110", "planned"),
        );
        write_coverage(
            dir.path(),
            43,
            &one_entry_body("SL-043", "REQ-999", "planned"),
        );

        let batch = scan_coverage_batch(dir.path(), &wanted(&["REQ-110", "REQ-200", "REQ-300"]));

        // Density: every wanted req is a key; REQ-300 (uncovered) present as [].
        assert_eq!(
            batch.keys().cloned().collect::<Vec<_>>(),
            vec![
                "REQ-110".to_owned(),
                "REQ-200".to_owned(),
                "REQ-300".to_owned()
            ],
            "dense: exactly the wanted reqs are keyed"
        );
        assert!(
            batch["REQ-300"].is_empty(),
            "uncovered wanted req → empty bucket"
        );

        // Partition: each bucket holds exactly its req's entries (no leakage of
        // the unwanted REQ-999 decoy).
        assert_eq!(batch["REQ-110"].len(), 2, "REQ-110 covered in two slices");
        assert_eq!(batch["REQ-200"].len(), 1);
        for (req, bucket) in &batch {
            assert!(
                bucket.iter().all(|(e, _)| &e.key.requirement == req),
                "bucket {req} holds only its own req's entries"
            );
        }

        // Equivalence at the composite seam: batching N reqs gives the same
        // per-req composite as scanning that req alone. NOT raw Vec order (E4).
        for req in ["REQ-110", "REQ-200"] {
            assert_eq!(
                coverage::composite(&batch[req]),
                coverage::composite(&scan_coverage(dir.path(), req)),
                "composite(batch[{req}]) equals composite(scan_coverage({req}))"
            );
        }
    }

    /// VT-3 — the slug-alias symlink must not double-count through the batch walk.
    #[test]
    fn batch_slug_alias_symlink_does_not_double_count() {
        let dir = tempfile::tempdir().unwrap();
        write_coverage(
            dir.path(),
            42,
            &one_entry_body("SL-042", "REQ-110", "planned"),
        );
        let slice_root = dir.path().join(SLICE_DIR);
        std::os::unix::fs::symlink(
            slice_root.join("042"),
            slice_root.join("042-reconciliation-observe-substrate"),
        )
        .unwrap();

        let batch = scan_coverage_batch(dir.path(), &wanted(&["REQ-110"]));
        assert_eq!(
            batch["REQ-110"].len(),
            1,
            "the slug-alias symlink must not re-yield the entry through the batch (ISS-006)"
        );
    }

    /// VT-2 — determinism (via composite, across two runs) + degradation
    /// tolerance: a malformed file is skipped, a missing tree yields an
    /// all-empty DENSE map (every wanted key present with []).
    #[test]
    fn batch_is_deterministic_and_degradation_tolerant() {
        let dir = tempfile::tempdir().unwrap();
        write_coverage(
            dir.path(),
            40,
            &one_entry_body("SL-040", "REQ-110", "verified"),
        );
        write_coverage(dir.path(), 41, "this is not valid toml = = =");

        let w = wanted(&["REQ-110", "REQ-200"]);
        let a = scan_coverage_batch(dir.path(), &w);
        let b = scan_coverage_batch(dir.path(), &w);
        for req in ["REQ-110", "REQ-200"] {
            assert_eq!(
                coverage::composite(&a[req]),
                coverage::composite(&b[req]),
                "composite(batch[{req}]) is stable across runs"
            );
        }
        // Malformed slice 41 skipped; the good REQ-110 entry survives.
        assert_eq!(
            a["REQ-110"].len(),
            1,
            "malformed file skipped, good one kept"
        );
        assert!(a["REQ-200"].is_empty());

        // Missing tree → all-empty dense map.
        let empty_dir = tempfile::tempdir().unwrap();
        let m = scan_coverage_batch(empty_dir.path(), &w);
        assert_eq!(
            m.keys().cloned().collect::<Vec<_>>(),
            vec!["REQ-110".to_owned(), "REQ-200".to_owned()],
            "missing tree still yields a dense map over the wanted set"
        );
        assert!(
            m.values().all(Vec::is_empty),
            "every bucket empty on a missing tree"
        );
    }
}
