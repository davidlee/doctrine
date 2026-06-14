// SPDX-License-Identifier: GPL-3.0-only
//! Pure dep/sequence schema + edit-preserving write leaf (SL-060 PHASE-02). The
//! shared substrate for an entity's `[relationships].{needs,after}` axes — the
//! `needs` hard-prerequisite list and the `after` soft-sequence edges (each with a
//! per-edge `rank`). Lifted verbatim from `backlog.rs` (the SL-047 origin) so a
//! later phase can reuse the SAME typed schema and the SAME strict edit-preserving
//! append for slices (design D1/§5.2), without a parallel implementation.
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
                // idempotent: skip a ref already in the array.
                if array.iter().any(|v| v.as_str() == Some(r.as_str())) {
                    continue;
                }
                array.push(r.as_str());
                changed = true;
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
    std::fs::write(toml_path, doc.to_string())
        .with_context(|| format!("Failed to write {}", toml_path.display()))?;
    Ok(())
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
}
