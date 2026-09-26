// SPDX-License-Identifier: GPL-3.0-only
//! The turn guard — baton, lock, `with_turn`/`with_turn_hooked`, and `unlock`
//! (SL-268 PHASE-02 T5).

use super::{
    Act, Context, Deserialize, FindingRow, FindingState, Path, PathBuf, ReviewDoc, ReviewError,
    ReviewOutput, Role, Serialize, Write, authored_path, canonical_id, derived_status,
    finding_states_of, fs, io, parse_ref, read_authored, seed, write_counter_seed,
};

// ===========================================================================
// PHASE-03 — the verb family + runtime coordination (the turn guard).
//
// The full finding lifecycle (raise/dispose/verify/contest/withdraw) rides ONE
// higher-order seam, `with_turn` (design §6, D6) — the single home of
// D-C3 (authored-first/baton-last ordering), D-C4 (the static verb→role gate),
// and D-C4a (the create_new lock + the sha256 CAS, fired in TWO distinct windows:
// entry — a hand-edit landing BEFORE this invocation; pre-write — a hand-edit
// landing DURING it). The lock serializes concurrent invocations; the CAS catches
// out-of-band human edits the lock cannot see (no invocation ⇒ no lock).
//
// Locus = the parent tree's gitignored runtime state, `.doctrine/state/review/NNN/`
// (D4/D-C7). A review verb whose resolved root is a *fork* bails — fork-invoked
// review is IMP-024, not yet supported (the pilot invariant, enforced at root
// resolution).
// ===========================================================================

/// The runtime baton (design §6, D-C2) — gitignored, regenerable, never authored.
/// `await`/`authored_hash` are cache-derivable from the authored ledger (the
/// recompute floor). `rounds`/`contests` are the legacy counters: no longer
/// incremented (SL-268 sec-2), read only to seed a ledger's `rounds_base`/
/// `contests_base` at its first journalled write. A legacy baton's retired keys
/// still parse (no `deny_unknown_fields`) and drop at the next baton write.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub(super) struct Baton {
    /// The summarized turn (D-C8) — a display/routing convenience, never a gate.
    #[serde(default)]
    pub(super) awaiting: String,
    /// The CAS key: sha256 of the authored ledger bytes this baton was last
    /// reconciled against (D-C4a). A divergence ⇒ an out-of-band edit landed.
    #[serde(default)]
    pub(super) authored_hash: String,
    /// The legacy turn counter — carried forward, never bumped (SL-268 sec-2).
    #[serde(default)]
    pub(super) rounds: u32,
    /// The legacy contest counter — carried forward, never bumped.
    #[serde(default)]
    pub(super) contests: u32,
}

/// The runtime subtree for one review's baton + lock (design §6). Gitignored
/// (`.gitignore` already covers `.doctrine/state/`) and **invoking-tree** locus:
/// root-derived, so a review driven from a coordination worktree keeps its baton
/// there (ISS-275). Safe because that tree is its branch's sole writer, and a
/// baton is disposable — an absent one reads as cold and recomputes (D-C4a).
pub(super) fn state_dir(root: &Path, id: u32) -> PathBuf {
    root.join(".doctrine/state/review").join(format!("{id:03}"))
}

pub(super) fn baton_path(root: &Path, id: u32) -> PathBuf {
    state_dir(root, id).join("baton.toml")
}

pub(super) fn lock_path(root: &Path, id: u32) -> PathBuf {
    state_dir(root, id).join("lock")
}

/// Read the baton if present (`None` = cold — treat as a fresh recompute, D-C4a).
pub(super) fn read_baton(root: &Path, id: u32) -> anyhow::Result<Option<Baton>> {
    let path = baton_path(root, id);
    match fs::read_to_string(&path) {
        Ok(text) => Ok(Some(toml::from_str(&text).with_context(|| {
            format!("Failed to parse baton {}", path.display())
        })?)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("Failed to read baton {}", path.display())),
    }
}

/// Write the baton atomically (temp+rename), creating the state subtree first.
pub(super) fn write_baton(root: &Path, id: u32, baton: &Baton) -> anyhow::Result<()> {
    let dir = state_dir(root, id);
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let body = toml::to_string(baton).context("serialize baton")?;
    crate::fsutil::write_atomic(&baton_path(root, id), body.as_bytes())
}

/// Compute the `(await, authored_hash)` the baton should carry for a ledger whose
/// findings are `findings`, concluded marker is `concluded`, and whose bytes hash
/// to `hash` — the D-C2 recompute floor reused by entry-CAS heal, the per-turn
/// refresh, and `status`.
pub(super) fn reconcile_baton_fields(
    findings: &[FindingState],
    concluded: bool,
    hash: &str,
) -> (String, String) {
    let (_, awaited) = derived_status(findings, concluded);
    (awaited.as_str().to_owned(), hash.to_owned())
}

/// A RAII lock: `create_new` the lockfile on construction (an `AlreadyExists`
/// race is the caller's "RV-NNN busy" bail), remove it on `drop` — covering the
/// normal AND panic paths (NOT a hard-kill `-9`, which leaves a stale lock for
/// `review unlock`). The lock serializes concurrent *invocations* only; it is
/// held within one invocation and the turn persists via the baton (design §6).
pub(super) struct LockGuard {
    path: PathBuf,
}

impl LockGuard {
    /// Acquire the per-review lock, writing a `pid timestamp` diagnostic body
    /// (`review unlock` surfaces it on a stale lock). `AlreadyExists` ⇒ a
    /// concurrent invocation holds it ⇒ a clean "busy; re-run" bail, no clobber.
    pub(super) fn acquire(root: &Path, id: u32) -> anyhow::Result<Self> {
        let dir = state_dir(root, id);
        fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
        let path = lock_path(root, id);
        match crate::fsutil::create_new_file(&path) {
            Ok(mut file) => {
                let stamp = crate::clock::now_timestamp().unwrap_or_default();
                let body = format!("pid = {}\nacquired = \"{stamp}\"\n", std::process::id());
                // Best-effort diagnostics body; a write failure does not invalidate
                // the lock (the file's existence is the mutex, not its contents).
                file.write_all(body.as_bytes())
                    .with_context(|| format!("write lock body {}", path.display()))?;
                Ok(Self { path })
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                // Structured contention (IMP-107): the MCP transport maps this by
                // variant identity to `LOCK_CONTENTION`; the Display impl
                // (`{canonical}: {details}`) reproduces the CLI "busy; re-run"
                // guidance, so both arms carry the `review unlock` hint.
                Err(ReviewError::LockContention {
                    canonical: canonical_id(id),
                    details: "busy (another `review` invocation holds the lock); re-run \
                              (a stale lock from a hard kill clears with `review unlock`)"
                        .to_owned(),
                }
                .into())
            }
            Err(e) => Err(e).with_context(|| format!("acquire lock {}", path.display())),
        }
    }
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        // Best-effort: a failed removal leaves a stale lock for `review unlock`.
        // Drop cannot propagate an error; the must-use Result is deliberately
        // discarded into a binding (the sanctioned form under the repo lint).
        let _ignored = fs::remove_file(&self.path);
    }
}

/// Resolve the project root for a review verb and ENFORCE the pilot invariant
/// (design D4/D-C1): a verb whose resolved root is a *worker fork* bails —
/// fork-invoked review is IMP-024, not yet supported. A fork's `WITHHELD` tier
/// keeps it from seeing the parent's gitignored state (`worktree/mod.rs`), so it
/// cannot co-write the baton it would need.
///
/// The test is the ROLE, not mere linkage (ISS-275). A dispatch *coordination*
/// tree is a linked worktree too, but it is the sole writer of `dispatch/<NNN>`
/// and carries its own state tier — and [`state_dir`] is root-derived, so its
/// baton lands in its own tree with nothing to contend over. Refusing it stranded
/// the design-gate reviews that ADR-012's topology puts there in the first place.
///
/// The guard lives here, in the shell, at root resolution — every verb routes
/// through it.
pub(super) fn resolve_review_root(path: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let linked = crate::worktree::is_linked_worktree(&root).unwrap_or(false);
    let branch = crate::git::current_branch(&root).unwrap_or(None);
    if crate::worktree::classify_worktree_role(branch.as_deref(), linked) == "fork" {
        anyhow::bail!(
            "review verbs are not supported on a worktree fork (IMP-024): the turn \
             baton lives in the parent tree's gitignored state, which a fork cannot \
             co-write. Run `review` from the parent or coordination tree."
        );
    }
    Ok(root)
}

/// A test seam for the pre-write CAS window (design §6 step 5). The default is a
/// no-op; a concurrency test injects a hand-edit here to fire mid-invocation,
/// between the step-2 read and the step-5 write — deterministically, without
/// threads. `with_turn` is the production entry point (no hook).
type MidTurnHook<'a> = &'a dyn Fn();

/// The single turn-taking seam (design §6, D6). Runs the numbered protocol:
///
/// 1. acquire the `create_new` lock (RAII) — `AlreadyExists` ⇒ "busy; re-run".
/// 2. read + snapshot the authored ledger bytes.
/// 3. ENTRY CAS: `sha256(authored) ≠ baton.authored_hash` ⇒ heal the baton (the
///    D-C2 recompute), bail "ledger changed underneath — re-run" (missing baton
///    ⇒ cold, proceed). Catches an edit landing BEFORE this invocation.
/// 4. STATIC role check: `role == act.required_role()` — mismatch ⇒ bail (D-C4).
/// 5. AUTHORED FIRST: run the closure `f` (per-finding `can()` + the edit), then
///    PRE-WRITE CAS (re-read bytes ≠ the step-2 snapshot ⇒ bail, do NOT write —
///    catches an edit landing DURING this invocation), else `write_atomic`.
/// 6. recompute `await` + the new hash from the written ledger.
/// 7. BATON LAST: `write_atomic` the baton.
/// 8. release the lock (`LockGuard` drop).
pub(super) fn with_turn<F, T>(root: &Path, id: u32, act: Act, role: Role, f: F) -> anyhow::Result<T>
where
    F: FnOnce(&mut toml_edit::DocumentMut, &[FindingRow]) -> anyhow::Result<T>,
{
    with_turn_hooked(root, id, act, role, &|| {}, f)
}

/// `with_turn` with an injectable mid-turn hook (the pre-write CAS test seam).
pub(super) fn with_turn_hooked<F, T>(
    root: &Path,
    id: u32,
    act: Act,
    role: Role,
    mid_turn: MidTurnHook<'_>,
    f: F,
) -> anyhow::Result<T>
where
    F: FnOnce(&mut toml_edit::DocumentMut, &[FindingRow]) -> anyhow::Result<T>,
{
    // 1. acquire lock (RAII — released on every exit path below, incl. panic).
    let _lock = LockGuard::acquire(root, id)?;

    // 2. read + snapshot the authored bytes.
    let (snapshot, doc) = read_authored(root, id)?;
    let snapshot_hash = crate::git::sha256(snapshot.as_bytes());

    // 3. ENTRY CAS — an edit landed BEFORE this invocation (baton stale).
    //    (a missing baton ⇒ cold — proceed; the per-turn write seeds it.) The
    //    baton is read once, under the lock: the seed and step 7 reuse it.
    let prior = read_baton(root, id)?;
    if let Some(baton) = prior.as_ref().filter(|b| b.authored_hash != snapshot_hash) {
        // Heal: recompute await from the authored truth (D-C2), refresh the
        // baton's CAS key, preserve the legacy counters, then bail.
        let (awaiting, hash) = reconcile_baton_fields(
            &finding_states_of(&doc),
            doc.review.concluded,
            &snapshot_hash,
        );
        let healed = Baton {
            awaiting,
            authored_hash: hash,
            ..baton.clone()
        };
        write_baton(root, id, &healed)?;
        anyhow::bail!(
            "{} ledger changed underneath the baton — re-run (the baton has been \
             refreshed from the authored ledger)",
            canonical_id(id)
        );
    }

    // 4. STATIC role check (D-C4) — the half the wrapper owns.
    if role != act.required_role() {
        return Err(ReviewError::RoleMismatch {
            expected: act.required_role(),
            actual: role,
            act,
        }
        .into());
    }

    // 5. AUTHORED FIRST — the closure runs the per-finding can() + applies the
    //    edit-preserving edit, then the PRE-WRITE CAS re-reads the bytes.
    let mut document = snapshot
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", authored_path(root, id).display()))?;
    let prior = prior.unwrap_or_default();
    // The counter seed (SL-268 sec-2): a ledger's first journalled write copies
    // the legacy baton counters into `[review]`, in THIS edit — so it rides the
    // pre-write CAS below and never gets a write of its own.
    if let Some(base) = seed(&doc, (prior.rounds, prior.contests)) {
        write_counter_seed(&mut document, base)?;
    }
    let result = f(&mut document, &doc.finding)?;

    // Test seam: a hand-edit injected here lands AFTER the step-2 read and BEFORE
    // the step-5 write — the exact window the pre-write CAS must catch.
    mid_turn();

    // PRE-WRITE CAS — the authored bytes must still match the step-2 snapshot.
    let current = fs::read_to_string(authored_path(root, id))
        .with_context(|| format!("re-read {}", authored_path(root, id).display()))?;
    if crate::git::sha256(current.as_bytes()) != snapshot_hash {
        anyhow::bail!(
            "{} ledger changed underneath this turn — re-run (a hand-edit landed \
             mid-turn; nothing was written, no clobber)",
            canonical_id(id)
        );
    }
    let new_body = document.to_string();
    crate::fsutil::write_atomic(&authored_path(root, id), new_body.as_bytes())?;

    // 6. recompute await + the new hash from the just-written ledger.
    let new_hash = crate::git::sha256(new_body.as_bytes());
    let new_doc: ReviewDoc = toml::from_str(&new_body)
        .with_context(|| format!("re-parse {}", authored_path(root, id).display()))?;
    let (awaiting, hash) = reconcile_baton_fields(
        &finding_states_of(&new_doc),
        new_doc.review.concluded,
        &new_hash,
    );

    // 7. BATON LAST — the legacy counters ride forward unchanged: the ledger's
    //    turn journal counts now (SL-268 sec-2).
    let baton = Baton {
        awaiting,
        authored_hash: hash,
        ..prior
    };
    write_baton(root, id, &baton)?;

    // 8. release lock — `_lock` drops at scope end.
    Ok(result)
}

/// `doctrine review unlock <RV-NNN>` — the escape hatch for a stale lock left by a
/// hard kill (`-9`, which RAII cannot cover, design §6/R-b). Removes the lockfile;
/// its `pid`/`acquired` body aids the operator's "is this really stale?" judgement
/// (printed before removal).
pub(crate) fn run_unlock(path: Option<PathBuf>, reference: &str) -> anyhow::Result<ReviewOutput> {
    let root = resolve_review_root(path)?;
    let id = parse_ref(reference)?;
    let canonical = canonical_id(id);
    let lock = lock_path(&root, id);
    match fs::read_to_string(&lock) {
        Ok(body) => {
            let mut formatted = format!("Removing stale lock for {canonical}:\n");
            for line in body.lines() {
                formatted.push_str("  ");
                formatted.push_str(line);
                formatted.push('\n');
            }
            fs::remove_file(&lock).with_context(|| format!("remove lock {}", lock.display()))?;
            Ok(ReviewOutput::Unlocked {
                canonical,
                formatted,
            })
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(ReviewOutput::Unlocked {
            canonical,
            formatted: String::new(),
        }),
        Err(e) => Err(e).with_context(|| format!("read lock {}", lock.display())),
    }
}
