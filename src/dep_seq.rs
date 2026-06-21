// SPDX-License-Identifier: GPL-3.0-only
//! Authored-TOML edit-preserving mutation leaf (SL-060 PHASE-02 dep/seq origin;
//! SL-062 PHASE-02 the unified status seam). Two families of strict, in-place
//! `toml_edit` mutations over an authored entity's `<stem>-NNN.toml`:
//!
//! - **dep/sequence** ([`read`]/[`append`]): the `[relationships].{needs,after}`
//!   axes — the `needs` hard-prerequisite list and the `after` soft-sequence edges
//!   (each with a per-edge `rank`). Lifted verbatim from `backlog.rs` (the SL-047
//!   origin) so slices reuse the SAME typed schema + the SAME strict append (D1/§5.2).
//! - **top-level status** ([`apply_status`]/[`set_authored_status`]): the ONE
//!   edit-preserving write-core for the four status setters (governance, slice,
//!   backlog, requirement). Each kind keeps its own gate/coupling in its shell and
//!   delegates only the WRITE here, eliminating the byte-duplicated write body.
//!
//! Both families share the same invariants — the no-op guard (an unchanged value
//! writes nothing, mtime holds), the F-1 strict refuse (a missing scaffold-seeded
//! key is malformed; a tail `insert` would land it inside a trailing subtable =
//! silent corruption, so refuse non-destructively, NEVER recreate), and write-once.
//!
//! Pure cores ([`apply_status`]/[`apply_string_append`]) take a held
//! `&mut DocumentMut` and do NO disk/clock (the date is injected by the shell). The
//! IO wrappers ([`set_authored_status`]/[`append_string_array`]/[`append`]) do
//! read→parse→core→write-once.
//!
//! Leaf layering (ADR-001): imports only `toml_edit`/`anyhow`/`std`. It does NOT
//! depend on `backlog`, the priority engine, or any command module — the engine
//! and command tiers depend on it, never the reverse. No cycle.
//!
//! `promoted` does NOT live here: it is a backlog projection detail (`resolution
//! == Promoted`), read backlog-side in backlog's own one-parse `dep_seq_for`. The
//! leaf `DepSeq` carries only the two relation axes, kind-neutral.

use std::path::Path;

use anyhow::Context;

/// A soft-sequence edge: this entity runs `after` the predecessor `to`, with an
/// optional per-edge `rank` (default `0` — a plain soft edge; a non-zero rank is a
/// manual tie-break hint). A bare `{ to = "X" }` is rank 0. Serde round-trips the
/// `[[relationships.after]]` inline-table shape (and feeds the `show --json`
/// projection verbatim, so the derives must stay).
#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct AfterEdge {
    pub(crate) to: String,
    #[serde(default)]
    pub(crate) rank: i32,
}

/// The typed dep/sequence block read off an entity's `[relationships]` table:
/// `needs` (hard prerequisite refs, payload-free) and `after` (soft manual
/// sequence, per-edge `rank`). Kind-neutral — a kind that does not author dep/seq
/// reads to an empty `DepSeq`. (NB: no `promoted` field — that is a backlog-only
/// projection, read separately backlog-side.)
//
// PHASE-03 wired the slice read path onto `read`/`DepSeq` (`slice show` surfaces the
// dep/seq axes), so the schema is now live in the lib build — the PHASE-02 dead-code
// expectations are retired.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct DepSeq {
    pub(crate) needs: Vec<String>,
    pub(crate) after: Vec<AfterEdge>,
}

/// The tolerant read layer for [`read`] — `[relationships]` with the two dep/seq
/// arrays, each `#[serde(default)]` so a table omitting one (or an entirely absent
/// table) still parses to the empty axis.
#[derive(Debug, Default, serde::Deserialize)]
struct RawRelationships {
    #[serde(default)]
    needs: Vec<String>,
    #[serde(default)]
    after: Vec<AfterEdge>,
}

#[derive(Debug, Default, serde::Deserialize)]
struct RawDepSeqToml {
    #[serde(default)]
    relationships: RawRelationships,
}

/// A single edit to one `[relationships]` array — append `needs` refs, or append
/// one `{ to, rank }` `after` edge (the array-of-inline-tables axis, one `to` per
/// invocation). The refs are pre-validated by the caller's shell before this runs.
pub(crate) enum RelEdit<'a> {
    /// Append these prereq refs to `[relationships].needs`.
    Needs(&'a [String]),
    /// Append one `{ to, rank }` edge to `[relationships].after`.
    After { to: &'a str, rank: i32 },
}

/// Read the typed `[relationships]` dep/seq block from an entity's TOML. An absent
/// `[relationships]` table (or absent axis) reads to an empty `DepSeq` — a kind
/// that does not author dep/seq. Pure over the file's own text.
pub(crate) fn read(toml_path: &Path) -> anyhow::Result<DepSeq> {
    let text = std::fs::read_to_string(toml_path)
        .with_context(|| format!("dep/seq entity not found at {}", toml_path.display()))?;
    let raw: RawDepSeqToml = toml::from_str(&text)
        .with_context(|| format!("Failed to parse {}", toml_path.display()))?;
    Ok(DepSeq {
        needs: raw.relationships.needs,
        after: raw.relationships.after,
    })
}

/// Edit-preserving append into one `[relationships]` array — the `toml_edit`
/// in-place mutate so comments, inert tables, and unknown keys survive verbatim
/// (the file is never reserialised). Navigates `[relationships]` → the target
/// array, pushes each new entry, and writes once.
///
/// **STRICT refuse (F-1)**: if `[relationships]` or the seeded target array is
/// absent, this is a malformed (hand-edited) entity — a tail `insert` would land
/// the array inside a trailing subtable. Refuse instead, touching nothing. The
/// refuse is NON-DESTRUCTIVE: it points at restoring the seeded arrays / running
/// the backfill, NEVER at regenerating or recreating the file (which would nuke an
/// authored entity).
///
/// **Idempotent**: an entry already present (a `needs` ref already listed, or an
/// identical `{ to, rank }` edge) is not duplicated; if every entry is already
/// present the file is left byte-identical (no write, mtime holds).
pub(crate) fn append(toml_path: &Path, edit: &RelEdit<'_>) -> anyhow::Result<()> {
    let text = std::fs::read_to_string(toml_path)
        .with_context(|| format!("dep/seq entity not found at {}", toml_path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", toml_path.display()))?;

    // F-1: `[relationships]` and the target axis array are scaffold-seeded; their
    // absence means a malformed entity (a tail insert would corrupt a trailing
    // subtable). The refuse stays non-destructive — restore the seeded arrays, do
    // NOT recreate the file.
    let axis = match edit {
        RelEdit::Needs(_) => "needs",
        RelEdit::After { .. } => "after",
    };
    let array = doc
        .get_mut("relationships")
        .and_then(toml_edit::Item::as_table_mut)
        .and_then(|t| t.get_mut(axis))
        .and_then(toml_edit::Item::as_array_mut)
        .with_context(|| {
            format!(
                "malformed entity at {}: missing seeded `[relationships].{axis}` array — restore the seeded `[relationships]` arrays (e.g. via the backfill) before adding edges; the file is left untouched",
                toml_path.display()
            )
        })?;

    let mut changed = false;
    match edit {
        RelEdit::Needs(refs) => {
            for r in *refs {
                // idempotent string-membership push — the SAME body as
                // `apply_string_append`'s inner helper (no parallel impl).
                changed |= push_str_if_absent(array, r.as_str());
            }
        }
        RelEdit::After { to, rank } => {
            // idempotent: skip an identical `{ to, rank }` edge.
            let present = array.iter().any(|v| {
                v.as_inline_table().is_some_and(|t| {
                    t.get("to").and_then(toml_edit::Value::as_str) == Some(to)
                        && t.get("rank").and_then(toml_edit::Value::as_integer)
                            == Some(i64::from(*rank))
                })
            });
            if !present {
                let mut edge = toml_edit::InlineTable::new();
                edge.insert("to", (*to).into());
                edge.insert("rank", i64::from(*rank).into());
                array.push(edge);
                changed = true;
            }
        }
    }

    if !changed {
        return Ok(()); // every entry already present — write nothing (mtime holds).
    }
    crate::fsutil::write_atomic(toml_path, doc.to_string().as_bytes())
        .with_context(|| format!("Failed to write {}", toml_path.display()))?;
    Ok(())
}

/// Remove `after` edges from `[relationships].after` matching `to`.
/// `rank_ceiling`: `None` → all ranks; `Some(n)` → only edges where rank ≤ n.
///
/// Returns the number of edges removed (0 if none matched).
///
/// **F-1**: `[relationships].after` array absent → bail with a non-destructive
/// message (malformed entity, never create).
pub(crate) fn remove_after(
    doc: &mut toml_edit::DocumentMut,
    to: &str,
    rank_ceiling: Option<i32>,
) -> anyhow::Result<usize> {
    let array = doc
        .get_mut("relationships")
        .and_then(toml_edit::Item::as_table_mut)
        .and_then(|t| t.get_mut("after"))
        .and_then(toml_edit::Item::as_array_mut)
        .with_context(|| {
            "malformed entity: missing seeded `[relationships].after` array — restore the seeded arrays before removing edges; the file is left untouched"
        })?;

    // Collect matching indices in forward order (remove in reverse to avoid shift).
    let indices: Vec<usize> = array
        .iter()
        .enumerate()
        .filter_map(|(idx, v)| {
            let t = v.as_inline_table()?;
            let to_matches = t.get("to").and_then(toml_edit::Value::as_str) == Some(to);
            if !to_matches {
                return None;
            }
            if let Some(ceiling) = rank_ceiling {
                let rank = t.get("rank").and_then(toml_edit::Value::as_integer)?;
                if rank > i64::from(ceiling) {
                    return None;
                }
            }
            Some(idx)
        })
        .collect();

    let count = indices.len();
    // Remove in reverse index order to avoid index shift.
    for idx in indices.into_iter().rev() {
        array.remove(idx);
    }
    Ok(count)
}

/// IO wrapper for [`remove_after`]: read→parse→core→write-once.
/// If count == 0, returns `Ok(0)` without writing (mtime holds).
pub(crate) fn remove(
    toml_path: &Path,
    to: &str,
    rank_ceiling: Option<i32>,
) -> anyhow::Result<usize> {
    let text = std::fs::read_to_string(toml_path)
        .with_context(|| format!("dep/seq entity not found at {}", toml_path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", toml_path.display()))?;
    let count = remove_after(&mut doc, to, rank_ceiling)?;
    if count > 0 {
        crate::fsutil::write_atomic(toml_path, doc.to_string().as_bytes())
            .with_context(|| format!("Failed to write {}", toml_path.display()))?;
    }
    Ok(count)
}

/// Idempotent string-membership push into a `toml_edit` array — the shared inner
/// core for BOTH `append`'s `Needs` arm and [`apply_string_append`]. Pushes
/// `value` only if no existing string element already equals it; returns whether
/// the array changed. No parallel implementation: one body, two callers.
fn push_str_if_absent(array: &mut toml_edit::Array, value: &str) -> bool {
    if array.iter().any(|v| v.as_str() == Some(value)) {
        return false;
    }
    array.push(value);
    true
}

// ---------------------------------------------------------------------------
// top-level status — the unified edit-preserving write seam (SL-062 PHASE-02)
// ---------------------------------------------------------------------------

/// Pure core for the four status setters: set each top-level `(key, value)` pair in
/// `managed` on a held `&mut DocumentMut`, edit-preserving. No disk, no clock — the
/// shell injects `today` into `managed`.
///
/// The variable-length `managed` slice is what lets ONE core serve every shape:
/// `[status, updated]` (slice/gov), `[status, resolution, updated]` (backlog), and
/// the lone `[status]` (requirement — no `updated`).
///
/// - **No-op** (I5): every managed *identity* pair already equals the current
///   top-level value → `Ok(false)`, no mutation (mtime/content hold). The `updated`
///   stamp is EXCLUDED from this comparison — it is a derived stamp (today), never an
///   identity, so it must not by itself force a write nor block the no-op. This
///   matches every donor: gov/slice keyed the no-op on `status` alone, backlog on
///   `status`+`resolution`; none compared `updated`.
/// - **F-1 strict refuse**: ANY managed key absent from the top-level table → the
///   entity is malformed (hand-edited); a tail `insert` would land the key inside a
///   trailing `[relationships]`/`[facet]` subtable (silent corruption). `bail!(hint)`,
///   NEVER insert a missing key. CHR-019 proved root `insert` is safe in
///   `toml_edit` 0.22, but the status-path bail is kept as
///   over-conservative-but-harmless per SL-136 D4 scoping.
/// - Else: insert each pair → `Ok(true)`.
pub(crate) fn apply_status(
    doc: &mut toml_edit::DocumentMut,
    managed: &[(&str, &str)],
    hint: &str,
) -> anyhow::Result<bool> {
    // No-op guard (before the malformed check): every managed *identity* pair already
    // current. The `updated` stamp is excluded — it is derived, not an identity.
    let unchanged = managed
        .iter()
        .filter(|(k, _)| *k != "updated")
        .all(|(k, v)| doc.get(k).and_then(toml_edit::Item::as_str) == Some(*v));
    if unchanged {
        return Ok(false);
    }

    let table = doc.as_table_mut();
    // F-1: the managed keys are scaffold-seeded — edit in place, never create. A
    // missing key means a malformed entity; refuse rather than tail-insert.
    if managed.iter().any(|(k, _)| !table.contains_key(k)) {
        anyhow::bail!("{hint}");
    }
    for (k, v) in managed {
        table.insert(k, toml_edit::value(*v));
    }
    Ok(true)
}

/// Pure core: idempotent string-append into `[relationships].<field>` on a held
/// `&mut DocumentMut`. REUSES [`push_str_if_absent`] — the SAME membership logic
/// `append`'s `Needs` arm uses. `Ok(false)` if `value` is already present (no
/// mutation); F-1 refuse (bail) if `[relationships].<field>` array is absent (never
/// create — a tail insert would corrupt a trailing subtable).
//
// Consumed in non-test builds by the `supersede` verb (SL-062 PHASE-03), which
// composes this core over both held docs — so the prior `dead_code` expectation is
// retired. `append_string_array` (the IO wrapper) stays staged/unused for now.
pub(crate) fn apply_string_append(
    doc: &mut toml_edit::DocumentMut,
    field: &str,
    value: &str,
) -> anyhow::Result<bool> {
    let array = doc
        .get_mut("relationships")
        .and_then(toml_edit::Item::as_table_mut)
        .and_then(|t| t.get_mut(field))
        .and_then(toml_edit::Item::as_array_mut)
        .with_context(|| {
            format!(
                "malformed entity: missing seeded `[relationships].{field}` array — restore the seeded `[relationships]` arrays (e.g. via the backfill) before adding edges; the file is left untouched"
            )
        })?;
    Ok(push_str_if_absent(array, value))
}

/// IO wrapper for [`apply_status`]: read→parse→core→write-once. Plain
/// `read_to_string`, parse a `DocumentMut`, call the core; if it returned `true`,
/// `fs::write` ONCE. Returns the core's changed-bool (so callers can branch).
pub(crate) fn set_authored_status(
    path: &Path,
    managed: &[(&str, &str)],
    hint: &str,
) -> anyhow::Result<bool> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("entity not found at {}", path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    let changed = apply_status(&mut doc, managed, hint)?;
    if changed {
        crate::fsutil::write_atomic(path, doc.to_string().as_bytes())
            .with_context(|| format!("Failed to write {}", path.display()))?;
    }
    Ok(changed)
}

/// IO wrapper for [`apply_string_append`]: read→parse→core→write-once.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "string-append IO wrapper staged for the slice relate consumer"
    )
)]
pub(crate) fn append_string_array(path: &Path, field: &str, value: &str) -> anyhow::Result<bool> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("entity not found at {}", path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    let changed = apply_string_append(&mut doc, field, value)?;
    if changed {
        crate::fsutil::write_atomic(path, doc.to_string().as_bytes())
            .with_context(|| format!("Failed to write {}", path.display()))?;
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal entity TOML with the seeded `[relationships]` arrays — the
    /// scaffold shape the strict append navigates into.
    fn seeded() -> String {
        "id = 1\nslug = \"a\"\ntitle = \"A\"\n\n[relationships]\nneeds = []\nafter = []\n"
            .to_string()
    }

    fn write_tmp(body: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("entity.toml");
        std::fs::write(&path, body).unwrap();
        (dir, path)
    }

    #[test]
    fn read_on_no_relationships_table_is_empty() {
        let (_dir, path) = write_tmp("id = 1\nslug = \"a\"\ntitle = \"A\"\n");
        let ds = read(&path).unwrap();
        assert_eq!(ds, DepSeq::default(), "absent table → empty DepSeq");
    }

    #[test]
    fn read_round_trips_needs_and_after() {
        let body = "[relationships]\nneeds = [\"ISS-002\"]\n\
             after = [{ to = \"ISS-003\", rank = 2 }, { to = \"ISS-004\" }]\n";
        let (_dir, path) = write_tmp(body);
        let ds = read(&path).unwrap();
        assert_eq!(ds.needs, vec!["ISS-002"]);
        assert_eq!(
            ds.after,
            vec![
                AfterEdge {
                    to: "ISS-003".to_string(),
                    rank: 2,
                },
                AfterEdge {
                    to: "ISS-004".to_string(),
                    rank: 0,
                },
            ]
        );
    }

    #[test]
    fn append_refuses_when_relationships_table_absent() {
        let (_dir, path) = write_tmp("id = 1\nslug = \"a\"\ntitle = \"A\"\n");
        let before = std::fs::read_to_string(&path).unwrap();
        let err = append(&path, &RelEdit::Needs(&["ISS-002".to_string()]));
        assert!(err.is_err(), "absent [relationships] table is refused");
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            before,
            "untouched on refuse"
        );
    }

    #[test]
    fn append_refuses_when_needs_array_absent() {
        // a bare `[relationships]` header (no seeded `needs`) still refuses — the
        // strict navigates to the array, not just the table.
        let (_dir, path) = write_tmp("id = 1\n[relationships]\nafter = []\n");
        let before = std::fs::read_to_string(&path).unwrap();
        let err = append(&path, &RelEdit::Needs(&["ISS-002".to_string()]));
        assert!(err.is_err(), "absent `needs` array is refused");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
    }

    #[test]
    fn append_refuses_when_after_array_absent() {
        let (_dir, path) = write_tmp("id = 1\n[relationships]\nneeds = []\n");
        let before = std::fs::read_to_string(&path).unwrap();
        let err = append(
            &path,
            &RelEdit::After {
                to: "ISS-002",
                rank: 0,
            },
        );
        assert!(err.is_err(), "absent `after` array is refused");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
    }

    #[test]
    fn append_refuse_message_is_non_destructive() {
        // E6/§5.2: the refuse must NOT instruct regeneration/recreation of the file.
        let (_dir, path) = write_tmp("id = 1\nslug = \"a\"\ntitle = \"A\"\n");
        let err = append(&path, &RelEdit::Needs(&["ISS-002".to_string()]))
            .expect_err("absent table refuses");
        let msg = format!("{err:#}").to_lowercase();
        assert!(
            !msg.contains("regenerate") && !msg.contains("recreate") && !msg.contains(" new`"),
            "refuse must never instruct regeneration/recreation: {msg}"
        );
    }

    #[test]
    fn append_is_idempotent() {
        let (_dir, path) = write_tmp(&seeded());
        append(&path, &RelEdit::Needs(&["ISS-002".to_string()])).unwrap();
        let once = std::fs::read_to_string(&path).unwrap();
        // a second identical append is a no-op — byte-identical, never duplicated.
        append(&path, &RelEdit::Needs(&["ISS-002".to_string()])).unwrap();
        assert_eq!(once, std::fs::read_to_string(&path).unwrap(), "idempotent");
        assert_eq!(
            read(&path).unwrap().needs,
            vec!["ISS-002"],
            "not duplicated"
        );
    }

    #[test]
    fn append_round_trip_golden_keeps_relationships_first() {
        // append needs + after, read back, assert structure AND that the written
        // TOML keeps `[relationships]` before any later content (the F-1 invariant).
        let mut body = seeded();
        body.push_str("\n# hand note — keep me\n[custom]\nkeep = \"yes\"\n");
        let (_dir, path) = write_tmp(&body);

        append(
            &path,
            &RelEdit::Needs(&["ISS-002".to_string(), "RSK-001".to_string()]),
        )
        .unwrap();
        append(
            &path,
            &RelEdit::After {
                to: "ISS-003",
                rank: 5,
            },
        )
        .unwrap();

        let ds = read(&path).unwrap();
        assert_eq!(ds.needs, vec!["ISS-002", "RSK-001"]);
        assert_eq!(
            ds.after,
            vec![AfterEdge {
                to: "ISS-003".to_string(),
                rank: 5,
            }]
        );

        let written = std::fs::read_to_string(&path).unwrap();
        assert!(
            written.contains("# hand note — keep me"),
            "comment survives"
        );
        assert!(written.contains("[custom]"), "inert table survives");
        let rel = written
            .find("[relationships]")
            .expect("relationships present");
        let custom = written.find("[custom]").expect("custom present");
        assert!(
            rel < custom,
            "[relationships] stays before later content:\n{written}"
        );
    }

    // --- VT-3: the unified status seam, driven through the IO wrapper -------------

    /// A seeded authored entity carrying the full managed-key set (status,
    /// resolution, updated) plus a comment, an inline `[relationships]` and an
    /// array-of-tables `[[relation]]` — the edit-preservation witnesses.
    fn seeded_status_entity() -> String {
        "id = 1\nslug = \"a\"\ntitle = \"A\"\n\
         status = \"pending\"\nresolution = \"\"\nupdated = \"2020-01-01\"\n\
         # hand note — keep me\n\
         [relationships]\nneeds = [\"ISS-002\"]\nafter = []\n\
         [[relation]]\nkind = \"refines\"\nto = \"SL-001\"\n"
            .to_string()
    }

    const HINT: &str = "malformed: missing seeded `status` — restore the seeded keys; untouched";

    #[test]
    fn set_authored_status_no_op_holds_content_and_mtime() {
        let (_dir, path) = write_tmp(&seeded_status_entity());
        let before = std::fs::read_to_string(&path).unwrap();
        let before_mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
        // every managed value already current → no write.
        let changed = set_authored_status(
            &path,
            &[
                ("status", "pending"),
                ("resolution", ""),
                ("updated", "2020-01-01"),
            ],
            HINT,
        )
        .unwrap();
        assert!(!changed, "no-op returns false");
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            before,
            "content held"
        );
        assert_eq!(
            std::fs::metadata(&path).unwrap().modified().unwrap(),
            before_mtime,
            "mtime held"
        );
    }

    #[test]
    fn set_authored_status_f1_refuses_non_destructively_and_touches_nothing() {
        // a file missing the `updated` managed key → bail; SL-060 lesson: the message
        // must NOT instruct regeneration ("regenerate"/"new"/"scaffold").
        let (_dir, path) =
            write_tmp("id = 1\nstatus = \"pending\"\n\n[relationships]\nneeds = []\nafter = []\n");
        let before = std::fs::read_to_string(&path).unwrap();
        let err = set_authored_status(
            &path,
            &[("status", "active"), ("updated", "2099-01-01")],
            "malformed: restore the seeded `status`/`updated` keys; the file is left untouched",
        )
        .expect_err("missing managed key refuses");
        let msg = format!("{err:#}").to_lowercase();
        assert!(
            !msg.contains("regenerate") && !msg.contains(" new`") && !msg.contains("scaffold"),
            "F-1 refuse must be non-destructive: {msg}"
        );
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            before,
            "untouched on refuse"
        );
    }

    #[test]
    fn set_authored_status_multi_key_round_trips_preserving_structure() {
        // backlog shape: status + resolution + updated; [relationships], [[relation]],
        // and the comment all survive verbatim.
        let (_dir, path) = write_tmp(&seeded_status_entity());
        let changed = set_authored_status(
            &path,
            &[
                ("status", "done"),
                ("resolution", "completed"),
                ("updated", "2099-12-31"),
            ],
            HINT,
        )
        .unwrap();
        assert!(changed, "a real move returns true");
        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains("status = \"done\""));
        assert!(written.contains("resolution = \"completed\""));
        assert!(written.contains("updated = \"2099-12-31\""));
        assert!(
            written.contains("# hand note — keep me"),
            "comment survives"
        );
        assert!(written.contains("[relationships]"), "inline table survives");
        assert!(
            written.contains("needs = [\"ISS-002\"]"),
            "relationships content survives"
        );
        assert!(written.contains("[[relation]]"), "array-of-tables survives");
    }

    #[test]
    fn set_authored_status_single_key_round_trips_no_updated() {
        // requirement shape: a lone managed key (no `updated`) — proves the
        // variable-length `managed` slice serves both shapes. No `updated` appears.
        let (_dir, path) = write_tmp(&seeded_status_entity());
        let changed = set_authored_status(&path, &[("status", "active")], HINT).unwrap();
        assert!(changed);
        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains("status = \"active\""));
        // the lone-key path touches only `status`: the prior `updated` stamp is
        // unchanged (no new stamp introduced by this shape).
        assert!(
            written.contains("updated = \"2020-01-01\""),
            "updated untouched"
        );
        assert!(written.contains("[relationships]"), "structure preserved");
    }

    #[test]
    fn append_string_array_idempotent_and_f1_refuse() {
        // the string-append IO wrapper shares push_str_if_absent with append's Needs.
        let (_dir, path) = write_tmp(&seeded_status_entity());
        // ISS-002 already present → no change.
        assert!(
            !append_string_array(&path, "needs", "ISS-002").unwrap(),
            "idempotent"
        );
        // a fresh ref appends.
        assert!(
            append_string_array(&path, "needs", "RSK-001").unwrap(),
            "appends new"
        );
        assert_eq!(read(&path).unwrap().needs, vec!["ISS-002", "RSK-001"]);
        // F-1: an absent array refuses, non-destructively.
        let (_dir2, bare) = write_tmp("id = 1\nstatus = \"a\"\n");
        let before = std::fs::read_to_string(&bare).unwrap();
        let err = append_string_array(&bare, "needs", "X").expect_err("absent array refuses");
        let msg = format!("{err:#}").to_lowercase();
        assert!(
            !msg.contains("regenerate") && !msg.contains("recreate"),
            "non-destructive: {msg}"
        );
        assert_eq!(std::fs::read_to_string(&bare).unwrap(), before, "untouched");
    }

    // --- remove_after & remove — SL-105 PHASE-01 --------------------------------

    /// Seeded entity with 3 `after` edges to X at ranks 0, 2, 5.
    fn seeded_with_after() -> String {
        "id = 1\nslug = \"a\"\ntitle = \"A\"\n\n[relationships]\nneeds = []\n\
         after = [\n  { to = \"X\", rank = 0 },\n  { to = \"X\", rank = 2 },\n  { to = \"X\", rank = 5 },\n]\n"
            .to_string()
    }

    fn seeded_mixed_after() -> String {
        "id = 1\nslug = \"a\"\ntitle = \"A\"\n\n[relationships]\nneeds = []\n\
         after = [\n  { to = \"X\", rank = 0 },\n  { to = \"Y\", rank = 1 },\n  { to = \"Z\", rank = 2 },\n]\n"
            .to_string()
    }

    #[test]
    fn remove_after_all_matching() {
        // VT-1: 3 after edges to X, remove all → count=3, none to X remain.
        let (_dir, path) = write_tmp(&seeded_with_after());
        let text = std::fs::read_to_string(&path).unwrap();
        let mut doc = text.parse::<toml_edit::DocumentMut>().unwrap();
        let count = remove_after(&mut doc, "X", None).unwrap();
        assert_eq!(count, 3, "all 3 edges to X removed");
        // Check the in-memory doc (we didn't write back, mutated doc directly).
        let array = doc
            .get("relationships")
            .and_then(|v| v.as_table())
            .and_then(|t| t.get("after"))
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(array.len(), 0, "no edges remain");
    }

    #[test]
    fn remove_after_rank_ceiling() {
        // VT-2: edges to X at ranks 0, 2, 5. rank_ceiling=2 → remove rank 0 and 2,
        // keep rank 5.
        let (_dir, path) = write_tmp(&seeded_with_after());
        let text = std::fs::read_to_string(&path).unwrap();
        let mut doc = text.parse::<toml_edit::DocumentMut>().unwrap();
        let count = remove_after(&mut doc, "X", Some(2)).unwrap();
        assert_eq!(count, 2, "only rank 0 and 2 removed");
        let array = doc
            .get("relationships")
            .and_then(|v| v.as_table())
            .and_then(|t| t.get("after"))
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(array.len(), 1, "one edge remains");
        let remaining = array.get(0).and_then(|v| v.as_inline_table()).unwrap();
        assert_eq!(
            remaining.get("rank").and_then(|v| v.as_integer()),
            Some(5),
            "rank 5 edge kept"
        );
    }

    #[test]
    fn remove_after_no_match() {
        // VT-3: edge to Y only. remove_after X → count=0.
        let body = "id = 1\nslug = \"a\"\ntitle = \"A\"\n\n[relationships]\nneeds = []\n\
                     after = [{ to = \"Y\", rank = 0 }]\n";
        let (_dir, path) = write_tmp(body);
        let text = std::fs::read_to_string(&path).unwrap();
        let mut doc = text.parse::<toml_edit::DocumentMut>().unwrap();
        let count = remove_after(&mut doc, "X", None).unwrap();
        assert_eq!(count, 0, "no match → zero removed");
    }

    #[test]
    fn remove_after_mixed_targets() {
        // VT-3 extension: edges to X, Y, Z. Remove X. Assert only X gone, Y and Z
        // untouched, count=1.
        let (_dir, path) = write_tmp(&seeded_mixed_after());
        let text = std::fs::read_to_string(&path).unwrap();
        let mut doc = text.parse::<toml_edit::DocumentMut>().unwrap();
        let count = remove_after(&mut doc, "X", None).unwrap();
        assert_eq!(count, 1, "only X removed");
        let array = doc
            .get("relationships")
            .and_then(|v| v.as_table())
            .and_then(|t| t.get("after"))
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(array.len(), 2, "Y and Z remain");
        let tos: Vec<&str> = array
            .iter()
            .filter_map(|v| {
                v.as_inline_table()
                    .and_then(|t| t.get("to"))
                    .and_then(|v| v.as_str())
            })
            .collect();
        assert_eq!(tos, vec!["Y", "Z"], "only Y and Z remain");
    }

    #[test]
    fn remove_after_f1_refuse() {
        // VT-5/F-1: entity with no `after` array at all — no `[relationships]` or
        // `[relationships]` without `after`. remove_after bails. Assert error contains
        // "malformed" or "missing seeded".
        let (_dir, path) = write_tmp("id = 1\nslug = \"a\"\ntitle = \"A\"\n");
        let text = std::fs::read_to_string(&path).unwrap();
        let mut doc = text.parse::<toml_edit::DocumentMut>().unwrap();
        let err = remove_after(&mut doc, "X", None).expect_err("absent after array refuses");
        let msg = format!("{err:#}").to_lowercase();
        assert!(
            msg.contains("malformed") || msg.contains("missing seeded"),
            "F-1 refuse message: {msg}"
        );
        assert!(
            !msg.contains("regenerate") && !msg.contains("recreate"),
            "non-destructive: {msg}"
        );
    }

    #[test]
    fn remove_io_noop_holds_mtime() {
        // VT-4: no matching edges. Call remove. Assert mtime unchanged.
        let body = "id = 1\nslug = \"a\"\ntitle = \"A\"\n\n[relationships]\nneeds = []\n\
                     after = [{ to = \"Y\", rank = 0 }]\n";
        let (_dir, path) = write_tmp(body);
        let before_mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
        let count = remove(&path, "X", None).unwrap();
        assert_eq!(count, 0, "no match");
        assert_eq!(
            std::fs::metadata(&path).unwrap().modified().unwrap(),
            before_mtime,
            "mtime held on noop"
        );
    }

    #[test]
    fn remove_io_roundtrip() {
        // VT-5: entity with comment and inert [notes] table plus after edges.
        // Remove one edge. Assert comment, [notes] table, other edges survive.
        let body = r#"id = 1
slug = "a"
title = "A"
# keep this comment
[notes]
info = "survive"

[relationships]
needs = []
after = [
  { to = "X", rank = 0 },
  { to = "Y", rank = 1 },
]
"#;
        let (_dir, path) = write_tmp(body);
        let count = remove(&path, "X", None).unwrap();
        assert_eq!(count, 1);
        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains("# keep this comment"), "comment survives");
        assert!(written.contains("[notes]"), "inert table survives");
        assert!(
            written.contains("info = \"survive\""),
            "inert table content survives"
        );
        // Y edge remains, X edge gone.
        assert!(
            written.contains("{ to = \"Y\", rank = 1 }"),
            "Y edge survives"
        );
        assert!(!written.contains("{ to = \"X\""), "X edge removed");
    }
}
