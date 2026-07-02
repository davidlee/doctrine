// SPDX-License-Identifier: GPL-3.0-only
//! Worktree inventory + provenance (SL-190 PHASE-05) — `doctrine worktree list`.
//!
//! ADR-001 tier: ENGINE (mirrors `worktree::gc`). The pure core
//! ([`classify_worktree`] → [`WorktreeRole`]) takes gathered FACTS
//! (`is_primary`, the branch name, the [`Cause`] marker signal) and no
//! disk/git/clock/rng — the CLAUDE.md pure/imperative split. The impure shell
//! ([`run_list`]) gathers those facts: it enumerates worktrees via
//! [`crate::git::list_worktrees`], reads each row's marker off disk, and computes
//! the role-conditional `landed` verdict by REUSING the PHASE-04 shared oracle
//! [`crate::worktree::gc::landed_against`] against the row-appropriate target
//! (fail-soft: a missing/unresolvable target reads `unknown`, never a hard error —
//! the caller owns the missing-target tri-state the oracle deferred).

use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::git::{self, WorktreeRecord};
use crate::root;

use super::gc::landed_against;
use super::marker::{Cause, describe_mode, marker_present};

/// The role of a worktree in the dispatch topology — the pure classification of a
/// row (design: inventory provenance). `Coordination` is the isolated funnel tree
/// (`dispatch/<slice>`); `WorkerFork` is a spawned worker (`dispatch/agent-*` or a
/// worktree bearing the worker marker); `Benign` is any other linked worktree (a
/// hand-made `/worktree` tree, a detached candidate, …); `Primary` is the main tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorktreeRole {
    /// The repository's primary (main) worktree.
    Primary,
    /// An isolated dispatch coordination tree (`dispatch/<slice>`).
    Coordination,
    /// A spawned dispatch worker fork (`dispatch/agent-*` or marker-bearing).
    WorkerFork,
    /// Any other linked worktree — no dispatch provenance.
    Benign,
}

impl WorktreeRole {
    /// The stable token for the table / `--json` cell (goldens key on these).
    pub(crate) fn token(self) -> &'static str {
        match self {
            WorktreeRole::Primary => "primary",
            WorktreeRole::Coordination => "coordination",
            WorktreeRole::WorkerFork => "worker-fork",
            WorktreeRole::Benign => "benign",
        }
    }
}

/// PURE role classifier (no git / disk / env — ADR-001 engine core, the
/// pure/imperative split). Deduces the [`WorktreeRole`] from the gathered facts:
///
/// * `is_primary` — the row is git's first (main) worktree ⇒ [`WorktreeRole::Primary`]
///   (a marker on the primary is inert, mirroring `describe_mode`, so this wins).
/// * `marker_cause` — a worker marker present ([`Cause::Marker`]/[`Cause::Both`]) is
///   the strongest worker signal ⇒ [`WorktreeRole::WorkerFork`].
/// * `branch` — a `dispatch/<name>` branch: `agent-*` ⇒ worker-fork; an all-numeric
///   `<slice>` suffix ⇒ [`WorktreeRole::Coordination`]. Anything else ⇒
///   [`WorktreeRole::Benign`].
pub(crate) fn classify_worktree(
    is_primary: bool,
    branch: Option<&str>,
    marker_cause: Cause,
) -> WorktreeRole {
    if is_primary {
        return WorktreeRole::Primary;
    }
    if matches!(marker_cause, Cause::Marker | Cause::Both) {
        return WorktreeRole::WorkerFork;
    }
    if let Some(suffix) = dispatch_suffix(branch) {
        if suffix.starts_with("agent-") {
            return WorktreeRole::WorkerFork;
        }
        if !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_digit()) {
            return WorktreeRole::Coordination;
        }
    }
    WorktreeRole::Benign
}

/// The `dispatch/<suffix>` tail of a branch ref, or `None` when the branch is not a
/// dispatch branch. Strips the `refs/heads/` prefix (porcelain form) then the
/// `dispatch/` marker. PURE.
fn dispatch_suffix(branch: Option<&str>) -> Option<&str> {
    branch
        .map(|b| b.strip_prefix("refs/heads/").unwrap_or(b))
        .and_then(|b| b.strip_prefix("dispatch/"))
}

/// The row-appropriate `landed` verdict — a TRI-STATE (`+`n/a) the shell computes
/// per row (the PHASE-04 oracle stays a clean total `bool`; this caller owns the
/// missing-target case).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LandedCell {
    /// The fork has provably landed against its role-conditional target.
    Landed,
    /// The fork has NOT landed against its target.
    NotLanded,
    /// The target/fork was missing or the oracle could not decide — fail-soft.
    Unknown,
    /// No landing target applies (primary / benign rows).
    NotApplicable,
}

impl LandedCell {
    fn token(self) -> &'static str {
        match self {
            LandedCell::Landed => "landed",
            LandedCell::NotLanded => "unlanded",
            LandedCell::Unknown => "unknown",
            LandedCell::NotApplicable => "n/a",
        }
    }
}

/// One fully-resolved inventory row — the record plus its derived provenance.
struct InventoryRow {
    path: PathBuf,
    role: WorktreeRole,
    slice: Option<u32>,
    branch: Option<String>,
    head: Option<String>,
    marker: bool,
    live: bool,
    landed: LandedCell,
}

/// `doctrine worktree list [--slice N] [--json] [--no-landed]` — the worktree
/// inventory verb (SL-190 PHASE-05, EX-3). Enumerates every linked worktree, prints
/// `path·role·slice·branch·head·marker·live?·landed`, filtered by `--slice` when
/// given, as a table or (`--json`) a structured array. The `landed` column is ON by
/// default (`--no-landed` suppresses it) and is role-conditional + fail-soft (see
/// [`landed_cell`]). Read-only — runs at the coordination root, safe under worker
/// mode.
pub(crate) fn run_list(
    path: Option<PathBuf>,
    slice_filter: Option<u32>,
    json: bool,
    no_landed: bool,
) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    let root = std::fs::canonicalize(&root)
        .with_context(|| format!("canonicalize root {}", root.display()))?;

    let records = git::list_worktrees(&root).context("enumerate worktrees")?;
    let rows: Vec<InventoryRow> = records
        .iter()
        .enumerate()
        .map(|(i, rec)| resolve_row(&root, rec, i == 0, no_landed))
        .filter(|row| slice_filter.is_none_or(|n| row.slice == Some(n)))
        .collect();

    if json {
        print_json(&rows, no_landed)
    } else {
        print_table(&rows, no_landed)
    }
}

/// Gather a record's derived facts (impure: disk marker read + the landed oracle).
fn resolve_row(
    root: &Path,
    rec: &WorktreeRecord,
    is_primary: bool,
    no_landed: bool,
) -> InventoryRow {
    let branch = rec.branch.as_deref();
    // The row's marker signal, via the SHARED marker verdict (env is irrelevant to
    // another worktree's provenance — only the on-disk marker of a linked tree is).
    let marker = !is_primary && marker_present(&rec.path);
    let cause = describe_mode(!is_primary, marker, false).cause;
    let role = classify_worktree(is_primary, branch, cause);
    let slice = slice_of(&rec.path, branch);
    let landed = if no_landed {
        LandedCell::NotApplicable
    } else {
        landed_cell(root, role, rec, slice)
    };
    InventoryRow {
        path: rec.path.clone(),
        role,
        slice,
        branch: rec.branch.clone(),
        head: rec.head.clone(),
        marker,
        live: rec.path.exists() && !rec.prunable,
        landed,
    }
}

/// The slice a row belongs to: a coordination branch's numeric `dispatch/<NNN>`
/// suffix, else the `SL-<N>` segment of a worker fork's nested path
/// (`.dispatch/SL-<N>/.worktrees/agent-*`). `None` when neither applies. PURE.
fn slice_of(path: &Path, branch: Option<&str>) -> Option<u32> {
    if let Some(n) = dispatch_suffix(branch).and_then(|s| s.parse::<u32>().ok()) {
        return Some(n);
    }
    path.components().find_map(|c| {
        c.as_os_str()
            .to_str()
            .and_then(|s| s.strip_prefix("SL-"))
            .and_then(|d| d.parse::<u32>().ok())
    })
}

/// The role-conditional, FAIL-SOFT `landed` verdict for a row (EX-3):
/// * worker-fork → its own branch landed against the coordination ref
///   `refs/heads/dispatch/<slice>`;
/// * coordination → its branch landed against trunk (the deliver-to base);
/// * primary / benign → [`LandedCell::NotApplicable`].
///
/// A missing/unresolvable target or fork, or an oracle error, degrades to
/// [`LandedCell::Unknown`] — NEVER a hard error (the caller owns the missing-target
/// tri-state PHASE-04 deferred to it).
fn landed_cell(
    root: &Path,
    role: WorktreeRole,
    rec: &WorktreeRecord,
    slice: Option<u32>,
) -> LandedCell {
    let (target, fork) = match role {
        WorktreeRole::Primary | WorktreeRole::Benign => return LandedCell::NotApplicable,
        WorktreeRole::WorkerFork => {
            let Some(n) = slice else {
                return LandedCell::Unknown;
            };
            (format!("refs/heads/dispatch/{n:03}"), rec.branch.clone())
        }
        WorktreeRole::Coordination => match git::trunk_commit(root).ok().flatten() {
            Some(trunk) => (trunk, rec.branch.clone()),
            None => return LandedCell::Unknown,
        },
    };
    let Some(fork) = fork else {
        return LandedCell::Unknown;
    };
    // Fail-soft: both refs must resolve to a commit before the oracle runs (a bad
    // ref would make `git cherry` error — that is the missing-target case).
    let (Some(target), Some(fork)) = (resolve_commit(root, &target), resolve_commit(root, &fork))
    else {
        return LandedCell::Unknown;
    };
    match landed_against(root, &target, &fork) {
        Ok(true) => LandedCell::Landed,
        Ok(false) => LandedCell::NotLanded,
        Err(_) => LandedCell::Unknown,
    }
}

/// Resolve `refspec` to a commit sha, or `None` if it does not resolve (fail-soft).
fn resolve_commit(root: &Path, refspec: &str) -> Option<String> {
    git::git_opt(
        root,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{refspec}^{{commit}}"),
        ],
    )
    .ok()
    .flatten()
}

/// The `refs/heads/`-stripped branch label for a row, or a `(detached)` placeholder.
fn branch_label(branch: Option<&str>) -> String {
    match branch {
        Some(b) => b.strip_prefix("refs/heads/").unwrap_or(b).to_string(),
        None => "(detached)".to_string(),
    }
}

/// The short head sha (first 12 chars), or `-` for a bare block.
fn head_label(head: Option<&str>) -> String {
    match head {
        Some(h) => h.chars().take(12).collect(),
        None => "-".to_string(),
    }
}

fn yes_no(v: bool) -> &'static str {
    if v { "yes" } else { "no" }
}

fn slice_label(slice: Option<u32>) -> String {
    slice.map_or_else(|| "-".to_string(), |n| n.to_string())
}

/// Render the inventory as a padded table (EX-3). `landed` is the last column,
/// dropped when `no_landed`.
fn print_table(rows: &[InventoryRow], no_landed: bool) -> anyhow::Result<()> {
    let mut header: Vec<&str> = vec!["path", "role", "slice", "branch", "head", "marker", "live?"];
    if !no_landed {
        header.push("landed");
    }
    let mut table: Vec<Vec<String>> = vec![header.iter().map(|s| (*s).to_string()).collect()];
    for row in rows {
        let mut cells = vec![
            row.path.display().to_string(),
            row.role.token().to_string(),
            slice_label(row.slice),
            branch_label(row.branch.as_deref()),
            head_label(row.head.as_deref()),
            yes_no(row.marker).to_string(),
            yes_no(row.live).to_string(),
        ];
        if !no_landed {
            cells.push(row.landed.token().to_string());
        }
        table.push(cells);
    }

    let cols = header.len();
    let mut widths = vec![0usize; cols];
    for row in &table {
        for (width, cell) in widths.iter_mut().zip(row.iter()) {
            *width = (*width).max(cell.len());
        }
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();
    for row in &table {
        let line = row
            .iter()
            .zip(widths.iter())
            .map(|(cell, width)| format!("{cell:<width$}", width = *width))
            .collect::<Vec<_>>()
            .join("  ");
        writeln!(out, "{}", line.trim_end())?;
    }
    Ok(())
}

/// Render the inventory as a structured JSON array (EX-3, `--json`). `landed` is
/// omitted per-object when `no_landed`.
fn print_json(rows: &[InventoryRow], no_landed: bool) -> anyhow::Result<()> {
    let items: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            let mut obj = serde_json::json!({
                "path": row.path.display().to_string(),
                "role": row.role.token(),
                "slice": row.slice,
                "branch": row.branch,
                "head": row.head,
                "marker": row.marker,
                "live": row.live,
            });
            if !no_landed && let Some(map) = obj.as_object_mut() {
                map.insert(
                    "landed".to_string(),
                    serde_json::Value::from(row.landed.token()),
                );
            }
            obj
        })
        .collect();
    let payload = serde_json::Value::Array(items);
    let text = serde_json::to_string_pretty(&payload).context("serialize worktree list json")?;
    writeln!(io::stdout(), "{text}")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests — VT-1: the pure classifier over each (is_primary, branch, marker) combo.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_worktree_over_each_combination() {
        // is_primary wins over everything (a marker on the primary is inert).
        assert_eq!(
            classify_worktree(true, Some("refs/heads/edge"), Cause::None),
            WorktreeRole::Primary
        );
        assert_eq!(
            classify_worktree(true, Some("refs/heads/dispatch/190"), Cause::Marker),
            WorktreeRole::Primary,
            "the primary is primary even bearing a marker"
        );

        // A worker marker is the strongest worker signal.
        assert_eq!(
            classify_worktree(false, None, Cause::Marker),
            WorktreeRole::WorkerFork
        );
        assert_eq!(
            classify_worktree(false, Some("refs/heads/anything"), Cause::Both),
            WorktreeRole::WorkerFork
        );

        // Coordination: a numeric dispatch/<slice> suffix, no marker.
        assert_eq!(
            classify_worktree(false, Some("refs/heads/dispatch/190"), Cause::None),
            WorktreeRole::Coordination
        );
        // The bare (non-`refs/heads/`) form is accepted too.
        assert_eq!(
            classify_worktree(false, Some("dispatch/007"), Cause::None),
            WorktreeRole::Coordination
        );

        // Worker fork: a dispatch/agent-* branch, no marker.
        assert_eq!(
            classify_worktree(
                false,
                Some("refs/heads/dispatch/agent-ab9f5d9e"),
                Cause::None
            ),
            WorktreeRole::WorkerFork
        );

        // Benign: a hand-made worktree branch, a detached tree, an env-only cause.
        assert_eq!(
            classify_worktree(false, Some("refs/heads/w/SL-186-p02"), Cause::None),
            WorktreeRole::Benign
        );
        assert_eq!(
            classify_worktree(false, None, Cause::None),
            WorktreeRole::Benign
        );
        assert_eq!(
            classify_worktree(false, Some("refs/heads/dispatch/nonnumeric"), Cause::None),
            WorktreeRole::Benign,
            "a non-numeric, non-agent dispatch suffix is not coordination"
        );
        assert_eq!(
            classify_worktree(false, None, Cause::Env),
            WorktreeRole::Benign,
            "env alone is not a per-row worker-fork signal for inventory"
        );
    }

    #[test]
    fn role_tokens_are_stable() {
        assert_eq!(WorktreeRole::Primary.token(), "primary");
        assert_eq!(WorktreeRole::Coordination.token(), "coordination");
        assert_eq!(WorktreeRole::WorkerFork.token(), "worker-fork");
        assert_eq!(WorktreeRole::Benign.token(), "benign");
    }

    #[test]
    fn dispatch_suffix_strips_both_prefixes() {
        assert_eq!(
            dispatch_suffix(Some("refs/heads/dispatch/190")),
            Some("190")
        );
        assert_eq!(dispatch_suffix(Some("dispatch/agent-x")), Some("agent-x"));
        assert_eq!(dispatch_suffix(Some("refs/heads/main")), None);
        assert_eq!(dispatch_suffix(None), None);
    }

    #[test]
    fn slice_of_reads_branch_then_path() {
        // Coordination: the numeric branch suffix.
        assert_eq!(
            slice_of(
                Path::new("/x/.dispatch/SL-190"),
                Some("refs/heads/dispatch/190")
            ),
            Some(190)
        );
        // Worker fork: the SL-<N> path segment (agent branch has no numeric suffix).
        assert_eq!(
            slice_of(
                Path::new("/x/.dispatch/SL-190/.worktrees/agent-abc"),
                Some("refs/heads/dispatch/agent-abc")
            ),
            Some(190)
        );
        // Neither: no dispatch branch, no SL- segment.
        assert_eq!(
            slice_of(Path::new("/x/plain"), Some("refs/heads/main")),
            None
        );
    }

    #[test]
    fn landed_cell_tokens_are_stable() {
        assert_eq!(LandedCell::Landed.token(), "landed");
        assert_eq!(LandedCell::NotLanded.token(), "unlanded");
        assert_eq!(LandedCell::Unknown.token(), "unknown");
        assert_eq!(LandedCell::NotApplicable.token(), "n/a");
    }
}
