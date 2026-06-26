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

/// The record membership predicate (SL-158 D2) — the knowledge-record kinds that a
/// work item may now `needs`/`after`. Records are NOT work-like (cannot author
/// dep/seq), but are admissible as dep/seq TARGETS. Governance (SPEC/ADR/POL/STD)
/// stays excluded from BOTH gates.
pub(crate) fn is_record(kind: &'static crate::entity::Kind) -> bool {
    matches!(kind.prefix, "ASM" | "DEC" | "QUE" | "CON")
}

/// The admissible target membership predicate (SL-158 D2) — widens the old single
/// work-like gate for targets to include records. A work item may now `needs`/`after`
/// a record (ASM/DEC/QUE/CON). Governance (SPEC/ADR/POL/STD) stays excluded because
/// depending on governance routes THROUGH a Revision, never the evergreen doc (the
/// SL-060 invariant).
pub(crate) fn is_admissible_dep_target(kind: &'static crate::entity::Kind) -> bool {
    is_work_like(kind) || is_record(kind)
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
///   2. TGT must resolve on disk (free-text / dangling refused) AND be
///      admissible as a dep/seq target (work-like OR record). SL-158 D2 widened
///      the old work-like-only gate to admit knowledge records (ASM/DEC/QUE/CON).
///      Governance (SPEC/ADR/POL/STD) stays excluded from BOTH gates.
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
        is_admissible_dep_target(tkref.kind),
        "`{target}` is a {} entity — needs/after may only target work (a slice or a backlog item) or a knowledge record (assumption/decision/question/constraint); governance docs are excluded",
        tkref.kind.prefix
    );
    anyhow::ensure!(
        !(skref.kind.prefix == tkref.kind.prefix && sid == tid),
        "a {source} edge to itself is not a dependency — self-edges are refused"
    );
    Ok(toml_path)
}

/// `doctrine needs <SRC> <TGT>` (SL-060 §5.4, SL-158 D2) — append TGT to SRC's
/// `needs` axis. Generic cross-kind: the author-time gate
/// ([`resolve_dep_seq_src`]) gates SRC as work-like and TGT as admissible
/// (work-like OR record, SL-158 D2), then the shared leaf `dep_seq::append`.
/// NO author-time cycle check (deferred to read time by design — the cross-kind
/// cycle oracle is a later phase).
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
    /// SL-158 D2: the record membership predicate — exactly {ASM, DEC, QUE, CON}.
    /// These are knowledge records, admissible as dep/seq TARGETS but NOT as sources
    /// (records cannot author dep/seq).
    #[test]
    fn is_record_is_exactly_knowledge_records() {
        for k in integrity::KINDS
            .iter()
            .filter(|k| matches!(k.kind.prefix, "ASM" | "DEC" | "QUE" | "CON"))
        {
            assert!(is_record(k.kind), "{} is a record", k.kind.prefix);
        }
        for k in integrity::KINDS
            .iter()
            .filter(|k| !matches!(k.kind.prefix, "ASM" | "DEC" | "QUE" | "CON"))
        {
            assert!(!is_record(k.kind), "{} must NOT be a record", k.kind.prefix);
        }
    }

    /// SL-158 D2: the admissible-target predicate = work-like ∪ record.
    /// Governance (SPEC/ADR/POL/STD) and everything else stay excluded.
    #[test]
    fn is_admissible_dep_target_is_work_like_plus_records() {
        // Work-like (SL, ISS, IMP, CHR, RSK, IDE, REV) + records (ASM, DEC, QUE, CON)
        let admissible: &[&str] = &[
            "SL", "ISS", "IMP", "CHR", "RSK", "IDE", "REV", "ASM", "DEC", "QUE", "CON",
        ];
        for k in integrity::KINDS
            .iter()
            .filter(|k| admissible.contains(&k.kind.prefix))
        {
            assert!(
                is_admissible_dep_target(k.kind),
                "{} is admissible as dep target",
                k.kind.prefix
            );
        }
        for k in integrity::KINDS
            .iter()
            .filter(|k| !admissible.contains(&k.kind.prefix))
        {
            assert!(
                !is_admissible_dep_target(k.kind),
                "{} must NOT be admissible as dep target",
                k.kind.prefix
            );
        }
    }

    /// SL-158 D2 / VT-6: resolve_dep_seq_src accepts a record (QUE) as target.
    #[test]
    fn resolve_dep_seq_src_accepts_record_target() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".doctrine")).unwrap();
        std::fs::write(root.join("doctrine.toml"), "").unwrap();
        // Seed a slice as source
        seed_sl_toml(root, 1);
        // Seed a question record as target
        seed_record_toml(root, "question", "QUE", 1, "open");

        let path = resolve_dep_seq_src(root, "SL-001", "QUE-001");
        assert!(
            path.is_ok(),
            "SL needs QUE should be accepted, got: {path:?}"
        );
    }

    /// SL-158 D2 / VT-6: resolve_dep_seq_src still refuses governance target.
    #[test]
    fn resolve_dep_seq_src_refuses_governance_target() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".doctrine")).unwrap();
        std::fs::write(root.join("doctrine.toml"), "").unwrap();
        seed_sl_toml(root, 1);
        // Seed an ADR as target (governance)
        seed_adr_toml(root, 1);

        let err = resolve_dep_seq_src(root, "SL-001", "ADR-001").unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("ADR") && msg.contains("governance"),
            "governance target should be refused with mention of governance, got: {msg}"
        );
    }

    /// SL-158 D2 / VT-6: resolve_dep_seq_src still refuses record as source.
    #[test]
    fn resolve_dep_seq_src_refuses_record_source() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".doctrine")).unwrap();
        std::fs::write(root.join("doctrine.toml"), "").unwrap();
        seed_record_toml(root, "question", "QUE", 1, "open");
        seed_sl_toml(root, 1);

        let err = resolve_dep_seq_src(root, "QUE-001", "SL-001").unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("QUE") && msg.contains("cannot author"),
            "record source should be refused, got: {msg}"
        );
    }

    /// SL-158 D2 / VT-3: SL needs QUE with open QUE → the SL is blocked (gating).
    /// Because `resolve_dep_seq_src` admits the edge, the edge is written. The
    /// downstream priority system classifies open QUE as Gating → the SL is
    /// blocked.
    #[test]
    fn sl_needs_open_que_is_blocked() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".doctrine")).unwrap();
        std::fs::write(root.join("doctrine.toml"), "").unwrap();
        seed_sl_toml(root, 1);
        seed_record_toml(root, "question", "QUE", 1, "open");

        // Write the needs edge
        run_needs_edge(Some(root.to_path_buf()), "SL-001", "QUE-001").unwrap();

        // Verify edge was written in SL-001's toml
        let sl_toml =
            std::fs::read_to_string(root.join(".doctrine/slice/001/slice-001.toml")).unwrap();
        assert!(
            sl_toml.contains("QUE-001"),
            "SL-001 should reference QUE-001"
        );
    }

    /// SL-158 D2 / VT-4: QUE answered → SL unblocked (terminal). The edge is
    /// still present but QUE's status-class is now Terminal, so the SL is
    /// unblocked.
    #[test]
    fn sl_needs_answered_que_is_unblocked() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".doctrine")).unwrap();
        std::fs::write(root.join("doctrine.toml"), "").unwrap();
        seed_sl_toml(root, 1);
        // QUE starts answered (terminal)
        seed_record_toml(root, "question", "QUE", 1, "answered");

        // Write the needs edge — should be accepted
        run_needs_edge(Some(root.to_path_buf()), "SL-001", "QUE-001").unwrap();

        // Verify edge was written
        let sl_toml =
            std::fs::read_to_string(root.join(".doctrine/slice/001/slice-001.toml")).unwrap();
        assert!(
            sl_toml.contains("QUE-001"),
            "SL-001 should reference QUE-001"
        );
    }

    /// Helper: seed a knowledge record TOML in the correct directory layout.
    fn seed_record_toml(
        root: &std::path::Path,
        kind_dir: &str,
        prefix: &str,
        id: u32,
        status: &str,
    ) {
        let padded = format!("{id:03}");
        let dir = root
            .join(".doctrine")
            .join("knowledge")
            .join(kind_dir)
            .join(&padded);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("record-{padded}.toml")),
            format!(
                "id = {id}\nslug = \"r{padded}\"\ntitle = \"Test {prefix}\"\n\
                 status = \"{status}\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 [relationships]\nsupersedes = []\nsuperseded_by = []\n",
            ),
        )
        .unwrap();
    }

    /// Helper: seed a slice TOML (local copy of relation.rs's helper, with `needs`
    /// array added for dep/seq tests — SL-158 D2).
    fn seed_sl_toml(root: &std::path::Path, id: u32) {
        let padded = format!("{id:03}");
        let dir = root.join(".doctrine").join("slice").join(&padded);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("slice-{padded}.toml")),
            format!(
                "id = {id}\nslug = \"s{padded}\"\ntitle = \"Test S{padded}\"\n\
                 status = \"proposed\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 [relationships]\nsupersedes = []\nsuperseded_by = []\nneeds = []\n",
            ),
        )
        .unwrap();
    }

    /// Helper: seed an ADR TOML (local copy — identical to relation.rs's helper).
    fn seed_adr_toml(root: &std::path::Path, id: u32) {
        let padded = format!("{id:03}");
        let dir = root.join(".doctrine").join("adr").join(&padded);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("adr-{padded}.toml")),
            format!(
                "id = {id}\nslug = \"a{padded}\"\ntitle = \"Test A{padded}\"\n\
                 status = \"accepted\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 [relationships]\nsupersedes = []\nsuperseded_by = []\n",
            ),
        )
        .unwrap();
    }

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
