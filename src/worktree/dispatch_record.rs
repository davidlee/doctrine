// SPDX-License-Identifier: GPL-3.0-only
//! Per-worktree dispatch record — the trust anchor `worker_commit` consumes (SL-198
//! PHASE-01, design §5.2/§5.3/§5.5).
//!
//! A SIBLING concern to the jail policy (`create.rs`): the trusted create-fork hook
//! writes an atomic record at the fork point (before the worker's first tool call);
//! gc/reap deletes it when it removes the worker worktree; and a resolver maps an
//! OPAQUE, sanitised agent-id → that record on exactly one live, consistent hit, else
//! a typed refusal. No worker-supplied path ever enters resolution.
//!
//! ADR-001 leaf/engine split (mirror of `classify_gc` / `run_gc`): a PURE classifier
//! ([`classify_resolve`]) reasons over already-gathered FACTS (no git / disk / clock);
//! the impure [`resolve_agent`] gathers them — `git worktree list` (via the shared
//! [`git::worktree_for_ref`] seam), the record disk read, and the branch/base
//! rev-parse — then classifies.

use super::create::{WORKTREES_SUBDIR, sanitise_name};
use crate::git;
use anyhow::Context;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// Where the per-worktree dispatch record lives under the coordination root:
/// `<coord>/.doctrine/state/dispatch/record/<name>.toml`. A SIBLING subpath to the
/// jail policy ([`super::create::JAIL_SUBPATH`], STD-001 single-source named const —
/// no magic strings), OUTSIDE every worktree and ro to the worker. Runtime state:
/// gitignored, deleted by gc with teardown (design §5.3).
pub(crate) const RECORD_SUBPATH: &str = ".doctrine/state/dispatch/record";

/// The DURABLE FORK BINDING (SL-228 PHASE-04, design §3): which `(slice, phase)` a
/// fork belongs to, snapshotted at fork creation exactly as `base` is. This is what
/// lets a Class-2 recorder (`worker_commit`) and heal-forward (`dispatch_import`)
/// name the funnel row a live fork's commit belongs to WITHOUT re-reading mutable
/// arming state — the arming slots are consumed one-shot at the fork point, so a
/// stale arm can never mis-bind the next spawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ForkBinding {
    /// The slice being dispatched.
    pub(crate) slice: u32,
    /// The phase id (`PHASE-NN`) this fork was armed for.
    pub(crate) phase: String,
}

/// The per-worktree dispatch record (design §5.3) — the single source of truth
/// `worker_commit` (PHASE-02) consumes. Snapshotted at the fork point by the trusted
/// create-fork hook and NEVER re-derived from mutable arming state: `base` is B
/// captured at fork time (supersedes the racy live-arming-slot read), and `slice` +
/// `phase` are the durable fork binding captured the same way (SL-228 PHASE-04).
///
/// `slice`/`phase` are OPTIONAL on the wire so a record written before the binding
/// existed still parses — an unbound record is not a parse failure, it is a fork whose
/// phase cannot be PROVEN, which [`require_binding`] turns into the typed
/// `unprovable-fork` refusal rather than a guess (zero-rescue, design §3).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DispatchRecord {
    /// The harness-assigned worktree name — the opaque agent-id resolution keys on.
    pub(crate) name: String,
    /// The worker worktree dir: `<coord>/.worktrees/<name>`.
    pub(crate) dir: PathBuf,
    /// The worker fork branch: `dispatch/<name>`.
    pub(crate) branch: String,
    /// The base commit B, snapshotted at fork time (NOT re-read from the arming slot).
    pub(crate) base: String,
    /// The coordination root the worktree was forked from.
    pub(crate) coord: PathBuf,
    /// The bound slice, snapshotted at fork time. Absent ⇒ unbound fork.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) slice: Option<u32>,
    /// The bound phase id, snapshotted at fork time. Absent ⇒ unbound fork.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) phase: Option<String>,
}

impl DispatchRecord {
    /// The durable fork binding, when this record carries a COMPLETE one. A half-bound
    /// record (slice without phase, or vice versa) is not a binding — it names no funnel
    /// row — so it folds to `None` rather than half-proving anything.
    pub(crate) fn binding(&self) -> Option<ForkBinding> {
        match (self.slice, self.phase.as_deref()) {
            (Some(slice), Some(phase)) if !phase.is_empty() => Some(ForkBinding {
                slice,
                phase: phase.to_owned(),
            }),
            _ => None,
        }
    }
}

/// The durable binding, or the typed refusal (design §3). PURE. A live, consistent fork
/// whose record carries no `(slice, phase)` is not PROVABLE: no verb may guess which
/// phase its commit belongs to, so both Class-2 consumers (`worker_commit` now,
/// `dispatch_import`'s heal-forward next) refuse identically through this ONE seam —
/// there is no "skip and let the other heal" arm, because the other refuses too.
pub(crate) fn require_binding(record: &DispatchRecord) -> Result<ForkBinding, ResolveRefusal> {
    record.binding().ok_or(ResolveRefusal::UnprovableFork)
}

/// The record file path under `coord` (one owner of the `<RECORD_SUBPATH>/<name>.toml`
/// shape — provision, delete, and resolve all route through here).
fn record_path(coord: &Path, name: &str) -> PathBuf {
    coord.join(RECORD_SUBPATH).join(format!("{name}.toml"))
}

/// Write the per-worktree record atomically beside the jail policy at the fork point
/// (design §5.3, EX-1). Mirror of `create.rs`'s `provision_jail_policy`: build the dest
/// under [`RECORD_SUBPATH`], `create_dir_all`, then `write_atomic` the serialised TOML
/// (no torn temp). Fail-closed — a failed provision propagates and aborts the spawn,
/// exactly like the jail-policy write, so a worker never spawns without its trust anchor.
pub(crate) fn provision_dispatch_record(
    coord: &Path,
    name: &str,
    base: &str,
    dir: &Path,
    branch: &str,
    binding: Option<&ForkBinding>,
) -> anyhow::Result<()> {
    let record = DispatchRecord {
        name: name.to_string(),
        dir: dir.to_path_buf(),
        branch: branch.to_string(),
        base: base.to_string(),
        coord: coord.to_path_buf(),
        slice: binding.map(|b| b.slice),
        phase: binding.map(|b| b.phase.clone()),
    };
    let body = toml::to_string(&record)
        .with_context(|| format!("serialise dispatch record for {name}"))?;
    let dest = record_path(coord, name);
    let parent = dest
        .parent()
        .ok_or_else(|| anyhow::anyhow!("record path {} has no parent", dest.display()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("create dispatch record dir {}", parent.display()))?;
    crate::fsutil::write_atomic(&dest, body.as_bytes())
        .with_context(|| format!("write dispatch record {}", dest.display()))?;
    Ok(())
}

/// **BIND** — step 2 of the fork's `claim → bind → act` sequence (SL-228 PHASE-04,
/// design §3). Writes the record for `name` UNDER A HELD CLAIM (the branch ref exists
/// and the per-name claim lock is held), so no concurrent writer for this name can
/// exist. The NO-CLOBBER belt is kept regardless of that reasoning: an existing record
/// means the name is already bound — refuse rather than repoint a live fork's binding
/// at a different base/phase, which is precisely the clobber RV-304 F-2 contests.
///
/// Distinct from [`provision_dispatch_record`], which deliberately OVERWRITES: the
/// fixup-revive path re-stamps a live record's `base` to `fork_tip` on purpose
/// (design §5.5). Bind is the spawn-time door; provision is the re-stamp door.
pub(crate) fn bind_dispatch_record(
    coord: &Path,
    name: &str,
    base: &str,
    dir: &Path,
    branch: &str,
    binding: Option<&ForkBinding>,
) -> anyhow::Result<()> {
    let dest = record_path(coord, name);
    if dest.exists() {
        anyhow::bail!(
            "bind-refused: a dispatch record already exists for {name} ({}) — the name is \
             already bound; sweep the residue (`doctrine worktree gc --fork {branch}`) before \
             re-claiming it",
            dest.display()
        );
    }
    provision_dispatch_record(coord, name, base, dir, branch, binding)
}

/// Delete the per-worktree record when gc reaps the worktree (design §5.3, EX-2) —
/// closes the stale-oracle: no record survives without a live worktree. Absent ⇒
/// no-op (an idempotent rerun, or a non-dispatch fork that never had one).
pub(crate) fn delete_dispatch_record(coord: &Path, name: &str) -> anyhow::Result<()> {
    let path = record_path(coord, name);
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e).with_context(|| format!("delete dispatch record {}", path.display())),
    }
}

/// Why [`resolve_agent`] refuses (design §5.2 step 1, §5.5 INV-4). Fails closed with a
/// distinct named token — the property the goldens assert, never a proxy. SEPARATE
/// from the create / gc refusal enums: resolution is its own verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResolveRefusal {
    /// Zero live worktree hits for `dispatch/<agent>` — no such live agent (also the
    /// fold for an unsanitisable agent-id: definitionally not a live agent).
    UnknownAgent,
    /// More than one live hit. git guarantees ≤1 worktree per branch, so this is
    /// normally unreachable; kept as a DEFENSIVE refusal because name uniqueness is
    /// not source-guaranteed (X-5).
    AmbiguousAgent,
    /// A single hit whose record ↔ worktree is inconsistent: record absent/corrupt,
    /// `dir` gone, branch unresolved, or the branch head does not match the CALLER'S
    /// declared [`ForkExpect`] (design §5.2 step 1d, SL-228 design §3 / RV-304 F-7).
    StaleRecord,
    /// A live, consistent fork whose record carries NO durable `(slice, phase)`
    /// binding: nothing can prove which funnel row its commit belongs to, so every
    /// consumer refuses here rather than guessing (SL-228, design §3 — the deliberate
    /// triage beat; zero-rescue never guesses ownership).
    UnprovableFork,
}

impl ResolveRefusal {
    /// The distinct named token each refusal fails closed with.
    pub(crate) fn token(self) -> &'static str {
        match self {
            ResolveRefusal::UnknownAgent => "unknown-agent",
            ResolveRefusal::AmbiguousAgent => "ambiguous-agent",
            ResolveRefusal::StaleRecord => "stale-record",
            ResolveRefusal::UnprovableFork => "unprovable-fork",
        }
    }
}

/// The CALLER-DECLARED expected fork state (SL-228 design §3, RV-304 F-7). Before this
/// existed the classifier folded ANY `HEAD != base` to `stale-record` — which is
/// precisely the ADVANCED state heal-forward and the retry signature need to see, so a
/// legitimate re-drive was misdiagnosed. The expectation is declared by the caller
/// because only the caller knows which leg it is on; every OTHER consistency check
/// (live worktree, record present, dir exists, branch resolves) is unchanged, and no
/// parallel raw record read exists — this is still the single door.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ForkExpect {
    /// The fork has NOT been committed on yet: `HEAD == base`. Declared by
    /// `worker_commit`'s FIRST-COMMIT leg (its pre-act base guard, INV-1).
    AtBase,
    /// The fork has advanced by EXACTLY ONE commit `C` with `C^ == base` — the
    /// import / heal-forward state, and `worker_commit`'s RETRY SIGNATURE
    /// (RV-305 F-1). A multi-commit or unrelated advance is still `stale-record`.
    Advanced,
}

/// The gathered, impure-read facts the PURE [`classify_resolve`] reasons over (mirror
/// of `GcState`). Every field is a FACT gathered in [`resolve_agent`]'s shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolveFacts {
    /// Live worktrees checked out on `dispatch/<agent>`. git guarantees ≤1 (so this is
    /// 0 or 1 through the [`git::worktree_for_ref`] seam), but `> 1` is defended.
    pub(crate) worktree_hits: usize,
    /// The record parsed from `<coord>/RECORD_SUBPATH/<agent>.toml` on the single hit —
    /// `None` when coord recovery failed or the file was absent/corrupt (⇒ stale).
    pub(crate) record: Option<DispatchRecord>,
    /// `record.dir` still exists on disk.
    pub(crate) dir_exists: bool,
    /// The resolved commit of `dispatch/<agent>` (branch tip); `None` ⇒ branch unresolved.
    pub(crate) branch_head: Option<String>,
    /// The resolved commit of `record.base`; `None` ⇒ record absent or base unresolvable.
    pub(crate) base_commit: Option<String>,
    /// `branch_head`'s parents. Exactly one ⇒ a non-merge child; the `Advanced`
    /// expectation reads `C^` from here (never a second raw git read elsewhere).
    pub(crate) head_parents: Vec<String>,
    /// What the CALLER expects of the fork's position — see [`ForkExpect`].
    pub(crate) expect: ForkExpect,
}

/// Whether the branch head satisfies the caller's declared expectation. PURE — the
/// ONE place `AtBase` and `Advanced` are spelled out, so neither consumer re-derives it.
fn head_meets(expect: ForkExpect, head: &str, base: &str, head_parents: &[String]) -> bool {
    match expect {
        ForkExpect::AtBase => head == base,
        // Exactly one commit past base, and non-merge: `C != B` and `C^ == B`.
        ForkExpect::Advanced => head != base && matches!(head_parents, [parent] if parent == base),
    }
}

/// PURE resolver classifier (no git / disk — ADR-001 leaf). Mirror of `classify_gc`:
/// 0 hits ⇒ `unknown-agent`; `> 1` ⇒ `ambiguous-agent`; a single hit whose record is
/// missing, whose `dir` is gone, whose branch is unresolved, or whose head does not
/// meet the caller's declared [`ForkExpect`] ⇒ `stale-record`; otherwise the record
/// (design §5.2 step 1d, EX-3; SL-228 design §3 for the expectation split).
pub(crate) fn classify_resolve(facts: ResolveFacts) -> Result<DispatchRecord, ResolveRefusal> {
    if facts.worktree_hits == 0 {
        return Err(ResolveRefusal::UnknownAgent);
    }
    if facts.worktree_hits > 1 {
        return Err(ResolveRefusal::AmbiguousAgent);
    }
    let Some(record) = facts.record else {
        return Err(ResolveRefusal::StaleRecord);
    };
    if !facts.dir_exists {
        return Err(ResolveRefusal::StaleRecord);
    }
    let Some(head) = facts.branch_head else {
        return Err(ResolveRefusal::StaleRecord);
    };
    // `base_commit` None (record base unresolvable) is stale under EVERY expectation:
    // with no base there is nothing to be at, or one commit past.
    let Some(base) = facts.base_commit else {
        return Err(ResolveRefusal::StaleRecord);
    };
    if !head_meets(facts.expect, &head, &base, &facts.head_parents) {
        return Err(ResolveRefusal::StaleRecord);
    }
    Ok(record)
}

/// Recover `(coord, name)` from a worker worktree dir by layout-stripping the
/// `<WORKTREES_SUBDIR>/<name>` shape create-fork lays down (design §5.3 — THE one owner
/// of the `.worktrees/<name>` layout, in both directions). `None` when the dir does not
/// match the layout, which is exactly "this fork is not resolvable".
pub(super) fn coord_and_name(dir: &Path) -> Option<(PathBuf, String)> {
    let name = dir.file_name()?.to_str()?.to_owned();
    let worktrees = dir.parent()?;
    if worktrees.file_name()?.to_str()? != WORKTREES_SUBDIR {
        return None;
    }
    Some((worktrees.parent()?.to_path_buf(), name))
}

/// The coordination root for a worker worktree dir KNOWN to belong to `name`.
fn coord_from_worktree_dir(dir: &Path, name: &str) -> Option<PathBuf> {
    coord_and_name(dir).and_then(|(coord, got)| (got == name).then_some(coord))
}

/// Read + parse the record at `<coord>/RECORD_SUBPATH/<name>.toml`; `None` on any
/// read/parse failure (folded to `stale-record` by the classifier).
fn read_record(coord: &Path, name: &str) -> Option<DispatchRecord> {
    let raw = fs::read_to_string(record_path(coord, name)).ok()?;
    toml::from_str(&raw).ok()
}

/// Resolve a ref/sha to its commit oid via `rev-parse --verify --quiet <rev>^{commit}`;
/// `None` when it does not resolve to a commit. Impure (the one git read).
fn resolve_commit(root: &Path, rev: &str) -> Option<String> {
    git::git_opt(
        root,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{rev}^{{commit}}"),
        ],
    )
    .ok()
    .flatten()
}

/// Resolve an OPAQUE, sanitised agent-id to its live per-worktree record on exactly one
/// consistent hit, else a typed [`ResolveRefusal`] (design §5.2 step 1, EX-3/EX-4). No
/// worker-supplied path enters resolution: `agent` is sanitised through the SAME
/// validator create-fork uses BEFORE any path join (X-4), then the worker branch
/// `dispatch/<agent>` keys the EXISTING [`git::worktree_for_ref`] seam (git guarantees
/// ≤1 worktree per branch, so a valid agent resolves to exactly one worktree — no coord
/// registry, no worker path input, EX-4).
///
/// Gather (impure) → [`classify_resolve`] (pure):
/// 1. sanitise `agent` FIRST — an unsanitisable id folds to `unknown-agent`,
/// 2. gather the worker worktree on `dispatch/<agent>` (0 or 1 via the seam),
/// 3. recover `coord` by layout-stripping `.worktrees/<name>`; read the record,
/// 4. gather `record.dir` existence, the branch tip, its parents, and the record's base,
/// 5. classify against the caller's declared `expect`.
pub(crate) fn resolve_agent(
    root: &Path,
    agent: &str,
    expect: ForkExpect,
) -> Result<DispatchRecord, ResolveRefusal> {
    // Sanitise BEFORE any path join (X-4). An unsanitisable id is definitionally not a
    // live agent, so fold it to `unknown-agent` rather than joining a hostile name.
    let Ok(name) = sanitise_name(agent) else {
        return Err(ResolveRefusal::UnknownAgent);
    };
    let branch_ref = format!("refs/heads/dispatch/{name}");

    // gather: the worker worktree checked out on dispatch/<agent> (git guarantees ≤1).
    let worktree = git::worktree_for_ref(root, &branch_ref).unwrap_or(None);
    let worktree_hits = usize::from(worktree.is_some());

    // Recover coord by stripping `.worktrees/<name>` from the resolved dir; read the record.
    let record = worktree
        .as_deref()
        .and_then(|dir| coord_from_worktree_dir(dir, &name))
        .and_then(|coord| read_record(&coord, &name));

    let dir_exists = record.as_ref().is_some_and(|r| r.dir.exists());
    let branch_head = resolve_commit(root, &branch_ref);
    let base_commit = record.as_ref().and_then(|r| resolve_commit(root, &r.base));
    // `C^` for the `Advanced` expectation, via the SHARED parents seam. Empty when the
    // head does not resolve (or is a root commit) — both fold to "not advanced".
    let head_parents = branch_head
        .as_deref()
        .and_then(|head| git::parents(root, head).ok())
        .unwrap_or_default();

    classify_resolve(ResolveFacts {
        worktree_hits,
        record,
        dir_exists,
        branch_head,
        base_commit,
        head_parents,
        expect,
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "tests: fail-fast unwrap on fixture setup is idiomatic"
)]
mod tests {
    use super::*;

    const B: &str = "1111111111111111111111111111111111111111";
    const C: &str = "2222222222222222222222222222222222222222";
    const D: &str = "3333333333333333333333333333333333333333";

    fn record() -> DispatchRecord {
        DispatchRecord {
            name: "wk1".to_owned(),
            dir: PathBuf::from("/coord/.worktrees/wk1"),
            branch: "dispatch/wk1".to_owned(),
            base: B.to_owned(),
            coord: PathBuf::from("/coord"),
            slice: Some(228),
            phase: Some("PHASE-04".to_owned()),
        }
    }

    /// Consistent facts (one live hit, record present, dir there) with a chosen head.
    fn facts(head: &str, parents: &[&str], expect: ForkExpect) -> ResolveFacts {
        ResolveFacts {
            worktree_hits: 1,
            record: Some(record()),
            dir_exists: true,
            branch_head: Some(head.to_owned()),
            base_commit: Some(B.to_owned()),
            head_parents: parents.iter().map(|p| (*p).to_owned()).collect(),
            expect,
        }
    }

    // --- VT-2: the caller-declared expected fork state --------------------------------

    #[test]
    fn at_base_accepts_only_head_equal_to_base() {
        // The FIRST-COMMIT leg's expectation: nothing has been committed yet.
        assert!(classify_resolve(facts(B, &[], ForkExpect::AtBase)).is_ok());
        // One commit past base is NOT `AtBase` — the caller declared it had not committed.
        assert_eq!(
            classify_resolve(facts(C, &[B], ForkExpect::AtBase)),
            Err(ResolveRefusal::StaleRecord)
        );
        // An unrelated head is stale under `AtBase` exactly as before the split.
        assert_eq!(
            classify_resolve(facts(D, &[C], ForkExpect::AtBase)),
            Err(ResolveRefusal::StaleRecord)
        );
    }

    #[test]
    fn advanced_accepts_exactly_one_non_merge_commit_past_base() {
        // The RETRY SIGNATURE / heal-forward expectation: exactly one commit C, C^ == B.
        assert!(classify_resolve(facts(C, &[B], ForkExpect::Advanced)).is_ok());
        // Still AT base is not advanced — there is no commit to adopt or heal.
        assert_eq!(
            classify_resolve(facts(B, &[], ForkExpect::Advanced)),
            Err(ResolveRefusal::StaleRecord)
        );
        // TWO commits past base: C^ is D, not B — a stacked fork, never one commit.
        assert_eq!(
            classify_resolve(facts(C, &[D], ForkExpect::Advanced)),
            Err(ResolveRefusal::StaleRecord)
        );
        // A MERGE commit whose first parent is B is still not the retry signature.
        assert_eq!(
            classify_resolve(facts(C, &[B, D], ForkExpect::Advanced)),
            Err(ResolveRefusal::StaleRecord)
        );
        // A root commit (no parents) that is not base: not advanced from base.
        assert_eq!(
            classify_resolve(facts(C, &[], ForkExpect::Advanced)),
            Err(ResolveRefusal::StaleRecord)
        );
    }

    #[test]
    fn every_other_consistency_check_is_preserved_under_both_expectations() {
        // The expectation split changed ONLY the head check: live worktree, record
        // present, dir exists and branch/base resolve still decide first, identically.
        for expect in [ForkExpect::AtBase, ForkExpect::Advanced] {
            let (ok_head, parents): (&str, &[&str]) = match expect {
                ForkExpect::AtBase => (B, &[]),
                ForkExpect::Advanced => (C, &[B]),
            };

            // 0 live hits ⇒ unknown-agent (before anything else).
            let mut f = facts(ok_head, parents, expect);
            f.worktree_hits = 0;
            assert_eq!(classify_resolve(f), Err(ResolveRefusal::UnknownAgent));

            // > 1 live hits ⇒ ambiguous-agent (defensive).
            let mut f = facts(ok_head, parents, expect);
            f.worktree_hits = 2;
            assert_eq!(classify_resolve(f), Err(ResolveRefusal::AmbiguousAgent));

            // record absent / dir gone / branch unresolved / base unresolvable ⇒ stale.
            let mut f = facts(ok_head, parents, expect);
            f.record = None;
            assert_eq!(classify_resolve(f), Err(ResolveRefusal::StaleRecord));

            let mut f = facts(ok_head, parents, expect);
            f.dir_exists = false;
            assert_eq!(classify_resolve(f), Err(ResolveRefusal::StaleRecord));

            let mut f = facts(ok_head, parents, expect);
            f.branch_head = None;
            assert_eq!(classify_resolve(f), Err(ResolveRefusal::StaleRecord));

            let mut f = facts(ok_head, parents, expect);
            f.base_commit = None;
            assert_eq!(classify_resolve(f), Err(ResolveRefusal::StaleRecord));

            // ...and the consistent case resolves under its own declared expectation.
            assert!(classify_resolve(facts(ok_head, parents, expect)).is_ok());
        }
    }

    // --- VT-6: the durable fork binding ------------------------------------------------

    #[test]
    fn a_complete_binding_is_the_only_provable_one() {
        assert_eq!(
            require_binding(&record()),
            Ok(ForkBinding {
                slice: 228,
                phase: "PHASE-04".to_owned(),
            })
        );
        // HALF-bound is NOT bound: neither half alone names a funnel row.
        for half in [
            DispatchRecord {
                phase: None,
                ..record()
            },
            DispatchRecord {
                slice: None,
                ..record()
            },
            DispatchRecord {
                phase: Some(String::new()),
                ..record()
            },
            DispatchRecord {
                slice: None,
                phase: None,
                ..record()
            },
        ] {
            assert_eq!(
                require_binding(&half),
                Err(ResolveRefusal::UnprovableFork),
                "a half-bound record is unprovable, never half-proven"
            );
        }
        assert_eq!(ResolveRefusal::UnprovableFork.token(), "unprovable-fork");
    }

    #[test]
    fn the_binding_round_trips_through_provision_and_read() {
        // VT-6: slice+phase are snapshotted at fork creation and read back verbatim.
        let tmp = tempfile::tempdir().unwrap();
        let coord = tmp.path();
        let dir = coord.join(WORKTREES_SUBDIR).join("wk1");
        let binding = ForkBinding {
            slice: 228,
            phase: "PHASE-04".to_owned(),
        };
        provision_dispatch_record(coord, "wk1", B, &dir, "dispatch/wk1", Some(&binding)).unwrap();

        let read = read_record(coord, "wk1").expect("the record parses");
        assert_eq!(read.base, B, "base is still snapshotted at fork time");
        assert_eq!(read.binding(), Some(binding), "the binding round-trips");
    }

    #[test]
    fn an_unbound_record_still_parses_and_reports_itself_unbound() {
        // Backward compatibility: a record written before the binding existed is NOT a
        // parse failure — it is a fork whose phase cannot be proven.
        let tmp = tempfile::tempdir().unwrap();
        let coord = tmp.path();
        let dir = coord.join(WORKTREES_SUBDIR).join("wk1");
        provision_dispatch_record(coord, "wk1", B, &dir, "dispatch/wk1", None).unwrap();

        let raw = fs::read_to_string(record_path(coord, "wk1")).unwrap();
        assert!(
            !raw.contains("slice") && !raw.contains("phase"),
            "an unbound record writes neither half: {raw}"
        );
        let read = read_record(coord, "wk1").expect("an unbound record still parses");
        assert_eq!(read.binding(), None);
    }

    // --- the no-clobber bind belt ------------------------------------------------------

    #[test]
    fn bind_refuses_to_clobber_an_existing_binding_but_provision_restamps() {
        // BIND is the spawn-time door and keeps a no-clobber belt: a name that is
        // already bound is not re-bound (that is the clobber RV-304 F-2 contests).
        let tmp = tempfile::tempdir().unwrap();
        let coord = tmp.path();
        let dir = coord.join(WORKTREES_SUBDIR).join("wk1");
        bind_dispatch_record(coord, "wk1", B, &dir, "dispatch/wk1", None).unwrap();

        let err = bind_dispatch_record(coord, "wk1", C, &dir, "dispatch/wk1", None)
            .expect_err("a second bind for the same name must refuse");
        assert!(
            format!("{err:#}").contains("bind-refused"),
            "the refusal names itself: {err:#}"
        );
        assert_eq!(
            read_record(coord, "wk1").unwrap().base,
            B,
            "the live binding is untouched by the refused clobber"
        );

        // PROVISION is the re-stamp door and deliberately overwrites (the fixup-revive
        // path re-points `base` at `fork_tip`, design §5.5).
        provision_dispatch_record(coord, "wk1", C, &dir, "dispatch/wk1", None).unwrap();
        assert_eq!(read_record(coord, "wk1").unwrap().base, C);
    }

    // --- the `.worktrees/<name>` layout strip, both directions -------------------------

    #[test]
    fn coord_and_name_strips_the_layout_and_rejects_anything_else() {
        assert_eq!(
            coord_and_name(Path::new("/coord/.worktrees/wk1")),
            Some((PathBuf::from("/coord"), "wk1".to_owned()))
        );
        // Anchored on the `.worktrees` COMPONENT, never a substring.
        assert_eq!(coord_and_name(Path::new("/coord/not-worktrees/wk1")), None);
        assert_eq!(coord_and_name(Path::new("/coord/wk1")), None);
    }
}
