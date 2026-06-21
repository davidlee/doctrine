// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine needs` / `doctrine after` — dep/seq verbs (SL-060 §5.4).
//! SL-129: uses `entity::id_path`

use std::path::PathBuf;

/// The work-like membership predicate (SL-060 §5.4, SL-066 §PHASE-04) — the ONE
/// widen-later guard. Work-like = { slice } ∪ { the 5 backlog kinds } ∪ { revision }.
/// Both the dep/seq-authoring SRC set and the admissible TGT set are this same
/// membership (a slice / backlog item / revision may author dep/seq, and may only
/// depend/sequence on another piece of work). REV is admitted as BOTH source and
/// target: a slice or backlog item may `needs`/`after` a REV-NNN, and a REV may
/// itself `needs`/`after` a work item (the IDE-010 payoff). Governance docs
/// (spec/ADR/POL/STD) stay EXCLUDED — depending on governance routes THROUGH a
/// Revision, never the evergreen doc (the SL-060 invariant). A future phase that
/// allows cross-tier dep/seq deletes just this predicate (and its refusal tests).
pub(crate) fn is_work_like(kind: &'static crate::entity::Kind) -> bool {
    matches!(
        kind.prefix,
        "SL" | "ISS" | "IMP" | "CHR" | "RSK" | "IDE" | "REV"
    )
}

/// Resolve a dep/seq source to its TOML path. Validates: canonical-ref parse,
/// work-like kind (slice or backlog). Returns the resolved path.
fn resolve_dep_seq_src_path(root: &std::path::Path, source: &str) -> anyhow::Result<PathBuf> {
    let (skref, sid) = crate::integrity::parse_canonical_ref(source)?;
    anyhow::ensure!(
        is_work_like(skref.kind),
        "`{source}` is a {} entity, which cannot author needs/after — only a slice or a backlog item (issue/improvement/chore/risk/idea) carries dep/seq",
        skref.kind.prefix
    );
    Ok(crate::entity::id_path(
        root,
        skref.kind,
        sid,
        crate::entity::Ext::Toml,
    ))
}

/// Resolve a generic dep/seq `(SRC, TGT)` pair against the author-time gate (§5.4),
/// returning SRC's `slice-NNN.toml`-shaped path ready for the leaf write. Rides the
/// SAME cross-kind canonical-ref seam as `link` (`integrity::parse_canonical_ref` +
/// the `KindRef` `(dir, stem)` path map) — no new resolver. The three refusals, each
/// a clear, specific message:
///   1. SRC must resolve AND be a dep/seq-authoring (work-like) kind.
///   2. TGT must resolve on disk (free-text / dangling refused) AND be work-like.
///   3. self-edge (SRC == TGT) refused.
fn resolve_dep_seq_src(
    root: &std::path::Path,
    source: &str,
    target: &str,
) -> anyhow::Result<PathBuf> {
    let toml_path = resolve_dep_seq_src_path(root, source)?;
    let (skref, sid) = crate::integrity::parse_canonical_ref(source)?;
    // TGT must resolve on disk — a free-text or dangling target is refused here
    // (never write an edge to a non-entity). `parse_canonical_ref` first so a
    // free-text target surfaces the canonical-ref shape error, then a dir probe.
    let (tkref, tid) = crate::integrity::parse_canonical_ref(target)?;
    crate::integrity::ensure_ref_resolves(root, target)?;
    anyhow::ensure!(
        is_work_like(tkref.kind),
        "`{target}` is a {} entity — needs/after may only target work (a slice or a backlog item); cross-tier dep/seq is not yet allowed",
        tkref.kind.prefix
    );
    anyhow::ensure!(
        !(skref.kind.prefix == tkref.kind.prefix && sid == tid),
        "a {source} edge to itself is not a dependency — self-edges are refused"
    );
    Ok(toml_path)
}

/// `doctrine needs <SRC> <TGT>` (SL-060 §5.4) — append TGT to SRC's `needs` axis.
/// Generic cross-kind: the author-time work-like gate ([`resolve_dep_seq_src`]) then
/// the shared leaf `dep_seq::append`. NO author-time cycle check (deferred to read
/// time by design — the cross-kind cycle oracle is a later phase).
pub(crate) fn run_needs_edge(
    path: Option<PathBuf>,
    source: &str,
    target: &str,
) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let toml_path = resolve_dep_seq_src(&root, source, target)?;
    crate::dep_seq::append(
        &toml_path,
        &crate::dep_seq::RelEdit::Needs(&[target.to_string()]),
    )?;
    writeln!(std::io::stdout(), "{source} needs {target}")?;
    Ok(())
}

/// `doctrine after <SRC> <TGT> [--rank N]` (SL-060 §5.4) — append `{ to, rank }` to
/// SRC's `after` axis through the same gate + leaf. Rank default 0.
pub(crate) fn run_after_edge(
    path: Option<PathBuf>,
    source: &str,
    target: &str,
    rank: i32,
) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let toml_path = resolve_dep_seq_src(&root, source, target)?;
    crate::dep_seq::append(
        &toml_path,
        &crate::dep_seq::RelEdit::After { to: target, rank },
    )?;
    let suffix = if rank == 0 {
        String::new()
    } else {
        format!(" (rank {rank})")
    };
    writeln!(std::io::stdout(), "{source} after {target}{suffix}")?;
    Ok(())
}

/// `doctrine after <SRC> <TGT> --remove [--rank N]`
pub(crate) fn run_after_remove(
    path: Option<PathBuf>,
    source: &str,
    target: &str,
    rank: i32,
) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let toml_path = resolve_dep_seq_src(&root, source, target)?;
    let ceiling = if rank == 0 { None } else { Some(rank) };
    let removed = crate::dep_seq::remove(&toml_path, target, ceiling)?;
    if removed == 0 {
        anyhow::bail!("{source} has no after edge to {target}");
    }
    writeln!(
        std::io::stdout(),
        "{source} after {target} removed ({} edge{})",
        removed,
        if removed == 1 { "" } else { "s" }
    )?;
    Ok(())
}

/// `doctrine after <SRC> --prune` (SL-105 PHASE-03) — probe every `after` target
/// of SRC for dangling edges (absent or terminal target) and remove them. Reads
/// the `DepSeq` ONCE before any modifications (collecting dangling targets), then
/// removes in a second pass using the shared `dep_seq::remove` leaf.
pub(crate) fn run_after_prune(path: Option<PathBuf>, source: &str) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let toml_path = resolve_dep_seq_src_path(&root, source)?;

    // 1. Read DepSeq
    let ds = crate::dep_seq::read(&toml_path)?;

    // 2. Probe each after-edge target: absent (dir missing) or terminal (resolved/closed) → dangling
    let mut dropped: Vec<(String, i32, String)> = Vec::new();
    let mut to_drop: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for edge in &ds.after {
        let is_dangling = match crate::integrity::parse_canonical_ref(&edge.to) {
            Ok((kref, tid)) => {
                let target_path =
                    crate::entity::id_path(&root, kref.kind, tid, crate::entity::Ext::Toml);
                if target_path.exists() {
                    let body = std::fs::read_to_string(&target_path).unwrap_or_default();
                    let val: toml::Value = match toml::from_str(&body) {
                        Ok(v) => v,
                        Err(_) => toml::Value::Table(toml::Table::new()),
                    };
                    let status = val.get("status").and_then(|s| s.as_str()).unwrap_or("");
                    status == "resolved" || status == "closed"
                } else {
                    true
                }
            }
            Err(_) => true,
        };

        if is_dangling {
            let reason = match crate::integrity::parse_canonical_ref(&edge.to) {
                Ok((kref2, tid2)) => {
                    let target_path =
                        crate::entity::id_path(&root, kref2.kind, tid2, crate::entity::Ext::Toml);
                    if target_path.exists() {
                        let body = std::fs::read_to_string(&target_path).unwrap_or_default();
                        let val: toml::Value = match toml::from_str(&body) {
                            Ok(v) => v,
                            Err(_) => toml::Value::Table(toml::Table::new()),
                        };
                        let status = val.get("status").and_then(|s| s.as_str()).unwrap_or("");
                        let resolution =
                            val.get("resolution").and_then(|s| s.as_str()).unwrap_or("");
                        if resolution.is_empty() {
                            status.to_string()
                        } else {
                            format!("{status}/{resolution}")
                        }
                    } else {
                        "absent".to_string()
                    }
                }
                Err(_) => "absent (unparseable ref)".to_string(),
            };
            dropped.push((edge.to.clone(), edge.rank, reason));
            to_drop.insert(edge.to.clone());
        }
    }

    if dropped.is_empty() {
        writeln!(std::io::stdout(), "{source}: nothing to prune")?;
        return Ok(());
    }

    // 3. Remove all edges per unique dangling target (one pass each) via shared leaf
    for target in &to_drop {
        // `None` ceiling → remove every edge matching the target wildcard
        let _ = crate::dep_seq::remove(&toml_path, target, None)?;
    }

    // 4. Report dropped edges
    for (target, rank, reason) in &dropped {
        writeln!(
            std::io::stdout(),
            "{source} after {target} (rank {rank}) dropped (dangling: {reason})"
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrity;
    use crate::slice;

    /// SL-060 / SL-066 §PHASE-04: the work-like membership predicate is the ONE
    /// widen-later guard — exactly { slice } ∪ { the 5 backlog kinds } ∪ { revision },
    /// every other admitted kind refused. REV joins as both dep/seq source and target
    /// (the IDE-010 payoff); governance docs stay off the allowlist (SL-060 invariant).
    #[test]
    fn is_work_like_is_exactly_slice_plus_backlog_plus_revision() {
        // The work-like set: slice + the five backlog kinds + revision.
        assert!(is_work_like(&slice::SLICE_KIND));
        for k in integrity::KINDS
            .iter()
            .filter(|k| matches!(k.kind.prefix, "ISS" | "IMP" | "CHR" | "RSK" | "IDE" | "REV"))
        {
            assert!(is_work_like(k.kind), "{} is work-like", k.kind.prefix);
        }
        // Every OTHER admitted kind in the corpus table is refused (gov / spec / req /
        // review / reconciliation / knowledge) — the closed allowlist.
        for k in integrity::KINDS.iter().filter(|k| {
            !matches!(
                k.kind.prefix,
                "SL" | "ISS" | "IMP" | "CHR" | "RSK" | "IDE" | "REV"
            )
        }) {
            assert!(
                !is_work_like(k.kind),
                "{} must NOT be work-like (off the allowlist)",
                k.kind.prefix
            );
        }
    }
}
