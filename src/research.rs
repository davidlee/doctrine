// SPDX-License-Identifier: GPL-3.0-only
//! Pre-design research staleness leaf (SL-229 D2/D5) — the mint/check engine
//! behind `doctrine slice research <id>`. A research artefact is baselined
//! against the slice's *intent* docs (scope / design / plan); when those drift
//! past the baseline the artefact may be stale and want a refresh round.
//!
//! ADR-001 leaf tier: the staleness core is the pure `ContentSet::diff` from the
//! `contentset` leaf; this module owns only the fixed baseline path set, the
//! `baseline.toml` serde shape, and two thin impure shells (`mint` writes it,
//! `check` reads-and-diffs it). No command-tier imports; fs lives at the edges,
//! and the date is passed in (date/uid pattern) so the logic stays clock-free.
//!
//! Absence is a defined state (inherited from `contentset::compute`): a baseline
//! path missing on disk is *omitted* from the set, not an error. Minting
//! pre-design records only the scope doc; `design.md`/`plan.*` appearing later
//! read as `added` drift — the lifecycle advance is itself the staleness signal.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::contentset::{self, ContentSet, SetDrift};

/// Slice-relative dir holding the research artefact + its baseline (STD-001).
/// Gitignored in place, riding the pre-existing SL-055 convention (D1).
pub(crate) const RESEARCH_DIR: &str = "research";

/// The staleness baseline file inside [`RESEARCH_DIR`] (STD-001).
pub(crate) const BASELINE_FILE: &str = "baseline.toml";

/// The `baseline.toml` document (D2): the durable id it belongs to, the mint
/// date, and the `[hashes]` `ContentSet` over the fixed path set — the
/// comparison key. `serde` round-trips all three; `hashes` is rebuilt from the
/// live path set on every mint/restamp so it cannot silently drift.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
struct Baseline {
    slice: String,
    date: String,
    #[serde(default)]
    hashes: BTreeMap<String, String>,
}

/// The research dir for a slice: `.doctrine/slice/NNN/research`.
pub(crate) fn research_dir(root: &Path, id: u32) -> PathBuf {
    root.join(crate::kinds::SLICE_DIR)
        .join(format!("{id:03}"))
        .join(RESEARCH_DIR)
}

/// The fixed v1 baseline path set (design A2): the slice's intent docs, as
/// repo-relative keys (what [`contentset::compute`] joins against `root`).
/// `plan.*` is included because criterion edits are frequently TOML-only and
/// `/phase-plan` is exactly the consumer that cares. Phase sheets and source
/// files are deliberately outside the domain — research goes stale against
/// intent, not implementation.
fn baseline_paths(id: u32) -> Vec<String> {
    let name = format!("{id:03}");
    let dir = format!("{}/{name}", crate::kinds::SLICE_DIR);
    vec![
        format!("{dir}/slice-{name}.md"),
        format!("{dir}/design.md"),
        format!("{dir}/plan.md"),
        format!("{dir}/plan.toml"),
    ]
}

/// Mint (or restamp) the baseline: create the research dir and write
/// `baseline.toml` recording the live hashes of the fixed path set. Idempotent
/// and overwriting — the sole writer for both the absent-dir mint and
/// `--restamp` re-baseline (D2). Returns the baseline path.
pub(crate) fn mint(root: &Path, id: u32, date: &str) -> anyhow::Result<PathBuf> {
    let dir = research_dir(root, id);
    std::fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let current = contentset::compute(root, &baseline_paths(id))
        .context("hash the slice's baseline path set")?;
    let baseline = Baseline {
        slice: format!("SL-{id:03}"),
        date: date.to_owned(),
        hashes: current.hashes().clone(),
    };
    let body = toml::to_string(&baseline).context("serialize research baseline")?;
    let path = dir.join(BASELINE_FILE);
    crate::fsutil::write_atomic(&path, body.as_bytes())?;
    Ok(path)
}

/// Diff the recorded baseline against the live path set — the drift advisory's
/// data (per-path changed / added / removed). Pure `ContentSet::diff` over the
/// impure `compute`; the caller decides fresh-vs-drift via `SetDrift::is_empty`.
pub(crate) fn check(root: &Path, id: u32) -> anyhow::Result<SetDrift> {
    let path = research_dir(root, id).join(BASELINE_FILE);
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("read research baseline {}", path.display()))?;
    let baseline: Baseline = toml::from_str(&text)
        .with_context(|| format!("parse research baseline {}", path.display()))?;
    let recorded = ContentSet::from_hashes(baseline.hashes);
    let live = contentset::compute(root, &baseline_paths(id))
        .context("hash the slice's baseline path set")?;
    Ok(recorded.diff(&live))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lay down a slice dir with the given intent docs written, returning the
    /// tempdir (kept alive by the caller). Files absent from `docs` stay absent.
    fn slice_tree(id: u32, docs: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let slice = dir
            .path()
            .join(crate::kinds::SLICE_DIR)
            .join(format!("{id:03}"));
        std::fs::create_dir_all(&slice).unwrap();
        for (name, body) in docs {
            std::fs::write(slice.join(name), body.as_bytes()).unwrap();
        }
        dir
    }

    #[test]
    fn mint_writes_baseline_that_round_trips_current() {
        let id = 229;
        let name = format!("{id:03}");
        let tree = slice_tree(
            id,
            &[
                (&format!("slice-{name}.md"), "scope"),
                ("design.md", "design"),
                ("plan.md", "plan prose"),
                ("plan.toml", "plan = 1"),
            ],
        );
        let path = mint(tree.path(), id, "2026-07-25").unwrap();
        assert_eq!(path.file_name().unwrap(), BASELINE_FILE);

        // Fresh immediately after mint: no drift against the live set.
        let drift = check(tree.path(), id).unwrap();
        assert!(
            drift.is_empty(),
            "fresh baseline should not drift: {drift:?}"
        );

        // The serialized fields are present and typed.
        let baseline: Baseline = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(baseline.slice, "SL-229");
        assert_eq!(baseline.date, "2026-07-25");
        assert_eq!(baseline.hashes.len(), 4);
    }

    #[test]
    fn absent_intent_doc_at_mint_reads_as_added_when_it_appears() {
        // Pre-design mint: only the scope doc exists; design.md/plan.* absent.
        let id = 42;
        let name = format!("{id:03}");
        let tree = slice_tree(id, &[(&format!("slice-{name}.md"), "scope")]);
        mint(tree.path(), id, "2026-07-25").unwrap();
        assert!(check(tree.path(), id).unwrap().is_empty());

        // design.md appears later — the lifecycle advance shows as `added` drift.
        let slice = tree
            .path()
            .join(crate::kinds::SLICE_DIR)
            .join(format!("{id:03}"));
        std::fs::write(slice.join("design.md"), b"now designed").unwrap();
        let drift = check(tree.path(), id).unwrap();
        assert!(!drift.is_empty());
        let added: Vec<&str> = drift.added.iter().map(String::as_str).collect();
        assert!(
            added.iter().any(|p| p.ends_with("design.md")),
            "design.md should be added drift: {drift:?}"
        );
    }

    #[test]
    fn changed_and_removed_intent_docs_drift() {
        let id = 7;
        let name = format!("{id:03}");
        let tree = slice_tree(
            id,
            &[
                (&format!("slice-{name}.md"), "scope v1"),
                ("design.md", "design v1"),
            ],
        );
        mint(tree.path(), id, "2026-07-25").unwrap();

        let slice = tree
            .path()
            .join(crate::kinds::SLICE_DIR)
            .join(format!("{id:03}"));
        // scope edited (changed); design deleted (removed).
        std::fs::write(slice.join(format!("slice-{name}.md")), b"scope v2").unwrap();
        std::fs::remove_file(slice.join("design.md")).unwrap();

        let drift = check(tree.path(), id).unwrap();
        assert!(
            drift
                .changed
                .iter()
                .any(|p| p.ends_with(&format!("slice-{name}.md")))
        );
        assert!(drift.removed.iter().any(|p| p.ends_with("design.md")));
        assert!(drift.added.is_empty());
    }

    #[test]
    fn restamp_overwrites_the_baseline_to_current() {
        let id = 7;
        let tree = slice_tree(id, &[("design.md", "v1")]);
        mint(tree.path(), id, "2026-07-25").unwrap();

        let slice = tree
            .path()
            .join(crate::kinds::SLICE_DIR)
            .join(format!("{id:03}"));
        std::fs::write(slice.join("design.md"), b"v2").unwrap();
        assert!(
            !check(tree.path(), id).unwrap().is_empty(),
            "v2 drifts pre-restamp"
        );

        // Restamp re-baselines against the live set; drift clears.
        mint(tree.path(), id, "2026-07-26").unwrap();
        assert!(
            check(tree.path(), id).unwrap().is_empty(),
            "restamp clears drift"
        );
    }
}
