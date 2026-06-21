// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine tag` — generic cross-kind tag verb (SL-136 PHASE-02).
//!
//! Thin command-tier shell over the shared leaf `tag::apply_tags_set`. Resolution,
//! taggability gating, normalisation, and overlap rejection are all handled here;
//! the write core is pure and kind-agnostic.

use std::collections::BTreeSet;
use std::io::Write;

use anyhow::Context;

use crate::tag::{self, normalize_tag};

#[derive(Debug, clap::Subcommand)]
pub(crate) enum TagCommand {
    /// Add and/or remove tags on an entity (additive-merge).
    Set {
        /// Canonical ref to tag, e.g. SL-136.
        reference: String,

        /// Tags to add (normalised: trimmed, lowercased, [a-z0-9_:-]).
        tags: Vec<String>,

        /// Tags to remove.
        #[arg(short = 'd', long)]
        remove: Vec<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<std::path::PathBuf>,
    },

    /// Remove all tags from an entity.
    Clear {
        /// Canonical ref to tag, e.g. SL-136.
        reference: String,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<std::path::PathBuf>,
    },
}

/// Dispatch the cross-kind tag verb.
pub(crate) fn dispatch(cmd: TagCommand) -> anyhow::Result<()> {
    match cmd {
        TagCommand::Set {
            reference,
            tags: adds,
            remove: removes,
            path,
        } => run_set(path, &reference, &adds, &removes),
        TagCommand::Clear { reference, path } => run_clear(path, &reference),
    }
}

fn run_set(
    path: Option<std::path::PathBuf>,
    reference: &str,
    adds: &[String],
    removes: &[String],
) -> anyhow::Result<()> {
    // A5: at least one add or remove.
    if adds.is_empty() && removes.is_empty() {
        anyhow::bail!("`doctrine tag set` needs at least one tag to add or remove (--remove/-d)");
    }

    let add_set: BTreeSet<String> = adds
        .iter()
        .map(|t| normalize_tag(t))
        .collect::<anyhow::Result<_>>()?;
    let remove_set: BTreeSet<String> = removes
        .iter()
        .map(|t| normalize_tag(t))
        .collect::<anyhow::Result<_>>()?;

    // Overlap reject.
    let overlap: Vec<&String> = add_set.intersection(&remove_set).collect();
    if let Some(first) = overlap.first() {
        anyhow::bail!("tag `{first}` is in both add and remove (pick one)");
    }

    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (kref, id) = {
        let (k, i) = crate::integrity::parse_canonical_ref(reference).map_err(|e| {
            anyhow::anyhow!(
                "{e}\n  hint: `doctrine tag` works on numbered entity refs (e.g. SL-136, ADR-004). \
                 Use `doctrine memory tag` for memories."
            )
        })?;
        // Taggability gate.
        anyhow::ensure!(
            tag::TAGGABLE.contains(&k.kind.prefix),
            "{} is not taggable yet (see IMP-144)",
            k.kind.prefix
        );
        crate::integrity::ensure_ref_resolves(&root, reference)?;
        (k, i)
    };
    let item_path = crate::entity::id_path(&root, kref.kind, id, crate::entity::Ext::Toml);

    let text = std::fs::read_to_string(&item_path)
        .with_context(|| format!("Failed to read {}", item_path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", item_path.display()))?;

    let changed = tag::apply_tags_set(&mut doc, &add_set, &remove_set, &crate::clock::today())?;
    if changed {
        crate::fsutil::write_atomic(&item_path, doc.to_string().as_bytes())
            .with_context(|| format!("Failed to write {}", item_path.display()))?;
    }

    print_post_state(&doc, reference);
    Ok(())
}

fn run_clear(path: Option<std::path::PathBuf>, reference: &str) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let (kref, id) = {
        let (k, i) = crate::integrity::parse_canonical_ref(reference)?;
        anyhow::ensure!(
            tag::TAGGABLE.contains(&k.kind.prefix),
            "{} is not taggable yet (see IMP-144)",
            k.kind.prefix
        );
        crate::integrity::ensure_ref_resolves(&root, reference)?;
        (k, i)
    };
    let item_path = crate::entity::id_path(&root, kref.kind, id, crate::entity::Ext::Toml);

    let text = std::fs::read_to_string(&item_path)
        .with_context(|| format!("Failed to read {}", item_path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", item_path.display()))?;

    // Read current tags, pass them as removes.
    let current_tags: Vec<String> = doc
        .as_table()
        .get("tags")
        .and_then(toml_edit::Item::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let remove_set: BTreeSet<String> = current_tags.iter().cloned().collect();
    let empty_adds = BTreeSet::new();

    let changed = tag::apply_tags_set(&mut doc, &empty_adds, &remove_set, &crate::clock::today())?;
    if changed {
        crate::fsutil::write_atomic(&item_path, doc.to_string().as_bytes())
            .with_context(|| format!("Failed to write {}", item_path.display()))?;
    }

    print_post_state(&doc, reference);
    Ok(())
}

fn print_post_state(doc: &toml_edit::DocumentMut, reference: &str) {
    let final_tags: Vec<String> = doc
        .as_table()
        .get("tags")
        .and_then(toml_edit::Item::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let listed = if final_tags.is_empty() {
        "(none)".to_string()
    } else {
        final_tags.join(", ")
    };
    // Best-effort print; a broken pipe is harmless for a CLI display.
    _ = writeln!(std::io::stdout(), "Tagged {reference}: {listed}");
}
