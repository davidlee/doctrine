// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine graph` — thin command composing the catalog-graph engine
//! (projection filters + neighbourhood + dot render, SL-226 PHASES 01–03).
//! ADR-001: the command layer holds NO projection logic — it only wires args,
//! classifies the focus, calls the engine, and prints.

use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::catalog::graph::CatalogGraph;
use crate::catalog::hydrate::CatalogKey;
use crate::catalog::scan::{EntityKey, ScanMode};

/// The CLI format variant for `--format` (dot | json). A dedicated enum —
/// NOT shared with `listing::Format` (which carries Table/Json).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GraphFormat {
    Dot,
    Json,
}

impl std::fmt::Display for GraphFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dot => f.write_str("dot"),
            Self::Json => f.write_str("json"),
        }
    }
}

impl std::str::FromStr for GraphFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dot" => Ok(Self::Dot),
            "json" => Ok(Self::Json),
            other => Err(format!(
                "unknown format '{other}' (expected 'dot' or 'json')"
            )),
        }
    }
}

// ── build_graph_output — the testable core ──────────────────────────────────

/// Build the graph output string for the given root and parsed args.
/// All logic lives here so tests assert on the output without capturing stdout.
pub(crate) fn build_graph_output(
    root: &Path,
    focus: Option<&str>,
    depth: u32,
    kinds: &[String],
    label: Option<&str>,
    include_memory: bool,
    format: &GraphFormat,
) -> anyhow::Result<String> {
    // 1. Scan + full graph
    let g_full = CatalogGraph::from_catalog(&crate::catalog::hydrate::scan_catalog(
        root,
        ScanMode::default(),
    )?);

    // 2. Validate --kind (D12): uppercase each K; legal iff in ALL_KINDS or "MEM"
    let kind_set: BTreeSet<String> = {
        let mut set = BTreeSet::new();
        for k in kinds {
            let upper = k.to_uppercase();
            if !crate::kinds::ALL_KINDS.contains(&upper.as_str()) && upper != "MEM" {
                let legal: Vec<&str> = crate::kinds::ALL_KINDS
                    .iter()
                    .copied()
                    .chain(std::iter::once("MEM"))
                    .collect();
                anyhow::bail!(
                    "unknown kind prefix '{k}'; legal prefixes: {}",
                    legal.join(", ")
                );
            }
            set.insert(upper);
        }
        set
    };
    let kind_given = !kind_set.is_empty();

    // 3. Resolve FOCUS (store the string for error messages)
    let focus_owned = focus.map(std::borrow::ToOwned::to_owned);
    let focus_key: Option<CatalogKey> = match focus_owned.as_deref() {
        Some(f) => Some(resolve_focus(root, f)?),
        None => None,
    };

    // 4. D6 exclusion check (only if FOCUS given)
    if let (Some(fk), Some(f)) = (&focus_key, focus) {
        if !g_full.contains(fk) {
            anyhow::bail!("focus '{f}' not found");
        }
        let mut nf = g_full.clone();
        if kind_given {
            nf = nf.filter_kinds(&kind_set);
        }
        if !include_memory {
            nf = nf.exclude_memory();
        }
        if !nf.contains(fk) {
            if matches!(fk, CatalogKey::Memory(_)) && !include_memory {
                anyhow::bail!("MEM focus requires --include-memory");
            }
            let joined: Vec<String> = kind_set.iter().cloned().collect();
            anyhow::bail!("{} excluded by --kind {}", f, joined.join(", "));
        }
    }

    // 5. Projection pipeline
    let mut g = g_full;
    if kind_given {
        g = g.filter_kinds(&kind_set);
    }
    if !include_memory {
        g = g.exclude_memory();
    }
    if let Some(l) = &label {
        g = g.filter_label(l);
    }
    if let Some(fk) = &focus_key {
        g = g.neighbourhood(fk, depth);
    } else if label.is_some() {
        g = g.drop_isolated();
    }

    // 6. Emit
    match format {
        GraphFormat::Json => Ok(serde_json::to_string(&g)?),
        GraphFormat::Dot => {
            let dot = crate::catalog::dot::render(&g, focus_key.as_ref());
            Ok(dot)
        }
    }
}

// ── focus resolution ────────────────────────────────────────────────────────

fn resolve_focus(root: &Path, f: &str) -> anyhow::Result<CatalogKey> {
    if f.starts_with("mem.") || f.starts_with("mem_") {
        let mref = crate::memory::MemoryRef::parse(f)?;
        let all = crate::memory::collect_all(root)?;
        let mem = crate::memory::resolve_memory_from_all(&all, &mref)?;
        return Ok(CatalogKey::Memory(mem.uid.clone()));
    }

    let (kref, id) = crate::kinds::parse_canonical_ref(f)
        .map_err(|e| anyhow::anyhow!("focus '{f}' not found: {e}"))?;
    Ok(CatalogKey::Numbered(EntityKey {
        prefix: kref.kind.prefix,
        id,
    }))
}

// ── run_graph — thin shell ──────────────────────────────────────────────────

#[expect(
    clippy::needless_pass_by_value,
    reason = "clap dispatch signature — clap consumes the values"
)]
pub(crate) fn run_graph(
    path: Option<PathBuf>,
    focus: Option<String>,
    depth: u32,
    kinds: Vec<String>,
    label: Option<String>,
    include_memory: bool,
    format: GraphFormat,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let output = build_graph_output(
        &root,
        focus.as_deref(),
        depth,
        kinds.as_slice(),
        label.as_deref(),
        include_memory,
        &format,
    )?;
    writeln!(std::io::stdout(), "{output}")?;
    Ok(())
}

// ── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;
    use crate::catalog::test_helpers::{seed_adr, seed_slice, seed_spec, tmp};
    use crate::test_support::SCHEMA_MEMORY;
    use std::fs;

    /// Seed a memory entity in the items dir.
    fn seed_memory_item(root: &Path, uid: &str, title: &str, relations: &[(&str, &str)]) {
        let mem_uid = format!("mem_{uid}");
        let dir = root.join(".doctrine/memory/items").join(&mem_uid);
        fs::create_dir_all(&dir).unwrap();
        let rels: Vec<String> = relations
            .iter()
            .map(|(l, t)| format!("[[relation]]\nlabel = \"{l}\"\ntarget = \"{t}\"\n"))
            .collect();
        fs::write(
            dir.join("memory.toml"),
            format!(
                "schema = \"{SCHEMA_MEMORY}\"\n\
                 schema_version = 1\n\
                 memory_uid = \"{mem_uid}\"\n\
                 title = \"{title}\"\n\
                 status = \"active\"\n\
                 memory_type = \"pattern\"\n\
                 created = \"2026-01-01\"\n\
                 updated = \"2026-01-01\"\n\
                 [scope]\n\
                 paths = []\n\
                 commands = []\n\
                 tags = []\n\
                 workspace = \"default\"\n\
                 repo = \"default\"\n\
                 [git]\n\
                 anchor_kind = \"none\"\n\
                 [review]\n\
                 verification_state = \"unverified\"\n\
                 review_by = \"2027-01-01\"\n\
                 [trust]\n\
                 trust_level = \"medium\"\n\
                 {}",
                rels.concat()
            ),
        )
        .unwrap();
    }

    /// Bare (no focus) → output starts `digraph G {` (whole-corpus DOT).
    #[test]
    fn bare_no_focus_output_starts_digraph() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, &[]);
        let output =
            build_graph_output(root, None, 1, &[], None, false, &GraphFormat::Dot).unwrap();
        assert!(
            output.starts_with("digraph G {"),
            "expected DOT output; got: {output}"
        );
    }

    /// FOCUS + depth bounds the output (distant node absent at depth 1, present at higher depth).
    #[test]
    fn focus_depth_bounds_output() {
        let dir = tmp();
        let root = dir.path();
        // Chain: SL-001 → SPEC-002 → PRD-003
        //   SL-001 references(implements) SPEC-002
        //   SPEC-002 descends_from PRD-003
        seed_slice(root, 1, &[("references(implements)", &["SPEC-002"])]);
        seed_spec(
            root,
            crate::spec::SpecSubtype::Tech,
            2,
            &[],
            &[],
            &[("descends_from", "PRD-003")],
        );
        seed_spec(root, crate::spec::SpecSubtype::Product, 3, &[], &[], &[]);

        // depth 1 from SL-001: SL-001 + SPEC-002, NOT PRD-003 (distance 2)
        let d1 = build_graph_output(root, Some("SL-001"), 1, &[], None, false, &GraphFormat::Dot)
            .unwrap();
        assert!(d1.contains("SL-001"));
        assert!(d1.contains("SPEC-002"));
        assert!(
            !d1.contains("PRD-003"),
            "PRD-003 at distance 2; depth=1 excludes it"
        );

        // depth 2 from SL-001: reaches PRD-003
        let d2 = build_graph_output(root, Some("SL-001"), 2, &[], None, false, &GraphFormat::Dot)
            .unwrap();
        assert!(d2.contains("SL-001"));
        assert!(d2.contains("SPEC-002"));
        assert!(d2.contains("PRD-003"));
    }

    /// --format json output parses as JSON and equals serde_json::to_string of the same projected graph (contract parity).
    #[test]
    fn format_json_parity_with_serde() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, &[("supersedes", &["ADR-002"])]);
        seed_adr(root, 2, &[]);

        let output = build_graph_output(
            root,
            Some("SL-001"),
            1,
            &[],
            None,
            false,
            &GraphFormat::Json,
        )
        .unwrap();

        // Must parse as JSON
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.is_object());

        // Contract parity: build the same projection manually and compare
        let catalog = crate::catalog::hydrate::scan_catalog(root, ScanMode::default()).unwrap();
        let g_full = CatalogGraph::from_catalog(&catalog);
        let sl = CatalogKey::Numbered(EntityKey {
            prefix: "SL",
            id: 1,
        });
        let g = g_full.neighbourhood(&sl, 1);
        let expected = serde_json::to_string(&g).unwrap();
        assert_eq!(output, expected);
    }

    /// Empty projection (filters exclude everything) → still Ok, valid empty digraph.
    #[test]
    fn empty_projection_yields_valid_empty_digraph() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, &[]);

        // --kind ADR filters out the only node (SL), leaving empty projection
        let output = build_graph_output(
            root,
            Some("SL-001"),
            1,
            &["ADR".to_string()],
            None,
            false,
            &GraphFormat::Dot,
        );

        // D6: SL-001 is excluded by --kind ADR (which only keeps ADR nodes)
        assert!(output.is_err());
        // Now test without focus so D6 doesn't apply
        let output = build_graph_output(
            root,
            None,
            1,
            &["ADR".to_string()],
            None,
            false,
            &GraphFormat::Dot,
        )
        .unwrap();
        // DOT contains a valid empty digraph
        assert!(output.contains("digraph G {"));
        assert!(output.contains('}'));
        // JSON: parseable
        let json_out = build_graph_output(
            root,
            None,
            1,
            &["ADR".to_string()],
            None,
            false,
            &GraphFormat::Json,
        )
        .unwrap();
        let _: serde_json::Value = serde_json::from_str(&json_out).unwrap();
    }

    /// A mem. focus resolves (no error).  Test fn name contains `include_memory`.
    #[test]
    fn mem_focus_with_include_memory_resolves() {
        let dir = tmp();
        let root = dir.path();
        let uid = "aaaaaaaa00000000bbbbbbbb11111111";
        seed_memory_item(root, uid, "TestMem", &[]);

        let output = build_graph_output(
            root,
            Some(&format!("mem_{uid}")),
            1,
            &[],
            None,
            true,
            &GraphFormat::Dot,
        )
        .unwrap();
        // The node id in DOT is the uid from the catalog, which is "mem_<hex>"
        assert!(output.contains(&format!("mem_{uid}")));
    }

    /// MEM focus WITHOUT `--include_memory` → Err whose message names `--include_memory`.
    #[test]
    fn mem_focus_without_include_memory_errors() {
        let dir = tmp();
        let root = dir.path();
        let uid = "aaaaaaaa00000000bbbbbbbb11111111";
        seed_memory_item(root, uid, "TestMem", &[]);

        let err = build_graph_output(
            root,
            Some(&format!("mem_{uid}")),
            1,
            &[],
            None,
            false,
            &GraphFormat::Dot,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("--include-memory"),
            "error should mention --include-memory: {err}"
        );
    }

    /// A focus excluded by --kind → Err naming the filter.
    #[test]
    fn focus_excluded_by_kind_errors() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, &[]);

        let err = build_graph_output(
            root,
            Some("SL-001"),
            1,
            &["ADR".to_string()],
            None,
            false,
            &GraphFormat::Dot,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("excluded by --kind"),
            "error should mention excluded by --kind: {err}"
        );
        assert!(
            err.to_string().contains("SL-001"),
            "error should name the excluded focus: {err}"
        );
    }

    /// Unknown focus → Err.
    #[test]
    fn unknown_focus_errors() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, &[]);

        let err = build_graph_output(root, Some("SL-999"), 1, &[], None, false, &GraphFormat::Dot)
            .unwrap_err();
        assert!(
            err.to_string().contains("not found"),
            "unknown focus should error: {err}"
        );
    }

    /// Unknown `--kind` (validated against ALL_KINDS) → Err.
    #[test]
    fn unknown_kind_validated_against_all_kinds_errors() {
        let dir = tmp();
        let root = dir.path();
        seed_slice(root, 1, &[]);

        let err = build_graph_output(
            root,
            None,
            1,
            &["ZZZ".to_string()],
            None,
            false,
            &GraphFormat::Dot,
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("unknown kind prefix"),
            "unknown kind should error: {err}"
        );
        assert!(
            err.to_string().contains("legal prefixes"),
            "error should list legal prefixes: {err}"
        );
    }

    /// --format json on an unknown format errors.
    #[test]
    fn unknown_format_errors() {
        let result = "bad".parse::<GraphFormat>();
        assert!(result.is_err());
    }
}
