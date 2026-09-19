// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine inspect` — cross-kind entity inspection with relation + actionability views.
//! SL-129: uses `entity::id_path`, `relation_graph`, `priority`

use crate::catalog::scan::ScanMode;
use crate::listing::Format;

/// Bundled `inspect` arguments — a struct keeps `run_inspect` a two-arg handler, so
/// the four SL-138 transitive flags do not trip `clippy::too_many_arguments`.
///
/// `direction` is the engine `TransitiveDir`, mapped DOWN from the clap `DirArg` at
/// the dispatch boundary (ADR-001).
pub(crate) struct InspectArgs<'a> {
    pub id: &'a str,
    pub format: Format,
    pub json: bool,
    pub transitive: bool,
    pub direction: crate::relation_graph::TransitiveDir,
    pub labels: Vec<String>,
    pub max_depth: Option<String>,
    /// SL-246: how much of each inbound knowledge record the composed read
    /// carries. `Skip` (the default) leaves the surface byte-identical to the
    /// pre-SL-246 output.
    pub knowledge: crate::knowledge::KnowledgeLevel,
}

/// Parse `--max-depth` (design §5 / EX-2): absent → `Some(5)` (the uniform default);
/// `0` or `all` → `None` (unbounded); `N` → `Some(N)`. A non-integer is a clean error.
fn parse_max_depth(raw: Option<&str>) -> anyhow::Result<Option<usize>> {
    match raw {
        None => Ok(Some(5)),
        Some("0" | "all") => Ok(None),
        Some(s) => {
            let n = s.parse::<usize>().map_err(|_err| {
                anyhow::anyhow!("--max-depth must be a non-negative integer or `all`, got `{s}`")
            })?;
            Ok(Some(n))
        }
    }
}

/// `doctrine inspect <ID>` — cross-kind relation tree + actionability block (the
/// default 1-hop view), or the SL-138 relation-only transitive walk when
/// `args.transitive`. The one layer allowed to depend on BOTH `relation_graph` and
/// `priority` (ADR-001 — `relation_graph` sits below `priority` and must never call up
/// into it). The relation portion stays byte-identical; the actionability block is
/// purely additive (EX-2).
///
/// - **human**: the relation render with the actionability block appended below.
/// - **`--json`**: the inspect envelope with an additive `"actionability"` key —
///   the relation surfaces (`outbound`/`inbound`/`danglers`) unchanged.
///
/// # Errors
///
/// A malformed / never-minted id, a memory ref combined with `--transitive` (F2), a
/// non-overlay `--labels` entry, or an unparseable `--max-depth`.
pub(crate) fn run_inspect(
    path: Option<std::path::PathBuf>,
    args: &InspectArgs<'_>,
) -> anyhow::Result<()> {
    use std::io::{self, Write};
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let resolved = if args.json { Format::Json } else { args.format };
    let id = args.id;

    // SL-246 EX-2: a non-Skip `--knowledge` level composes the inbound knowledge
    // block over the 1-hop relation view; `--transitive` walks N hops with no
    // block at all. The two are mutually exclusive because the transitive
    // knowledge closure is not built (IMP-398 S5) — sited first, ahead of the
    // SL-138 F2 memory-ref refusal below, for deterministic precedence.
    if !matches!(args.knowledge, crate::knowledge::KnowledgeLevel::Skip) && args.transitive {
        anyhow::bail!(
            "--knowledge {} cannot be combined with --transitive: the transitive \
             knowledge closure is not built (IMP-398 S5). Use --knowledge skip, or \
             drop --transitive.",
            args.knowledge.as_str()
        );
    }

    let is_memory_ref = matches!(
        crate::memory::MemoryRef::parse(id),
        Ok(crate::memory::MemoryRef::Uid(_)
            | crate::memory::MemoryRef::UidPrefix(_)
            | crate::memory::MemoryRef::Key(_))
    );

    // SL-138 F2: `--transitive` walks the ENTITY relation graph only. A memory ref +
    // `--transitive` is rejected HERE — before the memory inspect early-return below —
    // pointing at the memory graph's own expansion surface (`retrieve --expand`).
    if args.transitive && is_memory_ref {
        anyhow::bail!(
            "{id}: --transitive operates on the entity relation graph; \
             for the memory graph use `doctrine memory retrieve --expand <N>`"
        );
    }

    if is_memory_ref {
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

    // SL-138: the transitive surface is relation-only (no actionability/priority
    // call). Labels resolved + validated (table-derived) before the walk; `DirArg`
    // already mapped DOWN to the engine `TransitiveDir` at dispatch.
    if args.transitive {
        let labels = crate::relation_graph::resolve_transitive_label_names(&args.labels)?;
        let max_depth = parse_max_depth(args.max_depth.as_deref())?;
        let view = crate::relation_graph::transitive_from(
            &scanned,
            &root,
            id,
            args.direction,
            labels.as_deref(),
            max_depth,
        )?;
        let out = match resolved {
            Format::Table => crate::relation_graph::render_transitive_human(&view),
            Format::Json => crate::relation_graph::render_transitive_json(&view)?,
        };
        write!(std::io::stdout(), "{out}")?;
        return Ok(());
    }

    let out = match resolved {
        Format::Table => {
            // View + gate FIRST (the cheap oracle): F6's existence gate (inside
            // inspect_from) errors a ghost id BEFORE the heavier knowledge/priority
            // work. Held so `select_knowledge` below reuses this ONE build (SL-246
            // T4) rather than a second corpus-derived graph.
            let view = crate::relation_graph::inspect_from(&scanned, &root, id)?;
            let relation = crate::relation_graph::render_view(&root, &view, Format::Table)?;
            // SL-246 EX-1: composes relations + the knowledge block + actionability.
            // `commands/inspect.rs` owns no policy about which fields render or what
            // an empty facet means — it only calls select_knowledge / render_block.
            // The section frame is gated on the LEVEL (a property), never on
            // `render_block`'s output being empty (a proxy) — D-c.
            let knowledge_section: String =
                if matches!(args.knowledge, crate::knowledge::KnowledgeLevel::Skip) {
                    String::new()
                } else {
                    let selected = crate::relation_graph::select_knowledge(&view);
                    let block =
                        crate::knowledge::render_block(&root, id, &selected, args.knowledge);
                    ["\nknowledge:\n".to_string(), block].concat()
                };
            // Only reached for a minted id (the render gate passed).
            let block = crate::priority::surface::actionability_block_from(&scanned, &root, id)?;
            let block = crate::priority::render::actionability_block_human(&block);
            [relation, knowledge_section, block].concat()
        }
        Format::Json => {
            // View + gate FIRST, then the knowledge and priority blocks (gate inside
            // inspect_from on the relation projection).
            let view = crate::relation_graph::inspect_from(&scanned, &root, id)?;
            let block = crate::priority::surface::actionability_block_from(&scanned, &root, id)?;
            let mut value = crate::relation_graph::inspect_value(&view);
            if let Some(obj) = value.as_object_mut() {
                // The Option IS the omission mechanism (SL-246 EX-1/VT-1): a `Skip`
                // read inserts NOTHING, never `null` or `[]`. The key lands
                // alphabetically between `kind` and `outbound` (BTreeMap Map).
                // Gated on the LEVEL up front (mirrors the table arm — T4 refactor:
                // both arms now do select → produce in the same order, at the same
                // gate, rather than always selecting and gating the produce step).
                if !matches!(args.knowledge, crate::knowledge::KnowledgeLevel::Skip) {
                    let selected = crate::relation_graph::select_knowledge(&view);
                    if let Some(knowledge) =
                        crate::knowledge::knowledge_value(&root, &selected, args.knowledge)
                    {
                        obj.insert("knowledge".to_string(), knowledge);
                    }
                }
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
