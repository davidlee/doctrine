// SPDX-License-Identifier: GPL-3.0-only
//! Content-set staleness leaf (SL-040 D3, IMP-025 candidate) — a pure,
//! consumer-agnostic primitive: an ordered `root-relative path → content-hash`
//! map plus the pure `diff` / `is_stale_against` comparison, and one thin impure
//! shell (`compute`) that reads bytes off disk and hashes them.
//!
//! ADR-001 leaf tier: the comparison core takes maps as inputs — no disk, git,
//! clock, or rng. `compute` is the lone impure seam (a directory read + a
//! `sha2` hash). It owns `sha2` directly rather than depending on the impure
//! `git.rs` seam, so the leaf stays liftable (D3); the trivial two-call-site
//! hash duplication is accepted.
//!
//! Absence is a defined state, not an error (R1, design §9): `compute` *omits*
//! an absent path from the resulting set, so a baseline that recorded it sees
//! it as `removed` under `diff` — which makes the baseline stale. Other IO
//! errors (a permission failure, say) propagate; only `NotFound` is treated as
//! omission.
//!
//! Built ahead of its consumers (SL-040 PHASE-01): the first non-test caller is
//! the warm-cache in PHASE-05 (`prime`/`review status`). The module-level
//! `expect(dead_code)` is self-clearing — it errors the moment a consumer lands,
//! forcing its removal (mem.pattern.lint.dead-code-self-clearing-leaf).
//! The suppression is `cfg_attr(not(test), …)` so the test round-trips (real
//! uses) do not leave the expectation unfulfilled in the test build
//! (mem.pattern.lint.dead-code-expect-vs-cfg-test).
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "pure leaf stood up in SL-040 PHASE-01; first non-test consumer (warm-cache prime/status) lands in PHASE-05 — self-clearing"
    )
)]

use std::collections::BTreeMap;
use std::io;
use std::path::Path;

use sha2::{Digest, Sha256};

/// An ordered map of root-relative path → lowercase-hex content hash. The order
/// is an invariant (`BTreeMap`) so `diff` output is deterministic regardless of
/// insertion order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ContentSet(BTreeMap<String, String>);

/// The drift of a baseline `ContentSet` against a current one, partitioned into
/// three disjoint path classes. All three empty ⇔ the sets are identical.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SetDrift {
    /// Present in both, but the hash differs.
    pub(crate) changed: Vec<String>,
    /// Present in `current`, absent from the baseline (`self`).
    pub(crate) added: Vec<String>,
    /// Present in the baseline (`self`), absent from `current` — the
    /// absence⇒stale class (R1).
    pub(crate) removed: Vec<String>,
}

impl SetDrift {
    /// Whether any class is non-empty — i.e. the two sets differ at all.
    fn is_empty(&self) -> bool {
        self.changed.is_empty() && self.added.is_empty() && self.removed.is_empty()
    }
}

impl ContentSet {
    /// Build a set directly from `(path, hash)` pairs (test/seed convenience).
    #[cfg(test)]
    fn from_pairs<I, S>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (S, S)>,
        S: Into<String>,
    {
        Self(
            pairs
                .into_iter()
                .map(|(p, h)| (p.into(), h.into()))
                .collect(),
        )
    }

    /// Diff this baseline against `current`. Pure and total: every path in
    /// either set lands in exactly one class — `changed` (hash mismatch),
    /// `added` (only in `current`), or `removed` (only in the baseline). Class
    /// vectors are path-sorted (the `BTreeMap` walk order).
    pub(crate) fn diff(&self, current: &ContentSet) -> SetDrift {
        let mut drift = SetDrift::default();
        for (path, hash) in &self.0 {
            match current.0.get(path) {
                Some(current_hash) if current_hash == hash => {}
                Some(_) => drift.changed.push(path.clone()),
                None => drift.removed.push(path.clone()),
            }
        }
        for path in current.0.keys() {
            if !self.0.contains_key(path) {
                drift.added.push(path.clone());
            }
        }
        drift
    }

    /// Whether this baseline is stale against `current` — i.e. any path
    /// changed, was added, or was removed. Absence of a recorded path in
    /// `current` (a `removed`) counts as stale (R1).
    pub(crate) fn is_stale_against(&self, current: &ContentSet) -> bool {
        !self.diff(current).is_empty()
    }
}

/// Impure shell: read each of `paths` (root-relative) under `root`, hash its
/// bytes, and collect them into a `ContentSet`. An absent path (`NotFound`) is
/// *omitted* (R1 absence⇒stale, resolved by the omission) rather than failing;
/// any other IO error propagates. The hash is the lowercase-hex `sha256` of the
/// file's raw bytes (matches `git::sha256`, owned here to keep the leaf liftable
/// — D3).
pub(crate) fn compute(root: &Path, paths: &[String]) -> io::Result<ContentSet> {
    let mut set = BTreeMap::new();
    for rel in paths {
        match std::fs::read(root.join(rel)) {
            Ok(bytes) => {
                set.insert(rel.clone(), sha256_hex(&bytes));
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
    }
    Ok(ContentSet(set))
}

/// Lowercase-hex `sha256` of `bytes`. Owned directly (D3) so the leaf does not
/// depend on the impure `git.rs` seam; byte-identical to `git::sha256`.
fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(pairs: &[(&str, &str)]) -> ContentSet {
        ContentSet::from_pairs(pairs.iter().map(|&(p, h)| (p, h)))
    }

    #[test]
    fn diff_identical_sets_is_empty() {
        let a = set(&[("a", "1"), ("b", "2")]);
        assert!(!a.is_stale_against(&a));
        assert_eq!(a.diff(&a), SetDrift::default());
    }

    #[test]
    fn diff_classifies_changed_added_removed() {
        let base = set(&[("keep", "1"), ("mutate", "2"), ("gone", "3")]);
        let current = set(&[("keep", "1"), ("mutate", "9"), ("fresh", "7")]);
        let drift = base.diff(&current);
        assert_eq!(drift.changed, vec!["mutate".to_owned()]);
        assert_eq!(drift.added, vec!["fresh".to_owned()]);
        assert_eq!(drift.removed, vec!["gone".to_owned()]);
        assert!(base.is_stale_against(&current));
    }

    #[test]
    fn absent_path_is_removed_and_therefore_stale() {
        // R1: a recorded path missing from `current` ⇒ removed ⇒ stale.
        let base = set(&[("present", "1"), ("absent", "2")]);
        let current = set(&[("present", "1")]);
        let drift = base.diff(&current);
        assert_eq!(drift.removed, vec!["absent".to_owned()]);
        assert!(drift.changed.is_empty() && drift.added.is_empty());
        assert!(base.is_stale_against(&current));
    }

    #[test]
    fn diff_class_vectors_are_path_sorted() {
        let base = set(&[("z", "1"), ("a", "1")]);
        let current = set(&[("z", "9"), ("a", "9")]);
        assert_eq!(
            base.diff(&current).changed,
            vec!["a".to_owned(), "z".to_owned()]
        );
    }

    #[test]
    fn compute_omits_absent_path_yielding_stale_baseline() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("here.txt"), b"hello").unwrap();
        // Baseline recorded both; only `here.txt` exists on disk now.
        let current = compute(
            dir.path(),
            &["here.txt".to_owned(), "missing.txt".to_owned()],
        )
        .unwrap();
        // The absent path is omitted, not an error.
        let present_only = compute(dir.path(), &["here.txt".to_owned()]).unwrap();
        assert_eq!(current, present_only);

        // A baseline that recorded the now-absent path is stale against current.
        let baseline = compute(dir.path(), &["here.txt".to_owned()]).unwrap();
        let baseline_with_extra = {
            let mut s = baseline.0.clone();
            s.insert("missing.txt".to_owned(), "deadbeef".to_owned());
            ContentSet(s)
        };
        assert!(baseline_with_extra.is_stale_against(&current));
        assert_eq!(
            baseline_with_extra.diff(&current).removed,
            vec!["missing.txt".to_owned()]
        );
    }

    #[test]
    fn compute_hash_matches_sha256_of_bytes() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("f"), b"content").unwrap();
        let cs = compute(dir.path(), &["f".to_owned()]).unwrap();
        assert_eq!(cs.0.get("f").unwrap(), &sha256_hex(b"content"));
    }

    #[test]
    fn compute_propagates_non_notfound_io_error() {
        // A path that resolves to a directory is not NotFound — reading it errors.
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("sub")).unwrap();
        let err = compute(dir.path(), &["sub".to_owned()]).unwrap_err();
        assert_ne!(err.kind(), io::ErrorKind::NotFound);
    }
}
