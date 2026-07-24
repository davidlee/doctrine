// SPDX-License-Identifier: GPL-3.0-only
//! Reference probes over the [`KINDS`] table (SL-204 PHASE-03, design D2): resolve
//! a canonical prefix to its [`KindRef`], parse canonical / resolvable refs, the
//! read-only forward-edge guard, and the scanned-roster summary. All leaf —
//! `KINDS`/`KindRef` from the parent module, `fsutil` a leaf sibling.
//!
//! `kinds` owns the canonical-id **format** authority: [`canonical_id`] (the inverse
//! of [`parse_canonical_ref`]) is the single source of the `PREFIX-NNN` form, and
//! `listing::canonical_id` delegates down to it (SL-204 PHASE-04). The edge runs
//! `listing → kinds`, not `kinds → listing`: the latter would close a pre-existing
//! leaf cycle (`listing → tag → kinds`, from tag's `TAGGABLE` re-export) the
//! architecture-layering ratchet forbids, but `kinds` imports neither `listing` nor
//! `tag`, so the downward edge is cycle-free.

use std::path::Path;

use anyhow::Context;

use super::{KINDS, KindRef};
use crate::fsutil;

/// Resolve a numbered kind by its canonical prefix (`SL` → the slice [`KindRef`]).
pub(crate) fn kind_by_prefix(prefix: &str) -> Option<&'static KindRef> {
    KINDS.iter().find(|k| k.kind.prefix == prefix)
}

/// Validate that a canonical ref (`SL-024`) resolves to a real entity on disk —
/// the forward-edge guard a kind reuses at authoring time (SL-040 §7: `review
/// new` refuses a dangling / unknown-prefix `[target].ref` before minting an RV).
/// Two failure modes, both surfaced by [`parse_canonical_ref`] + a dir probe: an
/// unknown prefix / non-canonical shape, and a well-formed ref to an id with no
/// entity directory (dangling). Read-only.
pub(crate) fn ensure_ref_resolves(root: &Path, reference: &str) -> anyhow::Result<()> {
    parse_resolvable_ref(root, reference)?;
    Ok(())
}

/// The canonical-id **format** authority: `SL` + `31` → `"SL-031"`, zero-padded to
/// three digits. The symmetric inverse of [`parse_canonical_ref`]; `listing::canonical_id`
/// delegates here (SL-204 PHASE-04).
pub(crate) fn canonical_id(prefix: &str, id: u32) -> String {
    format!("{prefix}-{id:03}")
}

/// Parse a canonical ref (`SL-031`) into its kind and numeric id. Reseat keys on
/// the canonical ref, never a bare number (X2/D7) — the kind disambiguates the
/// per-namespace id.
pub(crate) fn parse_canonical_ref(reference: &str) -> anyhow::Result<(&'static KindRef, u32)> {
    let (prefix, num) = reference
        .rsplit_once('-')
        .with_context(|| format!("`{reference}` is not a canonical ref (expected e.g. SL-031)"))?;
    let kind = kind_by_prefix(prefix)
        .with_context(|| format!("unknown kind prefix `{prefix}` in `{reference}`"))?;
    let id = num
        .parse::<u32>()
        .with_context(|| format!("`{num}` is not a numeric id in `{reference}`"))?;
    Ok((kind, id))
}

/// Parse an entity reference, accepting both canonical (`SL-031`) and bare (`31`)
/// forms. For bare numbers, scans all entity kinds to find the matching entity
/// directory. Returns the resolved kind and numeric id.
pub(crate) fn parse_resolvable_ref(
    root: &Path,
    reference: &str,
) -> anyhow::Result<(&'static KindRef, u32)> {
    // Canonical form (has a hyphen) — parse, then verify the entity exists.
    if reference.contains('-') {
        let (kref, id) = parse_canonical_ref(reference)?;
        let name = format!("{id:03}");
        let dir = root.join(kref.kind.dir).join(&name);
        anyhow::ensure!(
            fsutil::is_real_dir(&dir),
            "`{reference}` does not resolve to an entity (no {} at {})",
            canonical_id(kref.kind.prefix, id),
            dir.display()
        );
        return Ok((kref, id));
    }
    // Bare number — scan all kinds for matching entity directories.
    let id: u32 = reference.parse().with_context(|| {
        format!("`{reference}` is not a valid entity reference (expected e.g. SL-031 or 31)")
    })?;
    let name = format!("{id:03}");
    let mut matches: Vec<&'static KindRef> = Vec::new();
    for kref in KINDS {
        let dir = root.join(kref.kind.dir).join(&name);
        if fsutil::is_real_dir(&dir) {
            matches.push(kref);
        }
    }
    match matches.as_slice() {
        [] => anyhow::bail!(
            "no entity found for bare id `{reference}` — try the prefixed form (e.g. SL-{name})"
        ),
        [single] => Ok((single, id)),
        many => {
            let ids: Vec<String> = many
                .iter()
                .map(|k| canonical_id(k.kind.prefix, id))
                .collect();
            anyhow::bail!(
                "bare id `{reference}` is ambiguous — matches {}; use the prefixed form",
                ids.join(", ")
            )
        }
    }
}

/// The roster of kinds `validate` scanned, for the summary line (so the memory
/// omission is visible, D-A).
pub(crate) fn scanned_kinds() -> String {
    KINDS
        .iter()
        .map(|k| k.kind.prefix)
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SL-040 §7: `ensure_ref_resolves` accepts a real target, refuses a dangling
    /// (well-formed but absent) ref and an unknown-prefix ref — the `review new`
    /// forward-edge guard.
    #[test]
    fn ensure_ref_resolves_guards_the_forward_edge() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let dir = root.join(".doctrine/slice/024");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("slice-024.toml"), "id = 24\n").unwrap();

        assert!(ensure_ref_resolves(root, "SL-024").is_ok());
        let dangling = ensure_ref_resolves(root, "SL-099").unwrap_err();
        assert!(
            dangling.to_string().contains("does not resolve"),
            "{dangling}"
        );
        let unknown = ensure_ref_resolves(root, "ZZ-001").unwrap_err();
        assert!(
            unknown.to_string().contains("unknown kind prefix"),
            "{unknown}"
        );
    }

    /// SL-204 PHASE-04: `canonical_id` is the format authority and the inverse of
    /// `parse_canonical_ref` — composing them round-trips prefix and id.
    #[test]
    fn canonical_id_round_trips_through_parse_canonical_ref() {
        assert_eq!(canonical_id("ADR", 1), "ADR-001");
        let (kind, id) = parse_canonical_ref(&canonical_id("SL", 31)).expect("round-trip");
        assert_eq!(kind.kind.prefix, "SL");
        assert_eq!(id, 31);
    }

    #[test]
    fn parse_canonical_ref_resolves_kind_and_id() {
        let (kind, id) = parse_canonical_ref("SL-031").expect("valid ref");
        assert_eq!(kind.kind.prefix, "SL");
        assert_eq!(id, 31);
        assert!(
            parse_canonical_ref("031").is_err(),
            "bare id is not canonical"
        );
        assert!(
            parse_canonical_ref("ZZ-001").is_err(),
            "unknown prefix rejected"
        );
        assert!(
            parse_canonical_ref("SL-x").is_err(),
            "non-numeric id rejected"
        );
    }
}
