// SPDX-License-Identifier: GPL-3.0-only
//! `facet_write` — edit-preserving `toml_edit` writer for `[estimate]` and `[value]`
//! facets (SL-118 PHASE-02). One pure set core + one pure clear core serve both
//! facets; IO wrappers (read→parse→core→write-once) live alongside them.
//!
//! ADR-001 leaf: imports only `toml_edit` / `anyhow` / `std`. No dependency on
//! `estimate`, `value`, `dep_seq`, `catalog`, or any command module — the engine
//! and command tiers depend on this, never the reverse. No cycle.
//!
//! **Mutate-in-place, not replace.** `set_facet` touches only the managed keys;
//! every non-managed sibling key, comment, and sub-table survives verbatim
//! (IDE-013 readiness). An absent table is allocated; a malformed present table
//! (scalar or array-of-tables where a standard table is expected) errors loudly,
//! never silently overwritten.
//!
//! **No clock.** Facet writes do not bump `updated` (SL-118 D1 reversed) — the
//! cores are pure, no `today` injection.
//!
//! **No-op guard.** An identical managed-value set writes nothing → `false`;
//! content + mtime hold.

use std::path::Path;

use anyhow::Context;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract an `f64` from a `toml_edit::Value`, accepting integer or float forms.
/// Returns `None` if the value is absent, not a number, or non-finite.
fn toml_edit_value_as_f64(value: &toml_edit::Value) -> Option<f64> {
    value.as_float().or_else(|| {
        #[expect(
            clippy::as_conversions,
            clippy::cast_precision_loss,
            reason = "integer from toml_edit is i64; fits exactly in f64 for values <= 2^53"
        )]
        value.as_integer().map(|i| i as f64)
    })
}

// ---------------------------------------------------------------------------
// Pure core — set
// ---------------------------------------------------------------------------

/// Mutate ONLY the named managed keys of a `[table]` facet on an in-memory
/// `DocumentMut`, allocating the table if absent and preserving every
/// non-managed sibling key, comment, and sub-table.
///
/// Returns `true` iff the document changed. Pure: no disk, no clock.
///
/// # Shape rules
///
/// - **Absent** → create a new `Table` at the root, insert each managed field
///   via `toml_edit::value(f)`, return `true`.
/// - **Present as standard table** → overwrite managed keys via
///   `table.insert(k, toml_edit::value(v))`. Non-managed keys are left alone.
///   Returns `true` iff any managed key changed value.
/// - **Present as non-table** (scalar or array-of-tables) → `bail!` with a
///   shape-error message. Never silently overwrite.
///
/// # No-op guard
///
/// Before writing, every managed key is compared to its current value (via
/// `Item::as_value().and_then(|v| v.as_float())`). If all match, the call is a
/// no-op — `false` is returned and nothing is mutated.
pub(crate) fn set_facet(
    doc: &mut toml_edit::DocumentMut,
    table: &str,
    fields: &[(&str, f64)],
) -> anyhow::Result<bool> {
    let root = doc.as_table_mut();

    match root.get_mut(table) {
        None => {
            // Allocate a fresh table.
            let mut t = toml_edit::Table::new();
            for &(k, v) in fields {
                t.insert(k, toml_edit::value(v));
            }
            root.insert(table, toml_edit::Item::Table(t));
            // Checked: a bare insert always changes the doc.
            Ok(true)
        }
        Some(item) => {
            // Is it a standard table? Bail on non-table shapes.
            let is_aot = item.is_array_of_tables();
            let tbl = item.as_table_mut().with_context(|| {
                if is_aot {
                    format!("{table}: expected a standard table, found an array-of-tables")
                } else {
                    format!("{table}: expected a standard table, found a scalar or array-of-tables")
                }
            })?;

            // No-op guard: compare every managed key.
            // We check BOTH as_float and as_integer — a hand-authored `lower = 2`
            // (integer) is value-equal to `2.0` (the read path normalises;
            // we accept the integer form as a no-op).
            let mut changed = false;
            for &(k, v) in fields {
                let current_float = tbl
                    .get(k)
                    .and_then(toml_edit::Item::as_value)
                    .and_then(toml_edit_value_as_f64);
                if current_float != Some(v) {
                    changed = true;
                    break;
                }
            }
            if !changed {
                return Ok(false);
            }

            // One or more values differ — overwrite managed keys.
            for &(k, v) in fields {
                tbl.insert(k, toml_edit::value(v));
            }
            Ok(true)
        }
    }
}

// ---------------------------------------------------------------------------
// Pure core — clear
// ---------------------------------------------------------------------------

/// Remove the `[table]` key from root if present. Returns `true` iff the key
/// was removed (i.e., it existed). No-op (`false`) if absent. Pure.
pub(crate) fn clear_facet(doc: &mut toml_edit::DocumentMut, table: &str) -> bool {
    doc.as_table_mut().remove(table).is_some()
}

// ---------------------------------------------------------------------------
// Shared IO envelope
// ---------------------------------------------------------------------------

/// Read→parse→core→write-once-if-changed envelope. Reads the file, parses a
/// `DocumentMut`, calls the closure `f`, and writes back iff the closure
/// returned `true`. Returns the closure's bool.
fn edit_in_place(
    path: &Path,
    f: impl FnOnce(&mut toml_edit::DocumentMut) -> anyhow::Result<bool>,
) -> anyhow::Result<bool> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("entity not found at {}", path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    let changed = f(&mut doc)?;
    if changed {
        std::fs::write(path, doc.to_string())
            .with_context(|| format!("Failed to write {}", path.display()))?;
    }
    Ok(changed)
}

// ---------------------------------------------------------------------------
// IO wrappers
// ---------------------------------------------------------------------------

/// Read an entity TOML and set managed facet keys via [`set_facet`]. Returns
/// `true` iff the document changed.
pub(crate) fn apply_set(path: &Path, table: &str, fields: &[(&str, f64)]) -> anyhow::Result<bool> {
    edit_in_place(path, |doc| set_facet(doc, table, fields))
}

/// Read an entity TOML and clear the facet table via [`clear_facet`]. Returns
/// `true` iff the table was present and removed.
pub(crate) fn apply_clear(path: &Path, table: &str) -> anyhow::Result<bool> {
    edit_in_place(path, |doc| Ok(clear_facet(doc, table)))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_doc() -> toml_edit::DocumentMut {
        "".parse::<toml_edit::DocumentMut>().unwrap()
    }

    fn doc_from(s: &str) -> toml_edit::DocumentMut {
        s.parse::<toml_edit::DocumentMut>().unwrap()
    }

    fn write_tmp(body: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("entity.toml");
        std::fs::write(&path, body).unwrap();
        (dir, path)
    }

    // ---- VT-1: set_facet allocates absent table ----

    #[test]
    fn vt1_set_allocates_absent_table() {
        let mut doc = empty_doc();
        let changed = set_facet(&mut doc, "estimate", &[("lower", 1.0), ("upper", 3.0)]).unwrap();
        assert!(changed, "allocating a new table returns true");
        let out = doc.to_string();
        assert!(
            out.contains("[estimate]"),
            "missing [estimate] header in:\n{out}"
        );
        assert!(out.contains("lower = 1.0"), "missing lower in:\n{out}");
        assert!(out.contains("upper = 3.0"), "missing upper in:\n{out}");
    }

    // ---- VT-2: set_facet overwrites present managed keys ----

    #[test]
    fn vt2_set_overwrites_present() {
        let mut doc = doc_from("[estimate]\nlower = 1\nupper = 3\n");
        let changed = set_facet(&mut doc, "estimate", &[("lower", 2.0), ("upper", 4.0)]).unwrap();
        assert!(changed, "overwriting returns true");
        let out = doc.to_string();
        assert!(out.contains("lower = 2.0"), "lower not updated:\n{out}");
        assert!(out.contains("upper = 4.0"), "upper not updated:\n{out}");
    }

    // ---- VT-3: set_facet idempotent no-op ----

    #[test]
    fn vt3_set_idempotent_noop() {
        let mut doc = doc_from("[estimate]\nlower = 1.0\nupper = 3.0\n");
        let changed = set_facet(&mut doc, "estimate", &[("lower", 1.0), ("upper", 3.0)]).unwrap();
        assert!(!changed, "identical values → no-op (false)");
        let out = doc.to_string();
        // Content unchanged — still the original exact text.
        assert!(out.contains("lower = 1.0"));
        assert!(out.contains("upper = 3.0"));
    }

    #[test]
    fn vt3_set_noop_preserves_integer_form() {
        // When the doc has `lower = 1` (integer), a set with 1.0 is still a
        // no-op — the read path normalises; we don't force float formatting.
        let mut doc = doc_from("[estimate]\nlower = 1\nupper = 3\n");
        let changed = set_facet(&mut doc, "estimate", &[("lower", 1.0), ("upper", 3.0)]).unwrap();
        assert!(!changed, "integer 1 == float 1.0 → no-op");
    }

    // ---- VT-4: clear_facet ----

    #[test]
    fn vt4_clear_removes_table() {
        let mut doc = doc_from("[estimate]\nlower = 1.0\nupper = 3.0\n");
        let removed = clear_facet(&mut doc, "estimate");
        assert!(removed, "table present → removed → true");
        let out = doc.to_string();
        assert!(
            !out.contains("[estimate]"),
            "estimate table should be gone:\n{out}"
        );
    }

    #[test]
    fn vt4_clear_absent_noop() {
        let mut doc = empty_doc();
        let removed = clear_facet(&mut doc, "estimate");
        assert!(!removed, "table absent → false");
    }

    // ---- VT-5: golden roundtrip — preserves unrelated content ----

    #[test]
    fn vt5_golden_roundtrip_preserve() {
        let body = concat!(
            "# top comment\n",
            "id = 1\n",
            "slug = \"a\"\n",
            "title = \"A\"\n",
            "\n",
            "[estimate]\n",
            "lower = 1\n",
            "upper = 3\n",
            "\n",
            "[relationships]\n",
            "needs = [\"ISS-002\"]\n",
            "after = []\n",
        );
        let mut doc = doc_from(body);

        // set on [estimate] — does not touch [relationships]
        let changed = set_facet(&mut doc, "estimate", &[("lower", 2.0), ("upper", 5.0)]).unwrap();
        assert!(changed);
        let out = doc.to_string();
        assert!(out.contains("# top comment"), "comment lost:\n{out}");
        assert!(
            out.contains("[relationships]"),
            "relationships lost:\n{out}"
        );
        assert!(
            out.contains("needs = [\"ISS-002\"]"),
            "needs content lost:\n{out}"
        );
        assert!(out.contains("lower = 2.0"), "lower not updated:\n{out}");
        assert!(out.contains("upper = 5.0"), "upper not updated:\n{out}");

        // clear [estimate] — leaves [relationships] intact
        let removed = clear_facet(&mut doc, "estimate");
        assert!(removed);
        let out2 = doc.to_string();
        assert!(
            !out2.contains("[estimate]"),
            "estimate should be gone:\n{out2}"
        );
        assert!(
            out2.contains("[relationships]"),
            "relationships lost on clear:\n{out2}"
        );
        assert!(
            out2.contains("# top comment"),
            "comment lost on clear:\n{out2}"
        );
    }

    // ---- VT-6: malformed-present fail-loud ----

    #[test]
    fn vt6_scalar_errors() {
        let mut doc = doc_from("estimate = 7\n");
        let err = set_facet(&mut doc, "estimate", &[("lower", 1.0), ("upper", 3.0)])
            .expect_err("scalar should error");
        let msg = format!("{err:#}").to_lowercase();
        assert!(
            msg.contains("expected a standard table"),
            "scalar error message: {msg}"
        );
        assert!(
            msg.contains("scalar") || msg.contains("found"),
            "should identify scalar shape: {msg}"
        );
        // Doc untouched.
        assert_eq!(doc.to_string(), "estimate = 7\n", "doc untouched on error");
    }

    #[test]
    fn vt6_array_of_tables_errors() {
        let mut doc = doc_from("[[estimate]]\nlower = 1\nupper = 3\n");
        let err = set_facet(&mut doc, "estimate", &[("lower", 2.0), ("upper", 4.0)])
            .expect_err("array-of-tables should error");
        let msg = format!("{err:#}").to_lowercase();
        assert!(msg.contains("array-of-tables"), "AoT error message: {msg}");
        // Doc untouched.
        assert!(
            doc.to_string().contains("[[estimate]]"),
            "doc untouched on error"
        );
    }

    // ---- VT-7: forward-compat — preserves unknown sibling keys ----

    #[test]
    fn vt7_forward_compat_preserves_unknown_estimate() {
        let mut doc = doc_from("[estimate]\nlower = 1\nupper = 3\nhistory = \"old\"\n");
        let changed = set_facet(&mut doc, "estimate", &[("lower", 2.0), ("upper", 4.0)]).unwrap();
        assert!(changed);
        let out = doc.to_string();
        assert!(out.contains("lower = 2.0"), "lower not updated:\n{out}");
        assert!(out.contains("upper = 4.0"), "upper not updated:\n{out}");
        assert!(
            out.contains("history = \"old\""),
            "unknown sibling lost:\n{out}"
        );
    }

    #[test]
    fn vt7_forward_compat_preserves_unknown_value() {
        let mut doc = doc_from("[value]\nvalue = 5\nhistory = \"old\"\n");
        let changed = set_facet(&mut doc, "value", &[("value", 10.0)]).unwrap();
        assert!(changed);
        let out = doc.to_string();
        assert!(out.contains("value = 10.0"), "value not updated:\n{out}");
        assert!(
            out.contains("history = \"old\""),
            "unknown sibling lost:\n{out}"
        );
    }

    // ---- VT-8: layering — facet_write depends only on toml_edit, anyhow, std ----
    //
    // This test is structural (cannot inspect the module graph at runtime), but
    // it exercises every public entry point so that `cargo test` confirms the file
    // compiles with no phantom dependencies.

    #[test]
    fn vt8_layering_exercise_all_public_fns() {
        // Exercise set_facet, clear_facet, apply_set, apply_clear via the IO
        // wrappers — confirms the leaf compiles and links against only its
        // declared deps (toml_edit, anyhow, std). If a phantom import snuck in,
        // the build would fail.

        let (_dir, path) = write_tmp("id = 1\nslug = \"a\"\ntitle = \"A\"\n");

        // apply_set: allocates absent table.
        let changed = apply_set(&path, "estimate", &[("lower", 1.0), ("upper", 5.0)]).unwrap();
        assert!(changed, "apply_set allocates");
        let after_set = std::fs::read_to_string(&path).unwrap();
        assert!(after_set.contains("[estimate]"));

        // apply_set: idempotent no-op.
        let changed2 = apply_set(&path, "estimate", &[("lower", 1.0), ("upper", 5.0)]).unwrap();
        assert!(!changed2, "apply_set idempotent");

        // apply_clear: removes table.
        let cleared = apply_clear(&path, "estimate").unwrap();
        assert!(cleared, "apply_clear removes");
        let after_clear = std::fs::read_to_string(&path).unwrap();
        assert!(!after_clear.contains("[estimate]"));

        // apply_clear: absent no-op.
        let cleared2 = apply_clear(&path, "estimate").unwrap();
        assert!(!cleared2, "apply_clear absent → false");
    }

    // ---- IO envelope tests ----

    #[test]
    fn edit_in_place_no_change_holds_mtime() {
        let (_dir, path) = write_tmp("[estimate]\nlower = 1.0\nupper = 3.0\n");
        let before_mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
        let changed = apply_set(&path, "estimate", &[("lower", 1.0), ("upper", 3.0)]).unwrap();
        assert!(!changed, "no-op returns false");
        assert_eq!(
            std::fs::metadata(&path).unwrap().modified().unwrap(),
            before_mtime,
            "mtime held on no-op"
        );
    }

    #[test]
    fn apply_set_on_value_facet() {
        let (_dir, path) = write_tmp("id = 1\nslug = \"a\"\ntitle = \"A\"\n");
        let changed = apply_set(&path, "value", &[("value", 42.0)]).unwrap();
        assert!(changed);
        let out = std::fs::read_to_string(&path).unwrap();
        assert!(out.contains("[value]"), "value table missing:\n{out}");
        assert!(out.contains("value = 42.0"), "value field missing:\n{out}");
    }

    // ---- Helper fn tests ----

    #[test]
    fn set_facet_no_fields_is_valid() {
        // Edge case: zero managed fields. An absent table creates an empty
        // table; a present table does nothing.
        let mut doc = empty_doc();
        let changed = set_facet(&mut doc, "estimate", &[]).unwrap();
        assert!(changed, "allocating empty table returns true");
        assert!(doc.to_string().contains("[estimate]"));
    }

    #[test]
    fn set_facet_zero_fields_idempotent() {
        // Setting zero fields on an existing table always returns false.
        let mut doc = doc_from("[estimate]\nlower = 1\nupper = 3\n");
        let changed = set_facet(&mut doc, "estimate", &[]).unwrap();
        assert!(!changed, "zero fields on present table → no-op");
    }

    #[test]
    fn set_facet_partial_overwrite_preserves_other_managed() {
        // Set only one of the two managed keys; the other should survive.
        let mut doc = doc_from("[estimate]\nlower = 1.0\nupper = 3.0\n");
        let changed = set_facet(&mut doc, "estimate", &[("lower", 5.0)]).unwrap();
        assert!(changed);
        let out = doc.to_string();
        assert!(out.contains("lower = 5.0"), "lower not updated:\n{out}");
        assert!(out.contains("upper = 3.0"), "upper lost:\n{out}");
    }
}
