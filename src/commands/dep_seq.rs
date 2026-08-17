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
    crate::kinds::WORK_LIKE.contains(&kind.prefix)
}

/// The record membership predicate (SL-158 D2) — the knowledge-record kinds that a
/// work item may now `needs`/`after`. Records are NOT work-like (cannot author
/// dep/seq), but are admissible as dep/seq TARGETS. Governance (SPEC/ADR/POL/STD)
/// stays excluded from BOTH gates.
pub(crate) fn is_record(kind: &'static crate::entity::Kind) -> bool {
    crate::kinds::is_record(kind.prefix)
}

/// The admissible target membership predicate (SL-158 D2) — widens the old single
/// work-like gate for targets to include records. A work item may now `needs`/`after`
/// a record (ASM/DEC/QUE/CON/EVD/HYP). Governance (SPEC/ADR/POL/STD) stays excluded because
/// depending on governance routes THROUGH a Revision, never the evergreen doc (the
/// SL-060 invariant).
pub(crate) fn is_admissible_dep_target(kind: &'static crate::entity::Kind) -> bool {
    is_work_like(kind) || is_record(kind)
}

/// SL-197 P4: the `RecordKind` vocabulary joined with "/" for the `dep_seq` error
/// message. When a 7th kind is added in PHASE-02, the message auto-includes it.
fn record_kind_list_slash() -> String {
    crate::knowledge::RecordKind::ALL
        .iter()
        .map(|k| k.as_str())
        .collect::<Vec<_>>()
        .join("/")
}

/// Resolve a dep/seq source to its TOML path. Validates: canonical-ref parse,
/// work-like kind (slice or backlog). Returns the resolved path plus the kind and
/// id it resolved to, so a caller that needs the canonical source id does not pay
/// a second [`crate::kinds::parse_resolvable_ref`] (SL-238 §6 — the remove paths
/// gate the source only, and still have to echo it).
fn resolve_dep_seq_src_path(
    root: &std::path::Path,
    source: &str,
) -> anyhow::Result<(PathBuf, &'static crate::kinds::KindRef, u32)> {
    let (skref, sid) = crate::kinds::parse_resolvable_ref(root, source)?;
    anyhow::ensure!(
        is_work_like(skref.kind),
        "`{source}` is a {} entity, which cannot author needs/after — only a slice or a backlog item (issue/improvement/chore/risk/idea) carries dep/seq",
        skref.kind.prefix
    );
    Ok((
        crate::entity::id_path(root, skref.kind, sid, crate::entity::Ext::Toml),
        skref,
        sid,
    ))
}

/// Canonicalise an authored dep/seq TARGET on the remove path, through §6's three
/// tiers in order (SL-238):
///
/// 1. [`crate::kinds::parse_resolvable_ref`] — so today's **bare-id** tolerance
///    survives (`after SL-100 154 --remove` resolves `154` to `SL-154`). Its disk
///    stat failing is NOT fatal here; falling through is the whole point.
/// 2. [`crate::kinds::parse_canonical_ref`] — pure and disk-free, so a well-formed
///    ref to a *deleted* target still canonicalises and can be cleared.
/// 3. verbatim — a ref that names nothing is still a string in an array that has
///    to come out.
///
/// **Known bound, deliberate (§6).** Both parse tiers hand off to `canonical_id`,
/// so the needle is always canonical: a hand-authored `needs = ["SL-1"]` is sought
/// as `SL-001` and never matches, and tier 3 does not rescue it because `SL-1`
/// *parses*. Narrow by construction — such a ref resolves, so the doctor check does
/// not report it either, and every CLI-authored ref is stored canonical. Closing it
/// means normalising on read in `kinds`, which has five other callers.
fn canonicalise_target(root: &std::path::Path, target: &str) -> String {
    // `map_or_else`, not `map(..).unwrap_or_else(..)` — clippy::pedantic denies the
    // latter, which is the shape §6's snippet uses. Same three tiers, same order.
    crate::kinds::parse_resolvable_ref(root, target)
        .or_else(|_| crate::kinds::parse_canonical_ref(target))
        .map_or_else(
            |_| target.to_string(),
            |(kref, id)| crate::listing::canonical_id(kref.kind.prefix, id),
        )
}

/// Resolve a generic dep/seq `(SRC, TGT)` pair against the author-time gate (§5.4),
/// returning SRC's `slice-NNN.toml`-shaped path ready for the leaf write. Rides the
/// SAME cross-kind canonical-ref seam as `link` (`crate::kinds::parse_canonical_ref` +
/// the `KindRef` `(dir, stem)` path map) — no new resolver. The three refusals, each
/// a clear, specific message:
///   1. SRC must resolve AND be a dep/seq-authoring (work-like) kind.
///   2. TGT must resolve on disk (free-text / dangling refused) AND be
///      admissible as a dep/seq target (work-like OR record). SL-158 D2 widened
///      the old work-like-only gate to admit knowledge records (ASM/DEC/QUE/CON/EVD/HYP).
///      Governance (SPEC/ADR/POL/STD) stays excluded from BOTH gates.
///   3. self-edge (SRC == TGT) refused.
///
/// Returns SRC's toml path plus the CANONICAL ids of both endpoints
/// ([`crate::listing::canonical_id`]) — the caller stores and echoes the canonical
/// form, so a non-canonical input (`SL-1`) normalizes at both the write and the
/// echo, matching the backlog path (IMP-140 F-13).
fn resolve_dep_seq_src(
    root: &std::path::Path,
    source: &str,
    target: &str,
) -> anyhow::Result<(PathBuf, String, String)> {
    let (toml_path, skref, sid) = resolve_dep_seq_src_path(root, source)?;
    // TGT must resolve on disk — a free-text or dangling target is refused here
    // (never write an edge to a non-entity). The resolver first so a
    // free-text target surfaces the ref-shape error, then a dir probe.
    let (tkref, tid) = crate::kinds::parse_resolvable_ref(root, target)?;
    anyhow::ensure!(
        is_admissible_dep_target(tkref.kind),
        "`{target}` is a {} entity — needs/after may only target work (a slice or a backlog item) or a knowledge record ({}); governance docs are excluded",
        tkref.kind.prefix,
        record_kind_list_slash(),
    );
    anyhow::ensure!(
        !(skref.kind.prefix == tkref.kind.prefix && sid == tid),
        "a {source} edge to itself is not a dependency — self-edges are refused"
    );
    let source_id = crate::listing::canonical_id(skref.kind.prefix, sid);
    let target_id = crate::listing::canonical_id(tkref.kind.prefix, tid);
    Ok((toml_path, source_id, target_id))
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
    let (toml_path, source_id, target_id) = resolve_dep_seq_src(&root, source, target)?;
    crate::dep_seq::append(
        &toml_path,
        &crate::dep_seq::RelEdit::Needs(std::slice::from_ref(&target_id)),
    )?;
    writeln!(std::io::stdout(), "{source_id} needs {target_id}")?;
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
    let (toml_path, source_id, target_id) = resolve_dep_seq_src(&root, source, target)?;
    crate::dep_seq::append(
        &toml_path,
        &crate::dep_seq::RelEdit::After {
            to: &target_id,
            rank,
        },
    )?;
    let suffix = if rank == 0 {
        String::new()
    } else {
        format!(" (rank {rank})")
    };
    writeln!(std::io::stdout(), "{source_id} after {target_id}{suffix}")?;
    Ok(())
}

/// `doctrine after <SRC> <TGT> --remove [--rank N]` — gate the SOURCE only, and
/// treat the target as an authored string canonicalised through
/// [`canonicalise_target`] (SL-238 §6).
///
/// **Deliberate behaviour change, superseding PHASE-02/VT-3.** This used to resolve
/// both endpoints through [`resolve_dep_seq_src`], which requires the target to
/// exist on disk and be an admissible kind. Right for authoring, wrong for repair:
/// it made the refs the doctor check reports at Error severity precisely the refs
/// `--remove` would not touch. Three author-time guarantees are given up ON THIS
/// PATH ONLY — the target's on-disk resolution, its kind gate, and the self-edge
/// refusal — because an edge that is already in the array has to be removable
/// whatever it says. [`run_after_edge`] and [`run_needs_edge`] keep the full gate.
pub(crate) fn run_after_remove(
    path: Option<PathBuf>,
    source: &str,
    target: &str,
    rank: i32,
) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (toml_path, skref, sid) = resolve_dep_seq_src_path(&root, source)?;
    let source_id = crate::listing::canonical_id(skref.kind.prefix, sid);
    let target_id = canonicalise_target(&root, target);
    let ceiling = if rank == 0 { None } else { Some(rank) };
    let removed = crate::dep_seq::remove(
        &toml_path,
        &crate::dep_seq::RelRemove::After {
            to: &target_id,
            rank_ceiling: ceiling,
        },
    )?;
    if removed == 0 {
        anyhow::bail!("{source_id} has no after edge to {target_id}");
    }
    writeln!(
        std::io::stdout(),
        "{source_id} after {target_id} removed ({} edge{})",
        removed,
        if removed == 1 { "" } else { "s" }
    )?;
    Ok(())
}

/// `doctrine needs <SRC> <TGT> --remove` (SL-238 §6) — the `needs` axis's first
/// removal verb, mirroring [`run_after_remove`]: gate the SOURCE only, canonicalise
/// the target through the three-tier needle, bail when nothing matched.
///
/// There is deliberately no `needs --prune`: a satisfied *hard* prerequisite is
/// meaningful history, and dropping it unasked is a judgement the tool should not
/// make. `--remove` is explicit and sufficient.
pub(crate) fn run_needs_remove(
    path: Option<PathBuf>,
    source: &str,
    target: &str,
) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (toml_path, skref, sid) = resolve_dep_seq_src_path(&root, source)?;
    let source_id = crate::listing::canonical_id(skref.kind.prefix, sid);
    let target_id = canonicalise_target(&root, target);
    let removed =
        crate::dep_seq::remove(&toml_path, &crate::dep_seq::RelRemove::Needs(&target_id))?;
    if removed == 0 {
        anyhow::bail!("{source_id} has no needs edge to {target_id}");
    }
    writeln!(
        std::io::stdout(),
        "{source_id} needs {target_id} removed ({} edge{})",
        removed,
        if removed == 1 { "" } else { "s" }
    )?;
    Ok(())
}

/// The reason word for a target the probe judged `Terminal` (SL-238 §6, `EX-4`).
///
/// `Known` renders the status verbatim. Today's copies append a `/resolution`
/// suffix by re-reading the raw toml; that is deliberately dropped — `Meta` carries
/// no `resolution` field, and adding one to a type this widely shared in order to
/// decorate a repair message is not the trade (§6, third consequence).
///
/// `Absent` is `Terminal` by the partition table — a status-less kind is
/// context-only and default-excluded — but has no word to render, so it names the
/// CLASS. This is §6's unnamed FIFTH consequence: an `after` edge onto a `REC`
/// becomes prunable where today it is kept, because today's `unwrap_or("")` matches
/// neither hardcoded literal (`EX-4`, owner accepted 2026-08-17).
///
/// `Unavailable` cannot reach here: [`crate::priority::partition::authored_class`]
/// maps it to `Unrecognised`, never `Terminal`, and that rule is precisely what
/// SL-238 exists to enforce. It is spelled out rather than `unreachable!()` so a
/// future change to that mapping degrades to an honest string instead of a panic.
fn terminal_reason(status: &crate::kinds::AuthoredStatus) -> String {
    match status {
        crate::kinds::AuthoredStatus::Known(s) => s.clone(),
        crate::kinds::AuthoredStatus::Absent => "status-less".to_string(),
        crate::kinds::AuthoredStatus::Unavailable => "status-unavailable".to_string(),
    }
}

/// `doctrine after <SRC> --prune` (SL-105 PHASE-03) — probe every `after` target
/// of SRC for dangling edges (absent or terminal target) and remove them. Reads
/// the `DepSeq` ONCE before any modifications (collecting dangling targets), then
/// removes in a second pass using the shared `dep_seq::remove` leaf.
pub(crate) fn run_after_prune(path: Option<PathBuf>, source: &str) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;
    // SL-238 PHASE-06: argument shape only. This copy echoes `{source}` AS TYPED,
    // which PHASE-02/VT-2 pinned as a deliberate divergence from the backlog copy's
    // canonical echo — PHASE-07 decides that, not this phase.
    let (toml_path, _, _) = resolve_dep_seq_src_path(&root, source)?;

    // 1. Read DepSeq
    let ds = crate::dep_seq::read(&toml_path)?;

    // 2. Probe each after-edge target — ONE resolver, ONE read per edge (SL-238 §6,
    //    EX-1). Terminality routes through `partition::authored_class` onto the
    //    single `status_class` table; no status literal is spelled here.
    let mut dropped: Vec<(String, i32, String)> = Vec::new();
    let mut to_drop: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for edge in &ds.after {
        let reason = match crate::kinds::parse_resolvable_ref(&root, &edge.to) {
            // The ref names nothing. Absorbs the missing-directory case with it, so
            // the three reason strings the old copies rendered (`absent`,
            // `(unparseable)`, `absent (unparseable ref)`) collapse to §4's token.
            Err(_) => Some("unresolved".to_string()),
            Ok((kref, tid)) => match crate::authored_status::read(&root, kref, tid) {
                // KEEP on an unreadable target — the conservative call — and say so
                // (STD-003). A repair verb that quietly declines to repair is the
                // dishonesty this slice is about. `{err:#}` renders the cause chain.
                Err(err) => {
                    writeln!(
                        std::io::stderr(),
                        "{source} after {} (rank {}) kept (unreadable: {err:#})",
                        edge.to,
                        edge.rank
                    )?;
                    None
                }
                Ok(authored) => {
                    match crate::priority::partition::authored_class(kref.kind, &authored.status) {
                        crate::priority::partition::StatusClass::Terminal => {
                            Some(terminal_reason(&authored.status))
                        }
                        // `Workable`, `Gating` and `Unrecognised` all keep the edge.
                        // `Unrecognised` is where `Unavailable` lands (§3, rule 3).
                        _ => None,
                    }
                }
            },
        };

        if let Some(reason) = reason {
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
        let _ = crate::dep_seq::remove(
            &toml_path,
            &crate::dep_seq::RelRemove::After {
                to: target,
                rank_ceiling: None,
            },
        )?;
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
    use crate::slice;

    /// P4 canary: the dep_seq error message lists record kinds from the RecordKind
    /// vocab (not a hand-spelled literal). When a 7th kind is added in PHASE-02, this
    /// test must be updated to match the new vocab.
    #[test]
    fn needs_after_message_lists_record_kinds_from_vocab() {
        let msg = record_kind_list_slash();
        assert_eq!(
            msg, "assumption/decision/question/constraint/evidence/hypothesis/concept",
            "P4: dep_seq record-kind list must be built from RecordKind vocab"
        );
    }

    /// SL-060 / SL-066 §PHASE-04: the work-like membership predicate is the ONE
    /// widen-later guard — exactly { slice } ∪ { the 5 backlog kinds } ∪ { revision },
    /// every other admitted kind refused. REV joins as both dep/seq source and target
    /// (the IDE-010 payoff); governance docs stay off the allowlist (SL-060 invariant).
    /// SL-158 D2 / SL-161 PHASE-01: the record membership predicate
    /// over KINDS equals [`crate::kinds::RECORD`] — set equality guards
    /// both false positives (a prefix in KINDS but not in RECORD) and
    /// false negatives (a prefix in RECORD but not captured by the predicate).
    #[test]
    fn is_record_predicate_matches_kinds_record() {
        let mut from_pred: Vec<&str> = crate::kinds::KINDS
            .iter()
            .filter(|k| is_record(k.kind))
            .map(|k| k.kind.prefix)
            .collect();
        from_pred.sort_unstable();
        let mut want: Vec<&str> = crate::kinds::RECORD.to_vec();
        want.sort_unstable();
        assert_eq!(from_pred, want);
    }

    /// SL-158 D2: the admissible-target predicate = work-like ∪ record.
    /// Governance (SPEC/ADR/POL/STD) and everything else stay excluded.
    #[test]
    fn is_admissible_dep_target_is_work_like_plus_records() {
        // work-like ∪ RECORD (kinds::ADMISSIBLE_DEP_TARGETS)
        let admissible: &[&str] = crate::kinds::ADMISSIBLE_DEP_TARGETS;
        for k in crate::kinds::KINDS
            .iter()
            .filter(|k| admissible.contains(&k.kind.prefix))
        {
            assert!(
                is_admissible_dep_target(k.kind),
                "{} is admissible as dep target",
                k.kind.prefix
            );
        }
        for k in crate::kinds::KINDS
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
        std::fs::write(root.join(crate::dtoml::DOCTRINE_TOML), "").unwrap();
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
        std::fs::write(root.join(crate::dtoml::DOCTRINE_TOML), "").unwrap();
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
        std::fs::write(root.join(crate::dtoml::DOCTRINE_TOML), "").unwrap();
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
        std::fs::write(root.join(crate::dtoml::DOCTRINE_TOML), "").unwrap();
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
        std::fs::write(root.join(crate::dtoml::DOCTRINE_TOML), "").unwrap();
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

    /// Helper: a project root with the doctrine marker, ready for `root::find`.
    fn seed_root(tmp: &tempfile::TempDir) -> &std::path::Path {
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".doctrine")).unwrap();
        std::fs::write(root.join(crate::dtoml::DOCTRINE_TOML), "").unwrap();
        root
    }

    /// Helper: seed a slice TOML carrying PRE-PLANTED dep/seq edges on either axis
    /// (SL-238 PHASE-06).
    ///
    /// The removal tests need edges the author-time gate REFUSES to write —
    /// `SL-9999` (parses, resolves to nothing), `not-a-ref` (does not parse),
    /// `SL-1` (well-formed but unpadded). There is no route to them through
    /// `run_*_edge`, so they are planted directly. One helper, both axes; do not
    /// hand-roll a second.
    fn seed_sl_with_edges(root: &std::path::Path, id: u32, needs: &[&str], after: &[(&str, i32)]) {
        let padded = format!("{id:03}");
        let dir = root.join(".doctrine").join("slice").join(&padded);
        std::fs::create_dir_all(&dir).unwrap();
        let needs_refs = needs
            .iter()
            .map(|r| format!("\"{r}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let after_edges = after
            .iter()
            .map(|(to, rank)| format!("{{ to = \"{to}\", rank = {rank} }}"))
            .collect::<Vec<_>>()
            .join(", ");
        std::fs::write(
            dir.join(format!("slice-{padded}.toml")),
            format!(
                "id = {id}\nslug = \"s{padded}\"\ntitle = \"Test S{padded}\"\n\
                 status = \"proposed\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 [relationships]\nsupersedes = []\nsuperseded_by = []\n\
                 needs = [{needs_refs}]\nafter = [{after_edges}]\n",
            ),
        )
        .unwrap();
    }

    /// Helper: the source slice's TOML text, for asserting what survived a removal.
    fn slice_toml(root: &std::path::Path, id: u32) -> String {
        let padded = format!("{id:03}");
        std::fs::read_to_string(
            root.join(".doctrine")
                .join("slice")
                .join(&padded)
                .join(format!("slice-{padded}.toml")),
        )
        .unwrap()
    }

    // --- `needs --remove` — SL-238 PHASE-06 EX-2 --------------------------------

    /// VT-1: the `needs` axis gains its first removal. One edge of two goes, the
    /// other stays, and the call succeeds.
    ///
    /// The count itself is echoed to `io::stdout()`, which these in-module tests do
    /// not capture — the rendered line is pinned black-box in
    /// `tests/e2e_dep_seq_verbs.rs` instead. Here the assertion is over the state.
    #[test]
    fn needs_remove_clears_one_edge_and_reports_the_count() {
        let tmp = tempfile::tempdir().unwrap();
        let root = seed_root(&tmp);
        seed_sl_with_edges(root, 1, &["SL-002", "SL-003"], &[]);
        seed_sl_toml(root, 2);
        seed_sl_toml(root, 3);

        run_needs_remove(Some(root.to_path_buf()), "SL-001", "SL-002").unwrap();

        let toml = slice_toml(root, 1);
        assert!(
            !toml.contains("SL-002"),
            "the named edge is cleared:\n{toml}"
        );
        assert!(toml.contains("SL-003"), "the other edge survives:\n{toml}");
    }

    /// VT-1: nothing matched → bail, and the file is untouched. Mirrors
    /// `run_after_remove`'s zero-count refusal.
    #[test]
    fn needs_remove_bails_when_no_edge_matches() {
        let tmp = tempfile::tempdir().unwrap();
        let root = seed_root(&tmp);
        seed_sl_with_edges(root, 1, &["SL-003"], &[]);
        seed_sl_toml(root, 2);
        seed_sl_toml(root, 3);

        let before = slice_toml(root, 1);
        let err = run_needs_remove(Some(root.to_path_buf()), "SL-001", "SL-002")
            .expect_err("no matching edge refuses");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("SL-001") && msg.contains("SL-002") && msg.contains("no needs edge"),
            "the refusal names both endpoints: {msg}"
        );
        assert_eq!(slice_toml(root, 1), before, "the file is untouched");
    }

    /// VT-2 (`needs` half): a ref that does not resolve is still clearable — the
    /// repair path PHASE-03's Error-severity check depends on. `needs` had no
    /// removal at all before this phase, so there is no before-state to supersede
    /// here; the `after` half of VT-2 is where the deliberate change lives.
    #[test]
    fn needs_remove_clears_a_ref_that_does_not_resolve() {
        let tmp = tempfile::tempdir().unwrap();
        let root = seed_root(&tmp);
        seed_sl_with_edges(root, 1, &["SL-9999", "not-a-ref"], &[]);

        run_needs_remove(Some(root.to_path_buf()), "SL-001", "SL-9999").unwrap();
        run_needs_remove(Some(root.to_path_buf()), "SL-001", "not-a-ref").unwrap();

        let toml = slice_toml(root, 1);
        assert!(
            !toml.contains("SL-9999") && !toml.contains("not-a-ref"),
            "both unresolvable refs are cleared:\n{toml}"
        );
    }

    // --- the remove path gates the SOURCE only — SL-238 PHASE-06 EX-3 ----------

    /// VT-2 (`after` half) — **the deliberate behaviour change of this phase.**
    ///
    /// `after --remove` used to resolve BOTH endpoints through
    /// `resolve_dep_seq_src`, so the very refs the doctor check reports at Error
    /// severity — `SL-9999`, `not-a-ref` — were exactly the refs `--remove` would
    /// not touch, leaving hand-editing the TOML as the only repair path.
    ///
    /// Supersedes **PHASE-02/VT-3**, which pinned that refusal precisely so this
    /// flip would read as intentional rather than as a regression. Authorised by
    /// design.md §6 `The remove path gates the source, not the target`, which calls
    /// it "a deliberate behaviour change ... and it is what makes §5's check
    /// repairable".
    #[test]
    fn after_remove_clears_a_ref_that_does_not_resolve() {
        let tmp = tempfile::tempdir().unwrap();
        let root = seed_root(&tmp);
        seed_sl_with_edges(root, 1, &[], &[("SL-9999", 0), ("not-a-ref", 0)]);

        run_after_remove(Some(root.to_path_buf()), "SL-001", "SL-9999", 0).unwrap();
        run_after_remove(Some(root.to_path_buf()), "SL-001", "not-a-ref", 0).unwrap();

        let toml = slice_toml(root, 1);
        assert!(
            !toml.contains("SL-9999") && !toml.contains("not-a-ref"),
            "both previously-unremovable edges are cleared:\n{toml}"
        );
    }

    /// VT-3 — the regression guard, and it is **green before and after** by design.
    ///
    /// Bare-id targets work today through `resolve_dep_seq_src`; resolving is what
    /// turns `154` into `SL-154`. The risk this phase introduces is losing that
    /// when the gate goes, so tier 1 of the needle is `parse_resolvable_ref`
    /// specifically to keep it. A test that never goes red looks weak unless its
    /// staying green IS the assertion — §7 calls this "the regression the
    /// three-tier needle exists to prevent".
    #[test]
    fn remove_accepts_a_bare_id_target() {
        let tmp = tempfile::tempdir().unwrap();
        let root = seed_root(&tmp);
        seed_sl_with_edges(root, 1, &[], &[("SL-154", 0)]);
        seed_sl_toml(root, 154);

        run_after_remove(Some(root.to_path_buf()), "SL-001", "154", 0)
            .expect("a bare id resolves to the canonical ref it names");

        assert!(
            !slice_toml(root, 1).contains("SL-154"),
            "the bare-id removal cleared the canonical edge"
        );
    }

    /// VT-4 — tier 2. `SL-9999` does not resolve on disk but still *parses*, so it
    /// canonicalises and a stale edge onto a deleted target stays clearable.
    /// Distinct from VT-2's case in what it exercises: here the stored ref is
    /// UNPADDED-safe canonical and the target is genuinely gone.
    #[test]
    fn remove_canonicalises_a_parseable_ref_to_a_deleted_target() {
        let tmp = tempfile::tempdir().unwrap();
        let root = seed_root(&tmp);
        // Stored canonical; removal requested in the equivalent bare-hyphen form
        // that `parse_canonical_ref` normalises to the same needle.
        seed_sl_with_edges(root, 1, &["SL-9999"], &[("SL-9999", 0)]);

        run_needs_remove(Some(root.to_path_buf()), "SL-001", "SL-9999").unwrap();
        run_after_remove(Some(root.to_path_buf()), "SL-001", "SL-9999", 0).unwrap();

        let toml = slice_toml(root, 1);
        assert!(
            !toml.contains("SL-9999"),
            "the stale edge is clearable on both axes:\n{toml}"
        );
    }

    /// VT-4 — tier 3. A ref that does not parse at all is still a string in an
    /// array that has to come out, so it is matched verbatim.
    #[test]
    fn remove_matches_an_unparseable_ref_verbatim() {
        let tmp = tempfile::tempdir().unwrap();
        let root = seed_root(&tmp);
        seed_sl_with_edges(root, 1, &["not-a-ref"], &[("not-a-ref", 0)]);

        run_needs_remove(Some(root.to_path_buf()), "SL-001", "not-a-ref").unwrap();
        run_after_remove(Some(root.to_path_buf()), "SL-001", "not-a-ref", 0).unwrap();

        let toml = slice_toml(root, 1);
        assert!(
            !toml.contains("not-a-ref"),
            "the free-text ref is matched verbatim and removed:\n{toml}"
        );
    }

    /// VT-5 / EX-4 — **the known bound, asserted rather than fixed.**
    ///
    /// A stored `needs = ["SL-1"]` is NOT cleared by `--remove SL-1`. Both parse
    /// tiers hand off to `canonical_id`, so the needle is `SL-001`; and tier 3 does
    /// not rescue it, because `SL-1` *parses*. The bare form is fine (`154` →
    /// `SL-154`); it is the short HYPHENATED form that is unreachable.
    ///
    /// Narrow by construction — such a ref resolves, so the doctor check does not
    /// report it either, and every CLI-authored ref is stored canonical. §6 names
    /// it rather than fixing it because normalising on read is a `kinds` change
    /// with five other callers, outside this slice. Pinned here so that closing it
    /// later is a deliberate change with a red test, not an accident.
    #[test]
    fn remove_does_not_match_a_well_formed_unpadded_ref() {
        let tmp = tempfile::tempdir().unwrap();
        let root = seed_root(&tmp);
        seed_sl_with_edges(root, 1, &["SL-1"], &[("SL-1", 0)]);

        let needs_err = run_needs_remove(Some(root.to_path_buf()), "SL-001", "SL-1")
            .expect_err("the unpadded ref is not matched");
        assert!(
            format!("{needs_err:#}").contains("SL-001"),
            "the needle canonicalised to SL-001, which is why it missed: {needs_err:#}"
        );
        run_after_remove(Some(root.to_path_buf()), "SL-001", "SL-1", 0)
            .expect_err("the same bound holds on the after axis");

        let toml = slice_toml(root, 1);
        assert!(
            toml.contains("\"SL-1\"") && toml.contains("to = \"SL-1\""),
            "both unpadded refs survive — the bound, not a capability:\n{toml}"
        );
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
        for k in crate::kinds::KINDS
            .iter()
            .filter(|k| matches!(k.kind.prefix, "ISS" | "IMP" | "CHR" | "RSK" | "IDE" | "REV"))
        {
            assert!(is_work_like(k.kind), "{} is work-like", k.kind.prefix);
        }
        // Every OTHER admitted kind in the corpus table is refused (gov / spec / req /
        // review / reconciliation / knowledge) — the closed allowlist.
        for k in crate::kinds::KINDS
            .iter()
            .filter(|k| !crate::kinds::WORK_LIKE.contains(&k.kind.prefix))
        {
            assert!(
                !is_work_like(k.kind),
                "{} must NOT be work-like (off the allowlist)",
                k.kind.prefix
            );
        }
    }
}
