// SPDX-License-Identifier: GPL-3.0-only
//! The reviewer-context warm cache and `review prime` (SL-268 PHASE-02 T5).

use super::turn::{LockGuard, resolve_review_root, state_dir};
use super::{
    BTreeMap, ContentSet, Context, Deserialize, Path, PathBuf, ReviewOutput, Serialize,
    canonical_id, contentset, fs, io, parse_ref, read_authored,
};

// ===========================================================================
// PHASE-05 — the reviewer-context warm-cache (`cache.toml`) + `prime` (design §9,
// D9, D-C10). The cache is the reviewer's *learned* model — runtime, regenerable,
// never authored, DECOUPLED from any LLM token cache (T-b: doctrine makes no
// attempt to observe token-cache warmth). It lives beside the baton/lock in the
// parent tree's gitignored state.
//
// Shape (§9, SL-147 PHASE-05): the resolved `paths` fileset (the target slice's
// selectors expanded against the tracked file set) and a `[hashes]` table = the
// `ContentSet` over those paths — the staleness baseline. Staleness is the pure
// `stored.diff(compute(parent_root, paths))` (T-b naming: `current` vs `stale`);
// it is an optimization SIGNAL surfaced by `status`, never a gate. Single parent
// root; absence⇒stale (R1, in `contentset`).
// ===========================================================================

/// The warm-cache document — `cache.toml` (§9). `paths` is the resolved fileset
/// the staleness signal covers; `hashes` is the `ContentSet` baseline over those
/// paths, the comparison key. `serde` round-trips both; `hashes` is rebuilt from
/// `paths` on every `prime` so it cannot drift.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub(super) struct Cache {
    #[serde(default)]
    paths: Vec<String>,
    #[serde(default)]
    pub(super) hashes: BTreeMap<String, String>,
}

impl Cache {
    /// The de-duplicated, sorted resolved fileset — the set the `[hashes]`
    /// baseline covers and the set `status` recomputes against.
    pub(super) fn tracked_paths(&self) -> Vec<String> {
        let mut set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for path in &self.paths {
            set.insert(path.clone());
        }
        set.into_iter().collect()
    }

    /// The stored `[hashes]` reconstituted as a `ContentSet` staleness baseline.
    fn baseline(&self) -> ContentSet {
        ContentSet::from_hashes(self.hashes.clone())
    }
}

/// The `cache.toml` path for a review id — beside `baton.toml`/`lock` in the
/// parent tree's gitignored state subtree (§6/§9).
fn cache_path(root: &Path, id: u32) -> PathBuf {
    state_dir(root, id).join("cache.toml")
}

/// Read the warm-cache if present (`None` = unprimed — no staleness signal to
/// report yet, design §9). A parse failure is a hard error (the file is ours).
pub(super) fn read_cache(root: &Path, id: u32) -> anyhow::Result<Option<Cache>> {
    let path = cache_path(root, id);
    match fs::read_to_string(&path) {
        Ok(text) => Ok(Some(toml::from_str(&text).with_context(|| {
            format!("Failed to parse warm-cache {}", path.display())
        })?)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("Failed to read {}", path.display())),
    }
}

/// Write the warm-cache atomically (temp+rename), creating the state subtree
/// first. The caller holds the per-review lock (§9 — prime serialises its write
/// against a concurrent prime/status).
fn write_cache(root: &Path, id: u32, cache: &Cache) -> anyhow::Result<()> {
    let dir = state_dir(root, id);
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let body = toml::to_string(cache).context("serialize warm-cache")?;
    crate::fsutil::write_atomic(&cache_path(root, id), body.as_bytes())
}

/// The staleness verdict for a primed cache (T-b naming): `current` when the
/// stored `[hashes]` baseline still matches the live `⋃ paths`, else `stale` with
/// the drifted paths listed (changed + removed[absence⇒stale, R1] + added). Pure
/// `diff` over the impure `compute` — the staleness DIFF is pure, `compute` (disk
/// + sha2) is the shell.
pub(super) fn cache_staleness(root: &Path, cache: &Cache) -> anyhow::Result<CacheVerdict> {
    let live = contentset::compute(root, &cache.tracked_paths())
        .context("hash the warm-cache's tracked paths")?;
    let drift = cache.baseline().diff(&live);
    let mut drifted: Vec<String> = Vec::new();
    drifted.extend(drift.changed);
    drifted.extend(drift.removed);
    drifted.extend(drift.added);
    if drifted.is_empty() {
        Ok(CacheVerdict::Current)
    } else {
        drifted.sort();
        drifted.dedup();
        Ok(CacheVerdict::Stale(drifted))
    }
}

/// The warm-cache staleness verdict (§9, T-b). `Stale` carries the drifted paths.
pub(super) enum CacheVerdict {
    Current,
    Stale(Vec<String>),
}

// ---------------------------------------------------------------------------
// `review prime` (Read class for authored conduct — no authored mutation — but it
// acquires the per-review lock to serialize the cache write, design §9).
// ---------------------------------------------------------------------------

/// Bundled `review prime` args (the clippy arg-ceiling — `cli-handler-args-struct`).
#[derive(Deserialize)]
pub(crate) struct PrimeArgs {
    pub(crate) reference: String,
}

// -- D11 degrade rendering fragments (STD-001: named, not inline literals) --

/// The degraded-prime line's fixed prefix — `print_review`'s `Primed` arm.
pub(super) const PRIMED_NOTHING_PREFIX: &str = "primed nothing: ";
/// The line printed after a degraded prime removes an earlier `cache.toml`.
pub(super) const REMOVED_PREVIOUS_CACHE: &str = "removed the previous cache";
/// The per-selector line prefix for a literal selector excluded as non-file.
pub(super) const SKIPPED_NON_FILE_SELECTOR_PREFIX: &str = "skipped non-file selector: ";

/// `doctrine review prime <RV-NNN>` — populate the warm-cache from the target
/// slice's selectors (SL-147 PHASE-05, F-4; degrade path: D11, IMP-259,
/// ISS-059, RV-396 `F-6`). Read-class for authored conduct (it mutates no
/// authored ledger) but it ACQUIRES THE PER-REVIEW LOCK to serialize the cache
/// write — and any degrade-path cache removal — against a concurrent
/// prime/status (§9). It runs neither the baton nor the CAS — only the lock.
///
/// The path-set is resolved from the slice the review targets: read the RV's
/// `[target].ref` and parse it as a slice ref. When the target is not a slice,
/// or the slice declares zero selectors, prime DEGRADES rather than erroring
/// (D11): it takes the lock, removes any earlier `cache.toml` (RV-396 `F-6` — a
/// stale cache must not survive a degrade), and returns `Primed` with
/// `tracked_count: 0` and a named `degraded` reason. Otherwise each selector is
/// resolved to concrete files (a literal passes through if it names a regular
/// file or is absent; a glob expands against `git ls-files`), the union is
/// hashed via `contentset::compute`, and `cache.toml` is written. The selector
/// read is committed authored slice TOML in the parent tree — review verbs
/// already refuse fork roots, so it is fork-safe.
pub(crate) fn run_prime(path: Option<PathBuf>, args: &PrimeArgs) -> anyhow::Result<ReviewOutput> {
    let root = resolve_review_root(path)?;
    let id = parse_ref(&args.reference)?;
    // The review must exist (the cache is a review's learned model) — fail early
    // with the same "not found" message the verbs give before touching state.
    let (_text, doc) = read_authored(&root, id)?;

    // Resolve the RV's target ref to a slice id. A non-slice target (a phase or
    // backlog ref) cannot source selectors — degrade named rather than error.
    let target_ref = &doc.target.reference;
    let Ok(slice_id) = crate::slice::parse_ref(target_ref) else {
        let reason =
            format!("target `{target_ref}` is not a slice reference (no selectors to prime from)");
        return degrade(&root, id, reason);
    };

    // The union of the slice's selectors (every intent). Zero selectors ⇒ nothing
    // to track — degrade named rather than write against an empty path-set.
    let selectors = crate::slice::selector_paths(&root, slice_id)?;
    if selectors.is_empty() {
        let reason = format!("slice {target_ref} declares no selectors");
        return degrade(&root, id, reason);
    }

    let resolved = resolve_selectors_to_fileset(&root, &selectors)?;

    // Serialize the cache write against a concurrent prime/status (§9). The lock —
    // and ONLY the lock — is reused from PHASE-03; no baton, no CAS.
    let _lock = LockGuard::acquire(&root, id)?;
    // Build `[hashes]` from the resolved fileset so the baseline cannot drift from
    // its source.
    let baseline =
        contentset::compute(&root, &resolved.fileset).context("hash the slice selector fileset")?;
    let cache = Cache {
        paths: resolved.fileset,
        hashes: baseline.hashes().clone(),
    };
    write_cache(&root, id, &cache)?;

    Ok(ReviewOutput::Primed {
        canonical: canonical_id(id),
        tracked_paths: cache.tracked_paths(),
        tracked_count: cache.tracked_paths().len(),
        degraded: None,
        cleared: false,
        skipped: resolved.skipped,
    })
}

/// The degrade path (D11): take the per-review lock, remove any earlier
/// `cache.toml` (RV-396 `F-6`), and report why prime tracked nothing.
fn degrade(root: &Path, id: u32, reason: String) -> anyhow::Result<ReviewOutput> {
    let _lock = LockGuard::acquire(root, id)?;
    let cleared = remove_cache(root, id)?;
    Ok(ReviewOutput::Primed {
        canonical: canonical_id(id),
        tracked_paths: vec![],
        tracked_count: 0,
        degraded: Some(reason),
        cleared,
        skipped: vec![],
    })
}

/// Remove `cache.toml` if present. `Ok(true)` when a file was removed,
/// `Ok(false)` when there was none to remove; any other error is propagated
/// with context (STD-003 — no silent skip).
fn remove_cache(root: &Path, id: u32) -> anyhow::Result<bool> {
    let path = cache_path(root, id);
    match fs::remove_file(&path) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e).with_context(|| format!("remove stale warm-cache {}", path.display())),
    }
}

/// The result of resolving selectors to a concrete fileset (ISS-059): the
/// tracked paths to hash, plus any literal selectors excluded as non-files.
struct Resolved {
    fileset: Vec<String>,
    /// Literal selectors naming a directory, a symlink (to anything), or
    /// another special file — sorted+deduped (P5: "non-file" is the filter's
    /// stated intent, not only the design's two named cases).
    skipped: Vec<String>,
}

/// Expand selector strings to a concrete, sorted+deduped fileset (SL-147 PHASE-05).
/// A LITERAL path (a degenerate glob — its match set is itself) passes through
/// only when it names a regular file or is absent (ISS-059): absence still
/// hashes-absent in `contentset::compute` and surfaces as drift the moment it
/// appears/vanishes (R1 absence⇒stale preserved), but a directory or a symlink
/// (to anything — never followed) is excluded and reported in `skipped`. A GLOB
/// expands against the tracked file set (`git ls-files --stage -z` in the parent
/// root) via the shared pure `globmatch` leaf. Only regular blobs are hashable
/// content: symlinks and gitlinks are excluded before matching rather than
/// followed through the working tree.
fn resolve_selectors_to_fileset(root: &Path, selectors: &[String]) -> anyhow::Result<Resolved> {
    // The tracked file set, read once (the impure `git` seam in the shell).
    let mut tracked: Option<Vec<String>> = None;
    let mut set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut skipped: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for sel in selectors {
        let pattern = glob::Pattern::new(sel)
            .with_context(|| format!("selector `{sel}` is not a valid glob/path pattern"))?;
        if is_literal_selector(sel) {
            // Degenerate glob — passes through unresolved (preserves absence⇒stale)
            // ONLY when it is a regular file or absent. `symlink_metadata` never
            // follows the symlink itself — a directory-or-anything-else symlink
            // must not be hashed as-is.
            match fs::symlink_metadata(root.join(sel)) {
                Ok(m) if m.file_type().is_file() => {
                    set.insert(sel.clone());
                }
                Ok(_) => {
                    skipped.insert(sel.clone());
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    set.insert(sel.clone());
                }
                Err(e) => {
                    return Err(e).with_context(|| format!("stat literal selector `{sel}`"));
                }
            }
            continue;
        }
        // Glob — expand against the tracked file set (lazily fetched once).
        if tracked.is_none() {
            let listing = crate::git::git_text(root, &["ls-files", "--stage", "-z"])
                .context("git ls-files for selector glob expansion")?;
            tracked = Some(parse_ls_files_stage_entries(&listing)?);
        }
        for path in tracked.as_deref().unwrap_or(&[]) {
            if crate::globmatch::glob_matches(&pattern, path) {
                set.insert(path.clone());
            }
        }
    }
    Ok(Resolved {
        fileset: set.into_iter().collect(),
        skipped: skipped.into_iter().collect(),
    })
}

const GIT_REGULAR_BLOB_MODES: [&str; 2] = ["100644", "100755"];

/// Parse NUL-delimited `git ls-files --stage -z` output and retain only regular
/// blob paths. The first TAB separates fixed metadata from the path; spaces,
/// tabs, and newlines inside the path therefore survive intact.
pub(super) fn parse_ls_files_stage_entries(listing: &str) -> anyhow::Result<Vec<String>> {
    let mut paths = Vec::new();
    for record in listing.split('\0').filter(|record| !record.is_empty()) {
        let malformed = || anyhow::anyhow!("malformed git ls-files --stage record: `{record}`");
        let (metadata, path) = record.split_once('\t').ok_or_else(&malformed)?;
        let mut fields = metadata.split_whitespace();
        let (Some(mode), Some(_object_id), Some(_stage), None) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            return Err(malformed());
        };
        if path.is_empty() {
            return Err(malformed());
        }
        if GIT_REGULAR_BLOB_MODES.contains(&mode) {
            paths.push(path.to_owned());
        }
    }
    Ok(paths)
}

/// Whether a selector string is a literal path (no glob metacharacters): a
/// degenerate `glob::Pattern` whose match set is itself. `glob`'s wildcards are
/// `*`, `?`, and `[…]` character classes — absent all three, the pattern matches
/// only its own string.
fn is_literal_selector(sel: &str) -> bool {
    !sel.contains(['*', '?', '['])
}

// ===========================================================================
// SL-268 PHASE-08 — D11 degrade + ISS-059 literal non-file filter (VT-1, VT-2).
// Fixtures reached via `super::super::tests` (`pub(super)` there): `fixture_rv`,
// `git_fixture_rv_with_selectors`, `plant_slice_with_selectors`, `new_args`.
// ===========================================================================
#[cfg(test)]
mod tests {
    use super::super::tests::{
        fixture_rv, git_fixture_rv_with_selectors, new_args, plant_slice_with_selectors,
    };
    use super::super::{Facet, print_review, run_new, run_status};
    use super::*;

    /// Extract the `Primed` fields a degrade/skip assertion needs, panicking
    /// (named) on any other variant — every test here calls `run_prime` and
    /// expects `Ok(Primed { .. })`, never an error (D11: prime no longer bails).
    fn primed_fields(out: ReviewOutput) -> (usize, Option<String>, bool, Vec<String>) {
        match out {
            ReviewOutput::Primed {
                tracked_count,
                degraded,
                cleared,
                skipped,
                ..
            } => (tracked_count, degraded, cleared, skipped),
            other => panic!("expected Primed, got {other:?}"),
        }
    }

    fn prime(root: &Path) -> anyhow::Result<ReviewOutput> {
        run_prime(
            Some(root.to_path_buf()),
            &PrimeArgs {
                reference: "RV-001".to_owned(),
            },
        )
    }

    /// D11 / IMP-259: a non-slice RV target (a phase or backlog ref) cannot
    /// source selectors — `run_prime` DEGRADES (no longer bails) with a named
    /// reason and writes nothing.
    #[test]
    fn prime_non_slice_degrades() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // A backlog target dir so `review new` resolves the ref, then mint an
        // RV against it (a non-slice target).
        let dir = root.join(".doctrine/backlog/issue/007");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("backlog-007.toml"), "id = 7\n").unwrap();
        run_new(
            Some(root.to_path_buf()),
            &new_args(Facet::Design, "ISS-007"),
        )
        .unwrap();

        let out = prime(root).unwrap();
        let rendered = print_review(&out);
        let (tracked_count, degraded, cleared, skipped) = primed_fields(out);
        assert_eq!(tracked_count, 0);
        let reason = degraded.expect("degraded reason");
        assert!(reason.contains("not a slice reference"), "{reason}");
        assert!(!cleared, "no earlier cache to clear");
        assert!(skipped.is_empty());
        assert_eq!(
            rendered,
            format!("RV-001 {PRIMED_NOTHING_PREFIX}{reason}\n")
        );
        assert!(read_cache(root, 1).unwrap().is_none(), "nothing written");
    }

    /// D11: a slice with ZERO selectors gives prime no path-set — degrades
    /// named rather than bailing.
    #[test]
    fn prime_zero_selectors_degrades() {
        let tmp = fixture_rv(); // SL-001 has no selectors.
        let root = tmp.path();

        let out = prime(root).unwrap();
        let rendered = print_review(&out);
        let (tracked_count, degraded, cleared, skipped) = primed_fields(out);
        assert_eq!(tracked_count, 0);
        let reason = degraded.expect("degraded reason");
        assert!(reason.contains("declares no selectors"), "{reason}");
        assert!(!cleared, "no earlier cache to clear");
        assert!(skipped.is_empty());
        assert_eq!(
            rendered,
            format!("RV-001 {PRIMED_NOTHING_PREFIX}{reason}\n")
        );
        assert!(read_cache(root, 1).unwrap().is_none(), "nothing written");
    }

    /// ISS-059: a literal selector naming a directory is excluded from the
    /// hashed fileset (never followed) and reported in `skipped`; an ABSENT
    /// literal is still kept (R1 absence⇒stale control).
    #[test]
    fn literal_directory_selector_skipped() {
        let tmp = git_fixture_rv_with_selectors(
            &["src", "src/a.rs", "absent.rs"],
            &[("src/a.rs", "a\n")],
        );
        let root = tmp.path();

        let out = prime(root).unwrap();
        let rendered = print_review(&out);
        let (tracked_count, degraded, cleared, skipped) = primed_fields(out);
        assert!(degraded.is_none(), "not a degrade — a real fileset");
        assert!(!cleared);
        assert_eq!(tracked_count, 2);
        assert_eq!(skipped, vec!["src".to_owned()]);
        assert!(
            rendered.contains(&format!("{SKIPPED_NON_FILE_SELECTOR_PREFIX}src\n")),
            "{rendered}"
        );

        let cache = read_cache(root, 1).unwrap().expect("cache primed");
        assert_eq!(
            cache.tracked_paths(),
            vec!["absent.rs".to_owned(), "src/a.rs".to_owned()]
        );
    }

    /// ISS-059: a literal symlink selector is excluded (never followed)
    /// regardless of what it points at — a symlink to a REGULAR FILE is the
    /// discriminating case (`symlink_metadata` alone tells them apart).
    #[test]
    #[cfg(unix)]
    fn literal_symlink_selector_skipped() {
        let tmp = git_fixture_rv_with_selectors(
            &["src/a.rs", "file-link", "dir-link"],
            &[("src/a.rs", "a\n")],
        );
        let root = tmp.path();
        // Symlinks planted after the fixture's commit — prime reads the
        // working tree for literals, not the git index.
        std::os::unix::fs::symlink(root.join("src/a.rs"), root.join("file-link")).unwrap();
        std::os::unix::fs::symlink(root.join("src"), root.join("dir-link")).unwrap();

        let out = prime(root).unwrap();
        let (tracked_count, degraded, cleared, skipped) = primed_fields(out);
        assert!(degraded.is_none());
        assert!(!cleared);
        assert_eq!(tracked_count, 1);
        assert_eq!(skipped, vec!["dir-link".to_owned(), "file-link".to_owned()]);

        let cache = read_cache(root, 1).unwrap().expect("cache primed");
        assert_eq!(cache.tracked_paths(), vec!["src/a.rs".to_owned()]);
    }

    /// D11 / RV-396 `F-6`: a degraded prime clears an earlier cache so `status`
    /// stops reporting a staleness signal for a path-set that no longer exists.
    #[test]
    fn degraded_prime_clears_stale_cache() {
        let tmp = git_fixture_rv_with_selectors(&["a.txt"], &[("a.txt", "x\n")]);
        let root = tmp.path();

        // Positive control: primed with a real path-set, status sees it.
        prime(root).unwrap();
        assert!(read_cache(root, 1).unwrap().is_some());
        let status = run_status(Some(root.to_path_buf()), "RV-001").unwrap();
        match status {
            ReviewOutput::Status { cache_primed, .. } => assert!(cache_primed),
            other => panic!("expected Status, got {other:?}"),
        }

        // Re-plant the slice with zero selectors, then prime again — degrade,
        // and the stale cache must not survive it (RV-396 F-6).
        plant_slice_with_selectors(root, 1, &[]);
        let out = prime(root).unwrap();
        let rendered = print_review(&out);
        let (tracked_count, degraded, cleared, _skipped) = primed_fields(out);
        assert_eq!(tracked_count, 0);
        assert!(degraded.is_some());
        assert!(cleared, "the stale cache was removed");
        assert!(
            rendered.ends_with(&format!("{REMOVED_PREVIOUS_CACHE}\n")),
            "{rendered}"
        );
        assert!(read_cache(root, 1).unwrap().is_none());

        let status = run_status(Some(root.to_path_buf()), "RV-001").unwrap();
        match status {
            ReviewOutput::Status {
                cache_primed,
                formatted,
                ..
            } => {
                assert!(!cache_primed);
                assert!(!formatted.contains("cache:"), "{formatted}");
            }
            other => panic!("expected Status, got {other:?}"),
        }
    }

    /// The degrade path still serialises against a concurrent prime/status —
    /// it takes the SAME per-review lock the success path does (§9).
    #[test]
    fn degraded_prime_takes_the_lock() {
        let tmp = git_fixture_rv_with_selectors(&["a.txt"], &[("a.txt", "x\n")]);
        let root = tmp.path();

        prime(root).unwrap();
        plant_slice_with_selectors(root, 1, &[]);

        let held = LockGuard::acquire(root, 1).unwrap();
        let err = prime(root).unwrap_err();
        assert!(format!("{err}").contains("busy"), "lock contention: {err}");
        assert!(
            read_cache(root, 1).unwrap().is_some(),
            "the held lock prevented the degrade's cache removal"
        );
        drop(held);

        let out = prime(root).unwrap();
        let (_tracked_count, degraded, cleared, _skipped) = primed_fields(out);
        assert!(degraded.is_some());
        assert!(cleared);
        assert!(read_cache(root, 1).unwrap().is_none());
    }
}
