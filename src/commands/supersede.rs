// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine supersede` — cross-kind supersession verb (SL-062 §5.4).
//! SL-129: uses `entity::id_path`, `dep_seq`, `relation`, `supersede`, `clock`

use std::path::PathBuf;

/// Resolve a supersession ref to its `<stem>-NNN.toml` path plus the canonical ref
/// string. Mirrors `resolve_link_path` (the same `KindRef` `(dir, stem)` path map),
/// but returns the normalised canonical id (`ADR-004`) — the exact string form the
/// `supersedes`/`superseded_by` arrays store (matching `validate`'s derived side,
/// which keys on `listing::canonical_id`).
fn resolve_supersede_path(
    root: &std::path::Path,
    kref: &crate::integrity::KindRef,
    id: u32,
) -> (PathBuf, String) {
    let toml_path = crate::entity::id_path(root, kref.kind, id, crate::entity::Ext::Toml);
    (
        toml_path,
        crate::listing::canonical_id(kref.kind.prefix, id),
    )
}

/// `doctrine supersede <NEW> <OLD>` (SL-062 §5.4) — the transactional, ADR-first
/// supersession verb. One parse-once / hold-both / write-once transaction composing
/// Cross-kind supersession for records + governance docs (`ADR` stays same-kind;
/// the four record kinds `ASM`/`DEC`/`QUE`/`CON` ride the §6 matrix). Composes
/// the PHASE-02 pure cores (`dep_seq::apply_string_append` + `dep_seq::apply_status`)
/// over docs parsed once and held in scope: `NEW.supersedes += OLD`,
/// `OLD.superseded_by += NEW` (the single sanctioned reverse carve-out, ADR-004 §5),
/// and flips `OLD.status → superseded`.
///
/// Pre-flight (NO write): refuse a self-edge, cross-kind refs, a non-ADR (no
/// `supersede_policy`) NEW; then parse BOTH docs and verify every touched key/array
/// is scaffold-present (F-1, non-destructive). The not-already-superseded guard (F-D)
/// allows ONLY the idempotent re-run (BOTH files already reciprocal); a different
/// supersessor or hand-drifted carve-out is refused. Writes NEW then OLD — the order
/// that makes a torn state (`NEW.supersedes∋OLD` without the reciprocal) detectable
/// by `doctrine validate`.
pub(crate) fn run_supersede(path: Option<PathBuf>, new: &str, old: &str) -> anyhow::Result<()> {
    use anyhow::Context;
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;

    // Pre-flight resolution + capability gate (NO write).
    let (new_kref, new_id) = crate::integrity::parse_canonical_ref(new)?;
    let (old_kref, old_id) = crate::integrity::parse_canonical_ref(old)?;
    anyhow::ensure!(
        !(new_kref.kind.prefix == old_kref.kind.prefix && new_id == old_id),
        "`{new}` cannot supersede itself — a self-supersession is not a decision change"
    );
    // Cross-kind gating: ADR → same-kind only; records → matrix; mixed → refuse.
    // The old same-kind guard is retained for non-ADR, non-record pairs (e.g. SL→SL).
    let new_is_adr = new_kref.kind.prefix == "ADR";
    let old_is_adr = old_kref.kind.prefix == "ADR";
    let new_is_record = crate::knowledge::RecordKind::from_prefix(new_kref.kind.prefix).is_some();
    let old_is_record = crate::knowledge::RecordKind::from_prefix(old_kref.kind.prefix).is_some();
    let same_family = if new_is_adr && old_is_adr {
        true // ADR family
    } else if new_is_record && old_is_record {
        // Both records: validate matrix. from_prefix already proved Some by the
        // is_some() gate, but each arm needs a non-panicking fallback for clippy.
        let Some(new_record_kind) = crate::knowledge::RecordKind::from_prefix(new_kref.kind.prefix)
        else {
            anyhow::bail!("NEW kind not a valid record kind")
        };
        let Some(old_record_kind) = crate::knowledge::RecordKind::from_prefix(old_kref.kind.prefix)
        else {
            anyhow::bail!("OLD kind not a valid record kind")
        };
        anyhow::ensure!(
            crate::supersede::validate_matrix(new_record_kind, old_record_kind),
            "cross-kind supersession refused: the §6 matrix disallows {} → {}",
            new_kref.kind.prefix,
            old_kref.kind.prefix
        );
        true // record family: matrix passed
    } else if new_kref.kind.prefix == old_kref.kind.prefix {
        true // same kind (e.g. SL→SL); fall through to supersede_policy "not yet supported"
    } else {
        false // cross-family or cross-kind
    };
    anyhow::ensure!(
        same_family,
        "cross-family supersession refused: `{new}` is a {} but `{old}` is a {}",
        new_kref.kind.prefix,
        old_kref.kind.prefix
    );
    let policy = crate::supersede::supersede_policy(new_kref.kind).with_context(|| {
        format!(
            "supersession not yet supported for {} (follow-up F2)",
            new_kref.kind.prefix
        )
    })?;

    // For cross-kind record supersession, OLD status should be based on OLD kind policy
    let old_policy = if !new_is_adr && !old_is_adr && new_kref.kind.prefix != old_kref.kind.prefix {
        crate::supersede::supersede_policy(old_kref.kind).with_context(|| {
            format!(
                "supersession not yet supported for OLD {} (follow-up F2)",
                old_kref.kind.prefix
            )
        })?
    } else {
        policy
    };

    let (new_path, new_ref) = resolve_supersede_path(&root, new_kref, new_id);
    let (old_path, old_ref) = resolve_supersede_path(&root, old_kref, old_id);

    // Parse BOTH docs ONCE and HOLD them in scope (parse-once / hold-both).
    let new_text = std::fs::read_to_string(&new_path)
        .with_context(|| format!("supersede: {new} not found at {}", new_path.display()))?;
    let mut new_doc = new_text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", new_path.display()))?;
    let old_text = std::fs::read_to_string(&old_path)
        .with_context(|| format!("supersede: {old} not found at {}", old_path.display()))?;
    let mut old_doc = old_text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", old_path.display()))?;

    // F-1 pre-flight on OLD's typed carve-out (always typed, both paths).
    let old_carveout = rel_array(&old_doc, policy.carveout_field);
    anyhow::ensure!(
        old_carveout.is_some(),
        "malformed `{old}` at {}: missing seeded `[relationships].{}` array — restore the seeded `[relationships]` arrays before superseding; the file is left untouched",
        old_path.display(),
        policy.carveout_field
    );
    anyhow::ensure!(
        old_doc
            .get("status")
            .and_then(toml_edit::Item::as_str)
            .is_some(),
        "malformed `{old}` at {}: missing seeded top-level `status` — restore the seeded keys before superseding; the file is left untouched",
        old_path.display()
    );
    anyhow::ensure!(
        old_doc
            .get("updated")
            .and_then(toml_edit::Item::as_str)
            .is_some(),
        "malformed `{old}` at {}: missing seeded top-level `updated` — restore the seeded keys before superseding; the file is left untouched",
        old_path.display()
    );

    let old_status = old_doc
        .get("status")
        .and_then(toml_edit::Item::as_str)
        .unwrap_or_default()
        .to_string();

    // Dispatch write path on storage discriminant (SL-095 D7).
    match policy.storage {
        crate::supersede::StorageTarget::RelationRow => {
            use crate::relation::{self, RelationLabel};

            // F-1 pre-flight: read [[relation]] rows for Supersedes on NEW.
            let relation_doc = crate::relation::RelationDoc::parse(&new_text)
                .with_context(|| {
                    format!(
                        "malformed `{new}` at {}: missing seeded `[[relation]]` table — restore the seeded template; the file is left untouched",
                        new_path.display()
                    )
                })?;
            let (edges, _illegal) = relation::read_block(new_kref.kind, &relation_doc);
            let existing_supersedes: Vec<_> = edges
                .iter()
                .filter(|e| e.label == RelationLabel::Supersedes)
                .collect();

            // F-D not-already-superseded guard ([[relation]] path).
            if old_status == policy.superseded_status {
                let carveout = old_carveout.unwrap_or_default();
                let new_lists_old = existing_supersedes.iter().any(|e| e.target == old_ref);
                let single_self = carveout.len() == 1 && carveout.first() == Some(&new_ref);
                if single_self && new_lists_old {
                    writeln!(
                        std::io::stdout(),
                        "already recorded: {new} supersedes {old}"
                    )?;
                    return Ok(());
                }
                if let Some(other) = carveout.iter().find(|x| **x != new_ref) {
                    anyhow::bail!("{old} already superseded by {other}; reopening is deferred");
                }
                anyhow::bail!(
                    "{old} status is superseded but its superseded_by carve-out is empty/inconsistent — run `doctrine validate`"
                );
            }
            // F-1: NEW must not already supersede a different entity.
            if let Some(edge) = existing_supersedes.first() {
                anyhow::bail!("{new} already supersedes {}", edge.target);
            }

            // Write NEW's outbound edge via [[relation]].
            let outcome = relation::append_edge(&new_path, RelationLabel::Supersedes, &old_ref)?;
            if matches!(outcome, relation::AppendOutcome::Noop) {
                writeln!(
                    std::io::stdout(),
                    "already recorded: {new} supersedes {old}"
                )?;
                // Still write OLD's carved-out + status (typed, below).
            } else {
                writeln!(std::io::stdout(), "{new} supersedes {old}")?;
            }

            // OLD: typed carved-out + status flip (unchanged).
            let today = crate::clock::today();
            let status_hint = format!(
                "malformed `{old}`: missing seeded top-level `status`/`updated` — restore the seeded keys; the file is left untouched"
            );
            crate::dep_seq::apply_string_append(&mut old_doc, policy.carveout_field, &new_ref)?;
            crate::dep_seq::apply_status(
                &mut old_doc,
                &[("status", policy.superseded_status), ("updated", &today)],
                &status_hint,
            )?;
            crate::fsutil::write_atomic(&old_path, old_doc.to_string().as_bytes())
                .with_context(|| format!("Failed to write {}", old_path.display()))?;
        }
        crate::supersede::StorageTarget::TypedArray { field } => {
            // F-1 pre-flight: typed outbound array must be scaffold-present.
            let new_sup = rel_array(&new_doc, field);
            anyhow::ensure!(
                new_sup.is_some(),
                "malformed `{new}` at {}: missing seeded `[relationships].{}` array — restore the seeded `[relationships]` arrays before superseding; the file is left untouched",
                new_path.display(),
                field
            );

            // F-D not-already-superseded guard (typed path, existing).
            if old_status == old_policy.superseded_status {
                let carveout = old_carveout.unwrap_or_default();
                let new_lists_old = new_sup.unwrap_or_default().contains(&old_ref);
                let single_self = carveout.len() == 1 && carveout.first() == Some(&new_ref);
                if single_self && new_lists_old {
                    writeln!(
                        std::io::stdout(),
                        "already recorded: {new} supersedes {old}"
                    )?;
                    return Ok(());
                }
                if let Some(other) = carveout.iter().find(|x| **x != new_ref) {
                    anyhow::bail!("{old} already superseded by {other}; reopening is deferred");
                }
                anyhow::bail!(
                    "{old} status is superseded but its superseded_by carve-out is empty/inconsistent — run `doctrine validate`"
                );
            }

            // Mutate the held docs (no IO) and write.
            let today = crate::clock::today();
            let status_hint = format!(
                "malformed `{old}`: missing seeded top-level `status`/`updated` — restore the seeded keys; the file is left untouched"
            );
            crate::dep_seq::apply_string_append(&mut new_doc, field, &old_ref)?;
            crate::dep_seq::apply_string_append(&mut old_doc, policy.carveout_field, &new_ref)?;

            // Conditional status flip: skip if OLD record is already terminal (SL-097 D2).
            let old_record_kind = crate::knowledge::RecordKind::from_prefix(old_kref.kind.prefix)
                .context("OLD kind not a valid record kind")?;
            if old_record_kind.is_terminal(&old_status) {
                // Already terminal: skip status flip, update timestamp only.
                old_doc
                    .as_table_mut()
                    .insert("updated", toml_edit::value(today.as_str()));
            } else {
                crate::dep_seq::apply_status(
                    &mut old_doc,
                    &[
                        ("status", old_policy.superseded_status),
                        ("updated", &today),
                    ],
                    &status_hint,
                )?;
            }

            // Write each file ONCE, NEW then OLD.
            crate::fsutil::write_atomic(&new_path, new_doc.to_string().as_bytes())
                .with_context(|| format!("Failed to write {}", new_path.display()))?;
            crate::fsutil::write_atomic(&old_path, old_doc.to_string().as_bytes())
                .with_context(|| format!("Failed to write {}", old_path.display()))?;

            writeln!(std::io::stdout(), "{new} supersedes {old}")?;
        }
    }
    Ok(())
}

/// Read a `[relationships].<field>` array's string elements off a held doc (pre-flight
/// presence probe + membership reads). `None` iff the seeded array is absent (F-1).
fn rel_array(doc: &toml_edit::DocumentMut, field: &str) -> Option<Vec<String>> {
    doc.get("relationships")
        .and_then(toml_edit::Item::as_table)
        .and_then(|t| t.get(field))
        .and_then(toml_edit::Item::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;
    use crate::catalog;

    /// CHR-008 (SL-078): `run_supersede` writes NEW then OLD — a crash between
    /// writes leaves a torn state (NEW.supersedes ∋ OLD without the reciprocal).
    /// Re-running the same command naturally completes recovery through the
    /// existing flow: F-1 passes, F-D skips (OLD.status ≠ superseded),
    /// push_str_if_absent on NEW is a no-op, push_str_if_absent on OLD writes
    /// the missing entry, and the status flip completes the transaction.
    #[test]
    fn supersede_recovery_from_torn_new_only_state() {
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        // Seed ADR-001 (NEW): supersedes = ["ADR-002"], superseded_by = [].
        catalog::test_helpers::write(
            root,
            ".doctrine/adr/001/adr-001.toml",
            "id = 1\nslug = \"a1\"\ntitle = \"A1\"\nstatus = \"accepted\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = [\"ADR-002\"]\nsuperseded_by = []\n",
        );
        catalog::test_helpers::write(root, ".doctrine/adr/001/adr-001.md", "body\n");

        // Seed ADR-002 (OLD) in the torn state: superseded_by = [], status = accepted.
        catalog::test_helpers::write(
            root,
            ".doctrine/adr/002/adr-002.toml",
            "id = 2\nslug = \"a2\"\ntitle = \"A2\"\nstatus = \"accepted\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n",
        );
        catalog::test_helpers::write(root, ".doctrine/adr/002/adr-002.md", "body\n");

        // Act: re-run supersede from the torn state.
        run_supersede(Some(root.to_path_buf()), "ADR-001", "ADR-002")
            .expect("recovery supersede should succeed");

        // Assert: OLD status flipped to superseded.
        let old_toml =
            std::fs::read_to_string(root.join(".doctrine/adr/002/adr-002.toml")).unwrap();
        assert!(
            old_toml.contains("status = \"superseded\""),
            "OLD.status should be superseded, got: {old_toml}"
        );

        // Assert: OLD.superseded_by contains ADR-001.
        assert!(
            old_toml.contains("superseded_by = [\"ADR-001\"]"),
            "OLD.superseded_by should contain ADR-001, got: {old_toml}"
        );

        // Assert: NEW has a [[relation]] label="supersedes" row targeting ADR-002
        // (SL-095 PHASE-03: RelationRow path writes [[relation]] rows, not typed arrays).
        let new_toml =
            std::fs::read_to_string(root.join(".doctrine/adr/001/adr-001.toml")).unwrap();
        assert!(
            new_toml.contains("[[relation]]")
                && new_toml.contains("label = \"supersedes\"")
                && new_toml.contains("target = \"ADR-002\""),
            "NEW should have [[relation]] supersedes → ADR-002: {new_toml}"
        );
    }

    // --- SL-097 PHASE-03: record cross-kind supersession tests ----------------

    #[test]
    fn supersede_same_kind_record_allowed() {
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        // Seed ASM-001 (NEW): open status
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/001/record-001.toml",
            "id = 1\nslug = \"a1\"\ntitle = \"A1\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"yes_no\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/001/record-001.md",
            "body\n",
        );

        // Seed ASM-002 (OLD): open
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/002/record-002.toml",
            "id = 2\nslug = \"a2\"\ntitle = \"A2\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"yes_no\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/002/record-002.md",
            "body\n",
        );

        // Act
        run_supersede(Some(root.to_path_buf()), "ASM-001", "ASM-002")
            .expect("same-kind record supersession should succeed");

        // Assert: OLD status flipped to obsolete
        let old_toml = std::fs::read_to_string(
            root.join(".doctrine/knowledge/assumption/002/record-002.toml"),
        )
        .unwrap();
        assert!(
            old_toml.contains("status = \"obsolete\""),
            "OLD.status should be obsolete, got: {old_toml}"
        );
        assert!(
            old_toml.contains("superseded_by = [\"ASM-001\"]"),
            "OLD.superseded_by should contain ASM-001, got: {old_toml}"
        );
        let new_toml = std::fs::read_to_string(
            root.join(".doctrine/knowledge/assumption/001/record-001.toml"),
        )
        .unwrap();
        assert!(
            new_toml.contains("supersedes = [\"ASM-002\"]"),
            "NEW.supersedes should contain ASM-002, got: {new_toml}"
        );
    }

    #[test]
    fn supersede_cross_kind_allowed_matrix() {
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        // Seed DEC-001 (NEW): decision
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/001/record-001.toml",
            "id = 1\nslug = \"d1\"\ntitle = \"D1\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"action_item\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/001/record-001.md",
            "body\n",
        );

        // Seed ASM-002 (OLD): assumption
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/002/record-002.toml",
            "id = 2\nslug = \"a2\"\ntitle = \"A2\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"yes_no\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/002/record-002.md",
            "body\n",
        );

        // Act: supersede cross-kind (DEC → ASM is allowed per §6 matrix)
        run_supersede(Some(root.to_path_buf()), "DEC-001", "ASM-002")
            .expect("cross-kind supersession DEC → ASM should succeed");

        // Assert: OLD status flipped to obsolete
        let old_toml = std::fs::read_to_string(
            root.join(".doctrine/knowledge/assumption/002/record-002.toml"),
        )
        .unwrap();
        assert!(
            old_toml.contains("status = \"obsolete\""),
            "OLD.status should be obsolete, got: {old_toml}"
        );
    }

    #[test]
    fn supersede_cross_kind_refused_matrix() {
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        // Seed ASM-001 (NEW): assumption
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/001/record-001.toml",
            "id = 1\nslug = \"a1\"\ntitle = \"A1\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"yes_no\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/001/record-001.md",
            "body\n",
        );

        // Seed DEC-002 (OLD): decision
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/002/record-002.toml",
            "id = 2\nslug = \"d2\"\ntitle = \"D2\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"action_item\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/002/record-002.md",
            "body\n",
        );

        // Act: supersede cross-kind (ASM → DEC is disallowed per §6 matrix)
        let result = run_supersede(Some(root.to_path_buf()), "ASM-001", "DEC-002");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("§6 matrix disallows ASM → DEC")
        );
    }

    #[test]
    fn supersede_question_reopening_refused() {
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        // Seed QUE-001 (NEW): question
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/question/001/record-001.toml",
            "id = 1\nslug = \"q1\"\ntitle = \"Q1\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"yes_no\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/question/001/record-001.md",
            "body\n",
        );

        // Seed QUE-002 (OLD): question with terminal status
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/question/002/record-002.toml",
            "id = 2\nslug = \"q2\"\ntitle = \"Q2\"\nstatus = \"answered\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"yes_no\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/question/002/record-002.md",
            "body\n",
        );

        // Act: supersede terminal record - status should NOT flip
        run_supersede(Some(root.to_path_buf()), "QUE-001", "QUE-002")
            .expect("supersession should proceed but not flip terminal status");

        // Assert: OLD status stays answered (terminal status not flipped)
        let old_toml =
            std::fs::read_to_string(root.join(".doctrine/knowledge/question/002/record-002.toml"))
                .unwrap();
        assert!(
            old_toml.contains("status = \"answered\""),
            "OLD.status should remain answered (terminal), got: {old_toml}"
        );
        // But timestamp should be updated
        assert!(
            !old_toml.contains("updated = \"2026-01-01\""),
            "OLD.updated should be refreshed, got: {old_toml}"
        );
    }

    #[test]
    fn supersede_cross_family_refused() {
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        // Seed ADR-001
        catalog::test_helpers::write(
            root,
            ".doctrine/adr/001/adr-001.toml",
            "id = 1\nslug = \"a1\"\ntitle = \"A1\"\nstatus = \"accepted\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n",
        );
        catalog::test_helpers::write(root, ".doctrine/adr/001/adr-001.md", "body\n");

        // Seed ASM-002
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/002/record-002.toml",
            "id = 2\nslug = \"a2\"\ntitle = \"A2\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"yes_no\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/002/record-002.md",
            "body\n",
        );

        // Cross-family: ADR (gov) vs assumption (record)
        let result = run_supersede(Some(root.to_path_buf()), "ADR-001", "ASM-002");
        assert!(
            result.is_err(),
            "cross-family supersession should be refused"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("cross-family"),
            "error should mention cross-family, got: {err}"
        );
    }

    #[test]
    fn supersede_self_supersession_refused() {
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        // Seed ASM-001
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/001/record-001.toml",
            "id = 1\nslug = \"a1\"\ntitle = \"A1\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"yes_no\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/assumption/001/record-001.md",
            "body\n",
        );

        // Act: self-supersession should fail
        let result = run_supersede(Some(root.to_path_buf()), "ASM-001", "ASM-001");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cannot supersede itself")
        );
    }

    #[test]
    fn supersede_already_terminal_no_flip() {
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        // Seed DEC-001 (NEW)
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/001/record-001.toml",
            "id = 1\nslug = \"d1\"\ntitle = \"D1\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/001/record-001.md",
            "body\n",
        );

        // Seed DEC-002 (OLD): decision with terminal status
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/002/record-002.toml",
            "id = 2\nslug = \"d2\"\ntitle = \"D2\"\nstatus = \"accepted\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"action_item\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/002/record-002.md",
            "body\n",
        );

        // Act: supersede terminal record
        run_supersede(Some(root.to_path_buf()), "DEC-001", "DEC-002")
            .expect("supersession should succeed but not flip terminal status");

        // Assert: OLD status stays accepted (terminal status preserved)
        let old_toml =
            std::fs::read_to_string(root.join(".doctrine/knowledge/decision/002/record-002.toml"))
                .unwrap();
        assert!(
            old_toml.contains("status = \"accepted\""),
            "OLD.status should remain accepted (terminal), got: {old_toml}"
        );
        // Timestamp should still be updated
        assert!(
            !old_toml.contains("updated = \"2026-01-01\""),
            "OLD.updated should be refreshed even for terminal, got: {old_toml}"
        );
    }

    #[test]
    fn supersede_idempotent_cross_kind() {
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        // Seed CON-001 (NEW): already linked to QUE-002
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/constraint/001/record-001.toml",
            "id = 1\nslug = \"c1\"\ntitle = \"C1\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = [\"QUE-002\"]\nsuperseded_by = []\n[facet]\nkind = \"implementation\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/constraint/001/record-001.md",
            "body\n",
        );

        // Seed QUE-002 (OLD): already superseded by CON-001
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/question/002/record-002.toml",
            "id = 2\nslug = \"q2\"\ntitle = \"Q2\"\nstatus = \"obsolete\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = [\"CON-001\"]\n[facet]\nkind = \"yes_no\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/question/002/record-002.md",
            "body\n",
        );

        // Act: re-run the same supersession (idempotent)
        run_supersede(Some(root.to_path_buf()), "CON-001", "QUE-002")
            .expect("idempotent cross-kind supersession should succeed");

        // Assert: no changes, relationships preserved
        let new_toml = std::fs::read_to_string(
            root.join(".doctrine/knowledge/constraint/001/record-001.toml"),
        )
        .unwrap();
        let old_toml =
            std::fs::read_to_string(root.join(".doctrine/knowledge/question/002/record-002.toml"))
                .unwrap();
        assert!(new_toml.contains("supersedes = [\"QUE-002\"]"));
        assert!(old_toml.contains("superseded_by = [\"CON-001\"]"));
        assert!(old_toml.contains("status = \"obsolete\""));
    }

    #[test]
    fn supersede_decision_to_question_reopening_refused() {
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        // Seed QUE-001 (NEW, question)
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/question/001/record-001.toml",
            "id = 1\nslug = \"q1\"\ntitle = \"Q1\"\nstatus = \"open\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/question/001/record-001.md",
            "body\n",
        );

        // Seed DEC-002 (OLD, decision, already superseded by DEC-003)
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/002/record-002.toml",
            "id = 2\nslug = \"d2\"\ntitle = \"D2\"\nstatus = \"superseded\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = [\"DEC-003\"]\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/002/record-002.md",
            "body\n",
        );

        // Act: QUE → DEC (should be refused — DEC already superseded by DEC-003)
        let result = run_supersede(Some(root.to_path_buf()), "QUE-001", "DEC-002");
        assert!(
            result.is_err(),
            "reopening a superseded decision with a question should be refused"
        );
    }

    #[test]
    fn supersede_torn_recovery() {
        // VT-10: NEW has the supersedes edge but OLD's superseded_by is missing
        // — re-running the supersede verb should recover (detected as drift into
        // empty/inconsistent carve-out, or recover on re-run if status is not yet
        // superseded).
        let tmp = catalog::test_helpers::tmp();
        let root = tmp.path();

        // First: create a valid supersession DEC-001 → DEC-002
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/001/record-001.toml",
            "id = 1\nslug = \"d1\"\ntitle = \"D1\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"action_item\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/001/record-001.md",
            "body\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/002/record-002.toml",
            "id = 2\nslug = \"d2\"\ntitle = \"D2\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"action_item\"\n",
        );
        catalog::test_helpers::write(
            root,
            ".doctrine/knowledge/decision/002/record-002.md",
            "body\n",
        );

        run_supersede(Some(root.to_path_buf()), "DEC-001", "DEC-002")
            .expect("initial supersession should succeed");

        // Now simulate torn state: remove superseded_by from OLD
        std::fs::write(
            root.join(".doctrine/knowledge/decision/002/record-002.toml"),
            "id = 2\nslug = \"d2\"\ntitle = \"D2\"\nstatus = \"superseded\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             [relationships]\nsupersedes = []\nsuperseded_by = []\n[facet]\nkind = \"action_item\"\n",
        )
        .unwrap();

        // Re-run: should detect drift (status=superseded but carve-out empty)
        let result = run_supersede(Some(root.to_path_buf()), "DEC-001", "DEC-002");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("superseded_by carve-out is empty")
        );
    }
}
