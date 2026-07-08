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

/// The per-worktree dispatch record (design §5.3) — the single source of truth
/// `worker_commit` (PHASE-02) consumes. Exactly five fields, snapshotted at the fork
/// point by the trusted create-fork hook and NEVER re-derived from mutable arming
/// state: `base` is B captured at fork time (supersedes the racy live-arming-slot read).
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
) -> anyhow::Result<()> {
    let record = DispatchRecord {
        name: name.to_string(),
        dir: dir.to_path_buf(),
        branch: branch.to_string(),
        base: base.to_string(),
        coord: coord.to_path_buf(),
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
    /// `dir` gone, branch unresolved, or `HEAD != record.base` (design §5.2 step 1d).
    StaleRecord,
}

impl ResolveRefusal {
    /// The distinct named token each refusal fails closed with.
    pub(crate) fn token(self) -> &'static str {
        match self {
            ResolveRefusal::UnknownAgent => "unknown-agent",
            ResolveRefusal::AmbiguousAgent => "ambiguous-agent",
            ResolveRefusal::StaleRecord => "stale-record",
        }
    }
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
}

/// PURE resolver classifier (no git / disk — ADR-001 leaf). Mirror of `classify_gc`:
/// 0 hits ⇒ `unknown-agent`; `> 1` ⇒ `ambiguous-agent`; a single hit whose record is
/// missing, whose `dir` is gone, whose branch is unresolved, or whose `HEAD != base`
/// ⇒ `stale-record`; otherwise the record (design §5.2 step 1d, EX-3).
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
    // `base_commit` None (record base unresolvable) also lands here — Some(head) != None.
    if facts.base_commit.as_deref() != Some(head.as_str()) {
        return Err(ResolveRefusal::StaleRecord);
    }
    Ok(record)
}

/// Recover the coordination root from a worker worktree dir by layout-stripping the
/// `<WORKTREES_SUBDIR>/<name>` shape create-fork lays down (design §5.3 — one owner of
/// the `.worktrees/<name>` layout). `None` when the dir does not match the layout.
fn coord_from_worktree_dir(dir: &Path, name: &str) -> Option<PathBuf> {
    if dir.file_name()?.to_str()? != name {
        return None;
    }
    let worktrees = dir.parent()?;
    if worktrees.file_name()?.to_str()? != WORKTREES_SUBDIR {
        return None;
    }
    Some(worktrees.parent()?.to_path_buf())
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
/// 4. gather `record.dir` existence, the branch tip, and the record's base commit,
/// 5. classify.
pub(crate) fn resolve_agent(root: &Path, agent: &str) -> Result<DispatchRecord, ResolveRefusal> {
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

    classify_resolve(ResolveFacts {
        worktree_hits,
        record,
        dir_exists,
        branch_head,
        base_commit,
    })
}
