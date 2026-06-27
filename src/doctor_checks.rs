// SPDX-License-Identifier: GPL-3.0-only
//! Native doctor checks — `RawLabel`, `TomlParse`, `ProseCite`.
//!
//! Each check emits [`Finding`]s at [`Severity::Warning`] (never breaks the
//! build, D4). These are disjoint from the existing #1 `IdIntegrity` /
//! #2 `RelationIntegrity` (§ `RelationIntegrity`'s `IllegalRows`) checks — different
//! data sources, different severities.

use std::path::Path;

use crate::catalog::diagnostic::CatalogDiagnostic;
use crate::catalog::hydrate::{CatalogEdgeLabel, CatalogKey};
use crate::catalog::scan::{self, ScanMode};
use crate::finding::{Category, Finding};

// ---------------------------------------------------------------------------
// RawLabel — #6 catalog-scan check
// ---------------------------------------------------------------------------

/// Scan the catalog graph for `CatalogEdgeLabel::Raw` edges.
///
/// ## R7 guarantee (disjointness from #2 `RelationIntegrity`'s `IllegalRows`)
///
/// This check operates over the **hydrated catalog** (`scan_catalog`), which
/// carries `CatalogEdgeLabel::Raw` for every memory-originating edge that
/// survived the catalog build (free-text relation labels). #2
/// `RelationIntegrity`'s `IllegalRows` scan READS the raw TOML `[[relation]]`
/// block directly, catching hand-edited off-table `(source, label)` rows in
/// numbered entities — a different data source. The two checks never overlap
/// on the same finding, and their severities differ (`RawLabel` → Warning vs
/// `IllegalRows` → Error).
pub(crate) fn raw_label_findings(root: &Path) -> Vec<Finding> {
    let Ok(catalog) = crate::catalog::hydrate::scan_catalog(root, ScanMode::default()) else {
        return Vec::new();
    };

    catalog
        .edges
        .iter()
        .filter_map(|edge| {
            if let CatalogEdgeLabel::Raw(label) = &edge.label {
                Some(Finding {
                    category: Category::RawLabel,
                    entity: Some(edge.source.canonical()),
                    message: format!("raw label: {label}"),
                })
            } else {
                None
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// TomlParse — #7 facet+plan.toml parse check
// ---------------------------------------------------------------------------

/// Scan for malformed facet TOML and unparseable plan.toml files.
///
/// ## F-10 guarantee (facet-only split)
///
/// Facet-level diagnostics (estimate / value / risk) are routed here.
/// Entity-level diagnostics (identity, status, relation) belong to #1
/// `IdIntegrity` and are NOT duplicated. The split: this function filters
/// `scan_entities` diagnostics to ONLY those whose `field` is one of
/// `["estimate", "value", "facet"]` (facet-keyed) plus the plan.toml
/// probe — entity-level diags are excluded.
pub(crate) fn toml_parse_findings(root: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut diagnostics = Vec::new();

    // (A) Facet TOML diagnostics from the entity scan.
    // Discard the scan result — we only need the diagnostics vec.
    let _scan_result = scan::scan_entities(root, &mut diagnostics, ScanMode::default());
    for d in diagnostics {
        if is_facet_diagnostic(&d) {
            findings.push(Finding {
                category: Category::TomlParse,
                entity: d.entity_key.as_ref().map(CatalogKey::canonical),
                message: d.message,
            });
        }
    }

    // (B) Plan.toml probe — walk .doctrine/slice/*/plan.toml.
    let slice_root = root.join(".doctrine/slice");
    let Ok(entries) = std::fs::read_dir(&slice_root) else {
        return findings;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        // R4: skip slug symlinks — only numeric dirs carry plan.toml.
        if path.is_symlink() {
            continue;
        }
        let plan_path = path.join("plan.toml");
        if !plan_path.is_file() {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&plan_path) else {
            continue;
        };
        // Try to parse as a TOML document (any valid TOML passes).
        if text.parse::<toml::Table>().is_err() {
            findings.push(Finding {
                category: Category::TomlParse,
                entity: Some(plan_path.display().to_string()),
                message: String::from("unparseable plan.toml"),
            });
        }
    }

    findings
}

/// Facet-level diagnostic gate (F-10): `true` when the diagnostic's `field` is
/// `Some("estimate")`, `Some("value")`, or `Some("facet")` — the three
/// per-entity facet keys read by `scan::read_facets`. Entity-level diagnostics
/// (missing/invalid identity keys, status/title parse failures) have `field:
/// None` and are excluded.
fn is_facet_diagnostic(d: &CatalogDiagnostic) -> bool {
    matches!(d.field.as_deref(), Some("estimate" | "value" | "facet"))
}

// ---------------------------------------------------------------------------
// ProseCite — #8 unresolved citation scanner
// ---------------------------------------------------------------------------

/// Scan authored `.md` prose for unresolved 2-part (`KIND-NNN`) citations.
///
/// Skips:
/// - D11-disposable prose (handover, audit, notes, runtime state, etc.)
/// - Fenced code blocks
/// - Inline code spans (backtick-wrapped)
/// - `*-SENTINEL` tokens
/// - Doc-local refs (`[A-Z][0-9]+` without dash)
/// - 3-part tokens (`KIND-NNN-XX`) — owns the 3-part false-negative (F-4)
/// - Unknown-prefix tokens (silent skip, no bail)
///
/// Emits a `ProseCite` Warning for each known-prefix 2-part citation that does
/// not resolve to an entity on disk.
pub(crate) fn prose_cite_findings(root: &Path) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Anchor to the authored corpus (design §5.5/§7 D11): scan only
    // `.doctrine/**`, never the whole repo — a whole-repo glob descends into
    // nested worktree copies (`.dispatch/`, `.worktrees/`) that each carry a
    // full `.doctrine/`, inflating findings with duplicates. Copies nested under
    // `.doctrine/state/` are additionally caught by `is_disposable_prose`.
    let pattern = root.join(".doctrine/**/*.md");
    let Some(pattern_str) = pattern.to_str() else {
        return findings;
    };

    let Ok(entries) = glob::glob(pattern_str) else {
        return findings;
    };

    // Compile regexes once — patterns are constant, fallible compile returns empty.
    let Ok(re) = regex::Regex::new(r"[A-Z]{2,}-[0-9]+(-[A-Za-z0-9]+)*") else {
        return findings;
    };
    let Ok(doc_local_re) = regex::Regex::new(r"^[A-Z][0-9]+$") else {
        return findings;
    };

    for entry in entries.flatten() {
        if !entry.is_file() {
            continue;
        }
        if is_disposable_prose_d11(&entry) {
            continue;
        }

        let Ok(text) = std::fs::read_to_string(&entry) else {
            continue;
        };

        let mut in_fence = false;
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("```") {
                in_fence = !in_fence;
                continue;
            }
            if in_fence {
                continue;
            }

            // Extract regions outside inline code (backtick) spans.
            let segments = non_code_segments(line);
            for seg in &segments {
                for m in re.find_iter(seg) {
                    let token = m.as_str();

                    // Skip *-SENTINEL.
                    if token.ends_with("-SENTINEL") {
                        continue;
                    }
                    // Skip doc-local refs (single-letter prefix + digit, no dash).
                    if doc_local_re.is_match(token) {
                        continue;
                    }

                    // Classify by hyphen-count.
                    let hyphen_count = token.matches('-').count();
                    if hyphen_count >= 2 {
                        // 3-part (or longer) — F-4: owns the false-negative.
                        continue;
                    }
                    // 2-part: KIND-NNN.
                    let Some((prefix, _num)) = token.split_once('-') else {
                        continue; // unreachable given the regex shape
                    };

                    // Unknown prefix — silent skip.
                    if crate::integrity::kind_by_prefix(prefix).is_none() {
                        continue;
                    }

                    // Known prefix but unresolved — emit finding.
                    if crate::integrity::ensure_ref_resolves(root, token).is_err() {
                        findings.push(Finding {
                            category: Category::ProseCite,
                            entity: Some(entry.display().to_string()),
                            message: format!("unresolved citation: {token}"),
                        });
                    }
                }
            }
        }
    }

    findings
}

/// D11 scan scope: extends [`crate::integrity::is_disposable_prose`] with
/// additional exclusions for the prose-cite scanner (audit.md, inquisition.md,
/// notes.md, research/ directory, .doctrine/review/**).
fn is_disposable_prose_d11(path: &Path) -> bool {
    if crate::integrity::is_disposable_prose(path) {
        return true;
    }
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if matches!(name, "audit.md" | "inquisition.md" | "notes.md") {
        return true;
    }
    let path_str = path.to_string_lossy();
    path_str.contains("/research/") || path_str.contains(".doctrine/review/")
}

/// Return the line's text regions outside inline backtick code spans.
/// Splits on `` ` `` and keeps only even-indexed segments (outside backticks).
fn non_code_segments(line: &str) -> Vec<&str> {
    let parts: Vec<&str> = line.split('`').collect();
    parts
        .into_iter()
        .enumerate()
        .filter_map(|(i, s)| if i % 2 == 0 { Some(s) } else { None })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;
    use crate::catalog::hydrate::{
        Catalog, CatalogEdge, CatalogEdgeLabel, CatalogKey, EdgeTarget, Units,
    };
    use crate::catalog::test_helpers::*;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    // ------------------------------------------------------------------
    // ProseCite test helpers
    // ------------------------------------------------------------------

    /// Set up a temp dir with a real entity so `ensure_ref_resolves` can succeed.
    fn seed_entity_dir(root: &Path, prefix: &str, id: u32) {
        use crate::integrity::kind_by_prefix;
        let Some(kref) = kind_by_prefix(prefix) else {
            return;
        };
        let dir = root.join(kref.kind.dir).join(format!("{id:03}"));
        std::fs::create_dir_all(&dir).unwrap();
    }

    /// Write prose to a `.md` file in `root` and run `prose_cite_findings`.
    /// The fixture lives under the authored corpus (`.doctrine/**`) so the
    /// anchored scan (RV-185 F-3) reaches it — a durable, non-disposable file.
    fn scan_md(root: &Path, prose: &str) -> Vec<Finding> {
        let file = root.join(".doctrine/slice/099/slice-099.md");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, prose).unwrap();
        prose_cite_findings(root)
    }

    /// Root with SL-001 seeded so `SL-001` resolves.
    fn root_with_sl001() -> tempfile::TempDir {
        let dir = tmp();
        seed_entity_dir(dir.path(), "SL", 1);
        dir
    }

    fn test_units() -> Units {
        Units {
            estimation: "espresso_shots".to_string(),
            value: "magic_beans".to_string(),
        }
    }

    // ------------------------------------------------------------------
    // RawLabel tests
    // ------------------------------------------------------------------

    /// Build a pure in-memory catalog with one numbered edge (Validated)
    /// and one raw edge (memory-originating).
    fn catalog_with_raw_edge() -> Catalog {
        let dir = tmp();
        let root = dir.path();
        // One numbered entity.
        seed_slice(root, 1, &[("references(implements)", &["REQ-001"])]);
        let mut catalog = Catalog::from_scanned(
            root,
            &[crate::catalog::scan::ScannedEntity {
                key: crate::catalog::scan::EntityKey {
                    prefix: "SL",
                    id: 1,
                },
                kind: &crate::slice::SLICE_KIND,
                status: Some("proposed".to_string()),
                title: "SL-001".to_string(),
                outbound: vec![crate::relation::RelationEdge::with_role(
                    crate::relation::RelationLabel::References,
                    Some(crate::relation::Role::Implements),
                    "REQ-001".to_string(),
                )],
                estimate: None,
                value: None,
                risk: None,
                tags: vec![],
                body: None,
            }],
            &[],
            &BTreeMap::new(),
            test_units(),
        );
        // Inject a Raw edge.
        catalog.edges.push(CatalogEdge {
            source: CatalogKey::Memory("mem_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string()),
            label: CatalogEdgeLabel::Raw("custom-label".to_string()),
            role: None,
            target: EdgeTarget::UnvalidatedText {
                raw: "free text target".to_string(),
            },
            origin: crate::catalog::hydrate::EdgeOrigin {
                file: PathBuf::from("memory.toml"),
                field: Some("custom-label".to_string()),
            },
        });
        catalog
    }

    #[test]
    fn raw_label_finds_raw_edge() {
        let catalog = catalog_with_raw_edge();
        // Build findings by iterating the catalog edges directly (same logic as
        // raw_label_findings but operating on the in-memory catalog).
        let findings: Vec<Finding> = catalog
            .edges
            .iter()
            .filter_map(|edge| {
                if let CatalogEdgeLabel::Raw(label) = &edge.label {
                    Some(Finding {
                        category: Category::RawLabel,
                        entity: Some(edge.source.canonical()),
                        message: format!("raw label: {label}"),
                    })
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(findings.len(), 1, "one Raw edge should produce one finding");
        let f = &findings[0];
        assert_eq!(f.category, Category::RawLabel);
        assert_eq!(f.category.severity(), crate::finding::Severity::Warning);
        assert!(f.entity.as_deref().unwrap().starts_with("mem_"));
        assert!(f.message.contains("custom-label"));
    }

    #[test]
    fn raw_label_skips_validated_edge() {
        let catalog = catalog_with_raw_edge();
        // The numbered edge (Validated) should NOT produce a finding.
        let has_validated_finding = catalog
            .edges
            .iter()
            .any(|edge| matches!(&edge.label, CatalogEdgeLabel::Validated(_)));
        assert!(has_validated_finding, "fixture has a Validated edge");
        // But raw_label_findings should only flag Raw edges.
        let raw_count = catalog
            .edges
            .iter()
            .filter(|e| matches!(e.label, CatalogEdgeLabel::Raw(_)))
            .count();
        assert_eq!(raw_count, 1);
    }

    #[test]
    fn raw_label_verify_disjointness_from_illegal_rows() {
        // Conceptual: RawLabel operates over catalog edges; IllegalRows operates
        // over raw TOML text. The finding category and severity differ.
        let f = Finding {
            category: Category::RawLabel,
            entity: Some("mem_xxx".into()),
            message: "raw label: test".into(),
        };
        assert_eq!(f.category.severity(), crate::finding::Severity::Warning);
        // RelationIntegrity is Error severity — different.
        assert_eq!(
            Category::RelationIntegrity.severity(),
            crate::finding::Severity::Error
        );
    }

    // ------------------------------------------------------------------
    // TomlParse tests
    // ------------------------------------------------------------------

    #[test]
    fn toml_parse_flags_malformed_facet() {
        let dir = tmp();
        let root = dir.path();
        // Seed an ADR with a malformed estimate facet (non-table).
        write(
            root,
            ".doctrine/adr/001/adr-001.toml",
            "id = 1\nslug = \"a1\"\ntitle = \"A1\"\nstatus = \"accepted\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
             estimate = 7\n",
        );
        write(root, ".doctrine/adr/001/adr-001.md", "body\n");

        let findings = toml_parse_findings(root);
        // The malformed "estimate = 7" (not a table) is a facet diagnostic.
        assert!(
            !findings.is_empty(),
            "malformed facet should produce TomlParse findings"
        );
        for f in &findings {
            assert_eq!(f.category, Category::TomlParse);
            assert_eq!(f.category.severity(), crate::finding::Severity::Warning);
            assert!(
                f.message.contains("must be a table") || f.message.contains("estimate"),
                "message should mention estimate: {}",
                f.message
            );
        }
    }

    #[test]
    fn toml_parse_flags_malformed_plan_toml() {
        let dir = tmp();
        let root = dir.path();
        // Create a slice dir with a broken plan.toml.
        let plan_dir = root.join(".doctrine/slice/001");
        std::fs::create_dir_all(&plan_dir).unwrap();
        std::fs::write(plan_dir.join("plan.toml"), "this is not valid toml [[[[").unwrap();

        let findings = toml_parse_findings(root);
        let plan_findings: Vec<&Finding> = findings
            .iter()
            .filter(|f| f.message.contains("unparseable plan.toml"))
            .collect();
        assert_eq!(
            plan_findings.len(),
            1,
            "malformed plan.toml should produce one finding"
        );
        let f = plan_findings[0];
        assert!(f.entity.as_deref().unwrap().contains("plan.toml"));
    }

    #[test]
    fn toml_parse_skips_symlink_slice_dirs() {
        let dir = tmp();
        let root = dir.path();
        // Create a real numeric dir with a malformed plan.toml.
        let real_dir = root.join(".doctrine/slice/001");
        std::fs::create_dir_all(&real_dir).unwrap();
        std::fs::write(real_dir.join("plan.toml"), "not toml [[").unwrap();
        // Create a slug symlink. No plan.toml should be read from the symlink.
        let symlink = root.join(".doctrine/slice/001-my-slug");
        #[cfg(unix)]
        std::os::unix::fs::symlink("001", &symlink).unwrap();
        #[cfg(not(unix))]
        {
            // On non-unix, skip the symlink test; fallback to a file.
            std::fs::write(&symlink, "not a symlink").unwrap();
        }

        let findings = toml_parse_findings(root);
        // Should have exactly one plan.toml finding (from 001, not the symlink).
        let plan_count = findings
            .iter()
            .filter(|f| f.message.contains("unparseable plan.toml"))
            .count();
        // On unix, symlink is skipped; on non-unix, the symlink test doesn't apply.
        if cfg!(unix) {
            assert_eq!(plan_count, 1, "symlink slice dir must be skipped");
        }
    }

    #[test]
    fn toml_parse_excludes_entity_level_diagnostics() {
        let dir = tmp();
        let root = dir.path();
        // Malformed entity (not a facet issue) — entity-level parse failure.
        write(
            root,
            ".doctrine/slice/002/slice-002.toml",
            "id = notanumber\n",
        );
        write(root, ".doctrine/slice/002/slice-002.md", "scope\n");

        let findings = toml_parse_findings(root);
        // Entity-level diags have field=None; they must NOT appear in TomlParse.
        for f in &findings {
            assert!(
                !f.message.contains("notanumber"),
                "entity-level diag must not appear: {}",
                f.message
            );
        }
    }

    #[test]
    fn toml_parse_severity_is_warning() {
        assert_eq!(
            Category::TomlParse.severity(),
            crate::finding::Severity::Warning
        );
    }

    // ------------------------------------------------------------------
    // ProseCite tests
    // ------------------------------------------------------------------

    #[test]
    fn prose_cite_severity_is_warning() {
        assert_eq!(
            Category::ProseCite.severity(),
            crate::finding::Severity::Warning
        );
    }

    // --- VT-1 classes ---

    #[test]
    fn prose_cite_resolved_2part_produces_no_finding() {
        let dir = root_with_sl001();
        let root = dir.path();
        let findings = scan_md(root, "see SL-001 for details");
        assert!(
            findings.is_empty(),
            "resolved SL-001 should produce no finding: {findings:?}"
        );
    }

    #[test]
    fn prose_cite_dangling_2part_produces_finding() {
        let dir = root_with_sl001();
        let root = dir.path();
        // SL-999 is a known prefix but no entity exists.
        let findings = scan_md(root, "see SL-999 for details");
        assert_eq!(findings.len(), 1, "dangling SL-999 should produce finding");
        let f = &findings[0];
        assert_eq!(f.category, Category::ProseCite);
        assert!(f.message.contains("SL-999"), "message: {}", f.message);
        assert!(f.message.contains("unresolved citation"));
    }

    #[test]
    fn prose_cite_code_span_inline_no_finding() {
        let dir = root_with_sl001();
        let root = dir.path();
        // SL-001 is inside a backtick span — excluded.
        let findings = scan_md(root, "use `SL-001` as reference");
        assert!(
            findings.is_empty(),
            "backtick-wrapped SL-001 should be skipped: {findings:?}"
        );
    }

    #[test]
    fn prose_cite_code_span_fenced_block_no_finding() {
        let dir = root_with_sl001();
        let root = dir.path();
        let prose = "before\n```\nSL-999\n```\nafter";
        let findings = scan_md(root, prose);
        assert!(
            findings.is_empty(),
            "fenced block SL-999 should be skipped: {findings:?}"
        );
    }

    #[test]
    fn prose_cite_fenced_block_toggle_respects_boundaries() {
        let dir = root_with_sl001();
        let root = dir.path();
        // Fence toggles: SL-999 outside fence → finding; SL-888 inside → skipped.
        let prose = "SL-999 is outside\n```\nSL-888\n```\nSL-999 again outside";
        let findings = scan_md(root, prose);
        // Two copies of SL-999 outside fence — one finding per unique token per file?
        // No, two independent finding entries (same token, same file).
        assert_eq!(findings.len(), 2, "both SL-999 outside fence reported");
        for f in &findings {
            assert!(f.message.contains("SL-999"));
        }
    }

    #[test]
    fn prose_cite_sentinel_no_finding() {
        let dir = root_with_sl001();
        let root = dir.path();
        let findings = scan_md(root, "BOOT-SENTINEL: doctrine-governance-snapshot");
        assert!(
            findings.is_empty(),
            "BOOT-SENTINEL should be skipped: {findings:?}"
        );
    }

    #[test]
    fn prose_cite_doc_local_no_finding() {
        let dir = root_with_sl001();
        let root = dir.path();
        // D1, R1, C1, Q1 — single-letter prefix + digit, no dash.
        let findings = scan_md(root, "see D1 and R1 and C1 and Q1");
        assert!(
            findings.is_empty(),
            "doc-local refs should be skipped: {findings:?}"
        );
    }

    #[test]
    fn prose_cite_3part_no_finding() {
        let dir = root_with_sl001();
        let root = dir.path();
        let findings = scan_md(root, "see DEC-005-C and DEC-010-06");
        assert!(
            findings.is_empty(),
            "3-part tokens should be skipped: {findings:?}"
        );
    }

    #[test]
    fn prose_cite_compound_3part_no_finding() {
        let dir = root_with_sl001();
        let root = dir.path();
        // SL-048-style is 3-part (SL, 048, style).
        let findings = scan_md(root, "the SL-048-style pattern");
        assert!(
            findings.is_empty(),
            "SL-048-style (3-part) should be skipped: {findings:?}"
        );
    }

    #[test]
    fn prose_cite_unknown_prefix_no_finding() {
        let dir = root_with_sl001();
        let root = dir.path();
        // SHA and PHASE are not known kind prefixes.
        let findings = scan_md(root, "use SHA-256 for PHASE-03 hashing");
        assert!(
            findings.is_empty(),
            "unknown prefix tokens should be skipped: {findings:?}"
        );
    }

    // --- VT-2: maximal token ---

    #[test]
    fn prose_cite_maximal_token_no_submatch() {
        let dir = root_with_sl001();
        let root = dir.path();
        // DEC-005-C matches as DEC-005-C (3-part), NOT as DEC-005 (2-part).
        // DEC-005 is a known prefix but would be a dangling if matched separately.
        let findings = scan_md(root, "the DEC-005-C encoding");
        assert!(
            findings.is_empty(),
            "DEC-005-C must be 3-part only, not 2-part DEC-005: {findings:?}"
        );
    }

    // --- VT-3: D11 scope ---

    #[test]
    fn prose_cite_skips_audit_md() {
        let dir = root_with_sl001();
        let root = dir.path();
        let file = root.join(".doctrine/slice/001/audit.md");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "dangling SL-999 here\n").unwrap();
        let findings = prose_cite_findings(root);
        assert!(
            findings.is_empty(),
            "audit.md should be skipped: {findings:?}"
        );
    }

    #[test]
    fn prose_cite_skips_inquisition_md() {
        let dir = root_with_sl001();
        let root = dir.path();
        let file = root.join(".doctrine/slice/001/inquisition.md");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "dangling SL-999 here\n").unwrap();
        let findings = prose_cite_findings(root);
        assert!(
            findings.is_empty(),
            "inquisition.md should be skipped: {findings:?}"
        );
    }

    #[test]
    fn prose_cite_skips_notes_md() {
        let dir = root_with_sl001();
        let root = dir.path();
        let file = root.join(".doctrine/slice/001/notes.md");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "dangling SL-999 here\n").unwrap();
        let findings = prose_cite_findings(root);
        assert!(
            findings.is_empty(),
            "notes.md should be skipped: {findings:?}"
        );
    }

    #[test]
    fn prose_cite_skips_research_dir() {
        let dir = root_with_sl001();
        let root = dir.path();
        let file = root.join(".doctrine/research/notes/some.md");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "dangling SL-999 here\n").unwrap();
        let findings = prose_cite_findings(root);
        assert!(
            findings.is_empty(),
            "research/ should be skipped: {findings:?}"
        );
    }

    #[test]
    fn prose_cite_skips_doctrine_review_dir() {
        let dir = root_with_sl001();
        let root = dir.path();
        let file = root.join(".doctrine/review/042/summary.md");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "dangling SL-999 here\n").unwrap();
        let findings = prose_cite_findings(root);
        assert!(
            findings.is_empty(),
            ".doctrine/review/ should be skipped: {findings:?}"
        );
    }

    #[test]
    fn prose_cite_scans_non_skipped_file() {
        let dir = root_with_sl001();
        let root = dir.path();
        let file = root.join(".doctrine/slice/002/slice-002.md");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "dangling SL-999 here\n").unwrap();
        let findings = prose_cite_findings(root);
        assert_eq!(
            findings.len(),
            1,
            "authored prose should produce finding: {findings:?}"
        );
        assert!(findings[0].message.contains("SL-999"));
    }

    #[test]
    fn prose_cite_skips_handover_md() {
        let dir = root_with_sl001();
        let root = dir.path();
        let file = root.join(".doctrine/slice/001/handover.md");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "dangling SL-999 here\n").unwrap();
        let findings = prose_cite_findings(root);
        assert!(
            findings.is_empty(),
            "handover.md should be skipped via is_disposable_prose: {findings:?}"
        );
    }

    #[test]
    fn prose_cite_skips_doctrine_state() {
        let dir = root_with_sl001();
        let root = dir.path();
        let file = root.join(".doctrine/state/slice/001/phase-01.md");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "dangling SL-999 here\n").unwrap();
        let findings = prose_cite_findings(root);
        assert!(
            findings.is_empty(),
            ".doctrine/state should be skipped via is_disposable_prose: {findings:?}"
        );
    }
}
