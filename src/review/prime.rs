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

/// `doctrine review prime <RV-NNN>` — populate the warm-cache from the target
/// slice's selectors (SL-147 PHASE-05, F-4). Read-class for authored conduct (it
/// mutates no authored ledger) but it ACQUIRES THE PER-REVIEW LOCK to serialize
/// the cache write against a concurrent prime/status (§9). It runs neither the
/// baton nor the CAS — only the lock around the cache write.
///
/// The path-set is resolved from the slice the review targets: read the RV's
/// `[target].ref`, parse it as a slice ref (else `bail!`), read that slice's
/// selectors (union of all intents; a slice with zero selectors `bail!`s),
/// resolve each selector to concrete files (a literal passes through; a glob
/// expands against `git ls-files`), then hash the union via `contentset::compute`
/// and write `cache.toml`. The selector read is committed authored slice TOML in
/// the parent tree — review verbs already refuse fork roots, so it is fork-safe.
pub(crate) fn run_prime(path: Option<PathBuf>, args: &PrimeArgs) -> anyhow::Result<ReviewOutput> {
    let root = resolve_review_root(path)?;
    let id = parse_ref(&args.reference)?;
    // The review must exist (the cache is a review's learned model) — fail early
    // with the same "not found" message the verbs give before touching state.
    let (_text, doc) = read_authored(&root, id)?;

    // Resolve the RV's target ref to a slice id. A non-slice target (a phase or
    // backlog ref) cannot source selectors — fail named, never silently empty.
    let target_ref = &doc.target.reference;
    let slice_id = crate::slice::parse_ref(target_ref).with_context(|| {
        format!(
            "review prime needs a slice target: {} targets `{target_ref}`, which is not a slice reference (no selectors to prime from)",
            canonical_id(id)
        )
    })?;

    // The union of the slice's selectors (every intent). Zero selectors ⇒ nothing
    // to track — fail named rather than write an empty cache.
    let selectors = crate::slice::selector_paths(&root, slice_id)?;
    if selectors.is_empty() {
        anyhow::bail!(
            "slice {target_ref} declares no selectors — review prime has no path-set to track (add `[[selector]]` entries to the slice)"
        );
    }

    let fileset = resolve_selectors_to_fileset(&root, &selectors)?;

    // Serialize the cache write against a concurrent prime/status (§9). The lock —
    // and ONLY the lock — is reused from PHASE-03; no baton, no CAS.
    let _lock = LockGuard::acquire(&root, id)?;
    // Build `[hashes]` from the resolved fileset so the baseline cannot drift from
    // its source.
    let baseline =
        contentset::compute(&root, &fileset).context("hash the slice selector fileset")?;
    let cache = Cache {
        paths: fileset,
        hashes: baseline.hashes().clone(),
    };
    write_cache(&root, id, &cache)?;

    Ok(ReviewOutput::Primed {
        canonical: canonical_id(id),
        tracked_paths: cache.tracked_paths(),
        tracked_count: cache.tracked_paths().len(),
    })
}

/// Expand selector strings to a concrete, sorted+deduped fileset (SL-147 PHASE-05).
/// A LITERAL path (a degenerate glob — its match set is itself) passes through
/// directly: it is hashed as-is, so an absent declared literal still hashes-absent
/// in `contentset::compute` and surfaces as drift the moment it appears/vanishes
/// (R1 absence⇒stale preserved). A GLOB expands against the tracked file set
/// (`git ls-files --stage -z` in the parent root) via the shared pure `globmatch`
/// leaf. Only regular blobs are hashable content: symlinks and gitlinks are
/// excluded before matching rather than followed through the working tree.
fn resolve_selectors_to_fileset(root: &Path, selectors: &[String]) -> anyhow::Result<Vec<String>> {
    // The tracked file set, read once (the impure `git` seam in the shell).
    let mut tracked: Option<Vec<String>> = None;
    let mut set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for sel in selectors {
        let pattern = glob::Pattern::new(sel)
            .with_context(|| format!("selector `{sel}` is not a valid glob/path pattern"))?;
        if is_literal_selector(sel) {
            // Degenerate glob — passes through unresolved (preserves absence⇒stale).
            set.insert(sel.clone());
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
    Ok(set.into_iter().collect())
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
