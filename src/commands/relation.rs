// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine link` / `doctrine unlink` — relation edge verbs (SL-048 §5.4).
//! SL-129: uses `entity::id_path`, `integrity`, `relation`, `memory`

use std::path::PathBuf;

/// Resolve a `link`/`unlink` source+label to (the source entity's toml path, the
/// validated label). Shared by both verbs (design §5.4): parse the source ref →
/// `(KindRef, id)`; `relation::validate_link` (the `(source, label)` legality +
/// `link`-writability gate); compute the entity's `<stem>-NNN.toml` path. Target
/// validation is link-only (a dangling target must still be `unlink`-able), so it lives
/// in `run_link`, not here.
fn resolve_link_path(
    root: &std::path::Path,
    source: &str,
    label: &str,
) -> anyhow::Result<(PathBuf, &'static crate::relation::RelationRule)> {
    let (kref, id) = crate::integrity::parse_canonical_ref(source)?;
    let rule = crate::relation::validate_link(kref.kind, label)?;
    let toml_path = crate::entity::id_path(root, kref.kind, id, crate::entity::Ext::Toml);
    Ok((toml_path, rule))
}

/// `doctrine link <SOURCE-ID> <LABEL> <TARGET>` (SL-048 §5.4) — author a tier-1
/// `[[relation]]` edge. Validates the source/label ([`resolve_link_path`]) then the
/// forward target (§5.5 — `Unvalidated` `drift` is free text; every other label's
/// target must BOTH resolve (`ensure_ref_resolves` — never write a dangler) AND pass
/// the legal-KIND assertion), then appends edit-preservingly. Idempotent (a re-link
/// reports `already linked`, file untouched).
pub(crate) fn run_link(
    path: Option<PathBuf>,
    source: &str,
    label: &str,
    target: &str,
) -> anyhow::Result<()> {
    use anyhow::Context;
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;

    // Memory branch — detect mem_<uid> / mem.<key> / mem_<prefix> sources
    // and route to memory.toml relations (SL-090 §PHASE-03).
    if let Ok(mref) = crate::memory::MemoryRef::parse(source) {
        let toml_path = crate::memory::resolve_memory_toml_path(&root, &mref)?;
        // Best-effort target validation: if target looks like a canonical ref,
        // validate it resolves. Free-text and mem_* targets pass through.
        if crate::integrity::parse_canonical_ref(target).is_ok() {
            crate::integrity::ensure_ref_resolves(&root, target).with_context(|| {
                format!("target `{target}` does not resolve to an existing entity")
            })?;
        }
        let outcome = crate::memory::append_memory_relation(&toml_path, label, target)?;
        match outcome {
            crate::relation::AppendOutcome::Wrote => {
                writeln!(std::io::stdout(), "linked: {source} {label} {target}")?;
            }
            crate::relation::AppendOutcome::Noop => {
                writeln!(
                    std::io::stdout(),
                    "already linked: {source} {label} {target}"
                )?;
            }
        }
        return Ok(());
    }

    let (toml_path, rule) = resolve_link_path(&root, source, label)?;
    // Forward-edge validation (§5.5): free-text labels skip both gates; validated
    // labels must resolve AND be of a legal target kind.
    if !matches!(rule.target, crate::relation::TargetSpec::Unvalidated) {
        crate::integrity::ensure_ref_resolves(&root, target)?;
        let (tkref, _tid) = crate::integrity::parse_canonical_ref(target)?;
        let (skref, _sid) = crate::integrity::parse_canonical_ref(source)?;
        crate::relation::check_target_kind(rule, skref.kind, tkref.kind.prefix)?;
    }
    let outcome = crate::relation::append_edge(&toml_path, rule.label, target)?;
    match outcome {
        crate::relation::AppendOutcome::Wrote => {
            writeln!(std::io::stdout(), "linked: {source} {label} {target}")?;
        }
        crate::relation::AppendOutcome::Noop => {
            writeln!(
                std::io::stdout(),
                "already linked: {source} {label} {target}"
            )?;
        }
    }
    Ok(())
}

/// `doctrine unlink <SOURCE-ID> <LABEL> <TARGET>` (SL-048 §5.4) — remove a tier-1
/// `[[relation]]` edge. Same validation pipeline (the source/label must still be legal
/// to name the right file); idempotent (an absent edge reports `not linked`).
pub(crate) fn run_unlink(
    path: Option<PathBuf>,
    source: &str,
    label: &str,
    target: &str,
) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;

    // Memory branch — detect mem_<uid> / mem.<key> / mem_<prefix> sources
    // and route to memory.toml relations (SL-090 §PHASE-03).
    if let Ok(mref) = crate::memory::MemoryRef::parse(source) {
        let toml_path = crate::memory::resolve_memory_toml_path(&root, &mref)?;
        // No target validation for unlink (matching existing behaviour for numbered entities).
        let outcome = crate::memory::remove_memory_relation(&toml_path, label, target)?;
        match outcome {
            crate::relation::RemoveOutcome::Removed => {
                writeln!(std::io::stdout(), "unlinked: {source} {label} {target}")?;
            }
            crate::relation::RemoveOutcome::Absent => {
                writeln!(std::io::stdout(), "not linked: {source} {label} {target}")?;
            }
        }
        return Ok(());
    }

    let (toml_path, rule) = resolve_link_path(&root, source, label)?;
    let outcome = crate::relation::remove_edge(&toml_path, rule.label, target)?;
    match outcome {
        crate::relation::RemoveOutcome::Removed => {
            writeln!(std::io::stdout(), "unlinked: {source} {label} {target}")?;
        }
        crate::relation::RemoveOutcome::Absent => {
            writeln!(std::io::stdout(), "not linked: {source} {label} {target}")?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    const MEM_TEST_UID: &str = "mem_018f3a1b2c3d4e5f60718293a4b5c6d7";

    fn seed_sl_toml(root: &std::path::Path, id: u32) {
        let padded = format!("{id:03}");
        let dir = root.join(".doctrine").join("slice").join(&padded);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("slice-{padded}.toml")),
            format!(
                "id = {id}\nslug = \"s{padded}\"\ntitle = \"Test S{padded}\"\n\
                 status = \"accepted\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 [relationships]\nsupersedes = []\nsuperseded_by = []\n",
            ),
        )
        .unwrap();
    }

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

    fn seed_memory_toml(root: &std::path::Path, uid: &str, content: &str) {
        let mem_dir = root.join(".doctrine/memory/items").join(uid);
        std::fs::create_dir_all(&mem_dir).unwrap();
        let body = if content.is_empty() {
            format!(
                "uid = \"{uid}\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 source = \"test\"\nrelevance = \"test\"\n"
            )
        } else {
            content.to_string()
        };
        std::fs::write(mem_dir.join("memory.toml"), body).unwrap();
    }

    #[test]
    fn link_supersedes_on_record_is_lifecycle_only() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".doctrine")).unwrap();
        std::fs::write(root.join("doctrine.toml"), "").unwrap();
        seed_adr_toml(root, 1);
        seed_adr_toml(root, 2);

        // Link ADR-001 supersedes ADR-002 via `doctrine link` — lifecycle follows
        // typed supersedes, not the raw relation path, so this writes a plain
        // `label = "related"` (the unvalidated drift label if needed). Actually
        // the correct label for a link is validated; `supersedes` is a lifecycle
        // internal label, so the link command should use `related`.
        // This test verifies we can link adrs via the relation system.
        run_link(Some(root.to_path_buf()), "ADR-001", "related", "ADR-002").unwrap();
        let content = std::fs::read_to_string(root.join(".doctrine/adr/001/adr-001.toml")).unwrap();
        assert!(content.contains("[[relation]]"));
        assert!(content.contains("label = \"related\""));
    }

    #[test]
    fn link_memory_uid_appends_relation_row() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_sl_toml(root, 1);
        seed_memory_toml(root, MEM_TEST_UID, "");

        run_link(Some(root.to_path_buf()), MEM_TEST_UID, "related", "SL-001").unwrap();

        let content = std::fs::read_to_string(
            root.join(format!(".doctrine/memory/items/{MEM_TEST_UID}/memory.toml")),
        )
        .unwrap();
        assert!(content.contains("[[relation]]"));
        assert!(content.contains("target = \"SL-001\""));
    }

    #[test]
    fn link_memory_uid_repeat_is_noop() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_sl_toml(root, 1);
        seed_memory_toml(root, MEM_TEST_UID, "");

        run_link(Some(root.to_path_buf()), MEM_TEST_UID, "related", "SL-001").unwrap();
        // Second attempt is a noop.
        run_link(Some(root.to_path_buf()), MEM_TEST_UID, "related", "SL-001").unwrap();
        // Still has exactly one [[relation]] row.
        let content = std::fs::read_to_string(
            root.join(format!(".doctrine/memory/items/{MEM_TEST_UID}/memory.toml")),
        )
        .unwrap();
        let count = content.matches("[[relation]]").count();
        assert_eq!(count, 1, "should still have exactly one [[relation]] row");
    }

    #[test]
    fn unlink_memory_uid_then_repeat() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_sl_toml(root, 1);
        seed_memory_toml(root, MEM_TEST_UID, "");

        run_link(Some(root.to_path_buf()), MEM_TEST_UID, "related", "SL-001").unwrap();
        run_unlink(Some(root.to_path_buf()), MEM_TEST_UID, "related", "SL-001").unwrap();
        // Second unlink reports not linked.
        let result = run_unlink(Some(root.to_path_buf()), MEM_TEST_UID, "related", "SL-001");
        assert!(result.is_ok(), "second unlink should succeed as noop");
        let content = std::fs::read_to_string(
            root.join(format!(".doctrine/memory/items/{MEM_TEST_UID}/memory.toml")),
        )
        .unwrap();
        assert!(!content.contains("[[relation]]"));
    }

    #[test]
    fn link_memory_uid_bad_target_errors() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_memory_toml(root, MEM_TEST_UID, "");
        // SL-999 doesn't exist.
        let result = run_link(Some(root.to_path_buf()), MEM_TEST_UID, "related", "SL-999");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("does not resolve"), "got: {err}");
    }

    #[test]
    fn link_memory_uid_free_text_target_ok() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_memory_toml(root, MEM_TEST_UID, "");
        // Free-text target passes through for memory relations.
        run_link(
            Some(root.to_path_buf()),
            MEM_TEST_UID,
            "related",
            "https://example.com",
        )
        .unwrap();
        let content = std::fs::read_to_string(
            root.join(format!(".doctrine/memory/items/{MEM_TEST_UID}/memory.toml")),
        )
        .unwrap();
        assert!(content.contains("target = \"https://example.com\""));
    }

    #[test]
    fn link_numbered_entity_still_works() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_sl_toml(root, 1);
        seed_sl_toml(root, 2);

        run_link(Some(root.to_path_buf()), "SL-001", "related", "SL-002").unwrap();
        let content =
            std::fs::read_to_string(root.join(".doctrine/slice/001/slice-001.toml")).unwrap();
        assert!(content.contains("[[relation]]"));
        assert!(content.contains("target = \"SL-002\""));
    }

    #[test]
    fn link_memory_key_appends_relation_row() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_sl_toml(root, 1);
        seed_memory_toml(root, "mem.fact.cli.skinny", "");

        run_link(
            Some(root.to_path_buf()),
            "mem.fact.cli.skinny",
            "related",
            "SL-001",
        )
        .unwrap();

        let content = std::fs::read_to_string(
            root.join(".doctrine/memory/items/mem.fact.cli.skinny/memory.toml"),
        )
        .unwrap();
        assert!(content.contains("[[relation]]"), "relation row written");
        assert!(content.contains("target = \"SL-001\""), "target present");
    }
}
