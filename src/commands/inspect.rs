// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine inspect` — cross-kind entity inspection with relation + actionability views.
//! SL-129: uses `entity::id_path`, `relation_graph`, `priority`

use crate::catalog::scan::ScanMode;
use crate::listing::Format;

/// `doctrine inspect <ID>` — cross-kind relation tree + actionability block.
/// The one layer allowed to depend on BOTH `relation_graph` and `priority`
/// (ADR-001 — `relation_graph` sits below `priority` and must never call up into it).
/// The relation portion stays byte-identical; the actionability block is purely additive (EX-2).
///
/// - **human**: the relation render with the actionability block appended below.
/// - **`--json`**: the inspect envelope with an additive `"actionability"` key —
///   the relation surfaces (`outbound`/`inbound`/`danglers`) unchanged.
pub(crate) fn run_inspect(
    path: Option<std::path::PathBuf>,
    id: &str,
    format: Format,
    json: bool,
) -> anyhow::Result<()> {
    use std::io::{self, Write};
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let resolved = if json { Format::Json } else { format };

    if let Ok(
        crate::memory::MemoryRef::Uid(_)
        | crate::memory::MemoryRef::UidPrefix(_)
        | crate::memory::MemoryRef::Key(_),
    ) = crate::memory::MemoryRef::parse(id)
    {
        let uid = crate::memory::resolve_inspect_uid(&root, id)?;
        let out = crate::memory::memory_inspect_view(&root, &uid, resolved)?;
        write!(std::io::stdout(), "{out}")?;
        return Ok(());
    }

    // SL-050 F2: ONE corpus scan shared by both consumers (was two — relation_graph and
    // priority each walked the corpus). Both `_from` entry points consume this slice;
    // the scan order is the same both saw (KINDS table / id ascending), preserving
    // REQ-077 determinism and the byte-identical relation/priority surfaces (VT-4).
    let mut diagnostics = Vec::new();
    let scanned =
        crate::relation_graph::scan_entities(&root, &mut diagnostics, ScanMode::default())?;
    // Surface scan degradation diagnostics to stderr before normal output (D3).
    for diag in &diagnostics {
        writeln!(io::stderr(), "{}: {}", diag.file.display(), diag.message)?;
    }

    let out = match resolved {
        Format::Table => {
            // Relation render FIRST (the cheap oracle): its F6 existence gate (inside
            // render_from → inspect_from on the relation projection) errors a ghost id
            // BEFORE the heavier priority block is built.
            let relation = crate::relation_graph::render_from(&scanned, &root, id, Format::Table)?;
            // Only reached for a minted id (the render gate passed).
            let block = crate::priority::surface::actionability_block_from(&scanned, &root, id)?;
            let block = crate::priority::render::actionability_block_human(&block);
            format!("{relation}{block}")
        }
        Format::Json => {
            // Relation view + gate FIRST, then the priority block (gate inside
            // inspect_from on the relation projection).
            let view = crate::relation_graph::inspect_from(&scanned, &root, id)?;
            let block = crate::priority::surface::actionability_block_from(&scanned, &root, id)?;
            let mut value = crate::relation_graph::inspect_value(&view);
            if let Some(obj) = value.as_object_mut() {
                obj.insert(
                    "actionability".to_string(),
                    crate::priority::render::actionability_block_value(&block),
                );
            }
            serde_json::to_string_pretty(&value)
                .map_err(|e| anyhow::anyhow!("failed to serialize inspect JSON: {e}"))?
        }
    };
    write!(std::io::stdout(), "{out}")?;
    Ok(())
}
