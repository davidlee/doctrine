// SPDX-License-Identifier: GPL-3.0-only
//! Unified finding type for the corpus health doctor.
//!
//! Pure leaf per ADR-001 — imports neither `clap` nor `entity`; check modules
//! import *down* into this, never the reverse.
//!
//! This module follows ADR-001: it imports neither `clap` nor `entity`.
//! ```bash
//! grep -c 'use clap' src/finding.rs  # must be 0
//! grep -c 'entity::' src/finding.rs  # must be 0
//! ```

#![allow(dead_code, reason = "PHASE-01 leaf — consumers arrive in later phases")]

use serde::Serialize;
use std::fmt;
use std::fmt::Write;

// ---- named constants (STD-001) ----

const CATEGORY_NAME_ID_INTEGRITY: &str = "Id Integrity";
const CATEGORY_NAME_RELATION_INTEGRITY: &str = "Relation Integrity";
const CATEGORY_NAME_SPEC_FK: &str = "Spec Foreign Key";
const CATEGORY_NAME_MEMORY_HEALTH: &str = "Memory Health";
const CATEGORY_NAME_LIFECYCLE: &str = "Lifecycle";
const CATEGORY_NAME_RAW_LABEL: &str = "Raw Label";
const CATEGORY_NAME_TOML_PARSE: &str = "TOML Parse";
const CATEGORY_NAME_PROSE_CITE: &str = "Prose Citation";
const CATEGORY_NAME_AGENT_CONFORMANCE: &str = "Agent Conformance";
const CATEGORY_NAME_SPAWN_SEAM_SYMMETRY: &str = "Spawn Seam Symmetry";

const SEVERITY_ERROR: &str = "error";
const SEVERITY_WARNING: &str = "warning";

const CORPUS_CLEAN: &str = "doctor: corpus clean";
const FINDING_COUNT_FMT: &str = "{} finding(s)";

// ---- types ----

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Severity {
    Error,
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Error => SEVERITY_ERROR,
            Self::Warning => SEVERITY_WARNING,
        };
        f.write_str(s)
    }
}

impl Serialize for Severity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            Self::Error => SEVERITY_ERROR,
            Self::Warning => SEVERITY_WARNING,
        };
        serializer.serialize_str(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Category {
    IdIntegrity,
    RelationIntegrity,
    SpecFk,
    MemoryHealth,
    Lifecycle,
    RawLabel,
    TomlParse,
    ProseCite,
    AgentConformance,
    SpawnSeamSymmetry,
}

impl Category {
    /// Single severity source (F5) — IdIntegrity/RelationIntegrity/SpecFk/MemoryHealth,
    /// `AgentConformance` (SL-198 RSK-225: worker tool-surface is a jail wall), and
    /// `SpawnSeamSymmetry` (SL-206 design §5.6 I1: unjail nomination/gate drift is a
    /// security boundary, not a style nit) are errors; Lifecycle/RawLabel/TomlParse/
    /// `ProseCite` are warnings.
    #[must_use]
    pub(crate) const fn severity(self) -> Severity {
        match self {
            Self::IdIntegrity
            | Self::RelationIntegrity
            | Self::SpecFk
            | Self::MemoryHealth
            | Self::AgentConformance
            | Self::SpawnSeamSymmetry => Severity::Error,
            Self::Lifecycle | Self::RawLabel | Self::TomlParse | Self::ProseCite => {
                Severity::Warning
            }
        }
    }

    #[must_use]
    const fn ordinal(self) -> u8 {
        match self {
            Self::IdIntegrity => 0,
            Self::RelationIntegrity => 1,
            Self::SpecFk => 2,
            Self::MemoryHealth => 3,
            Self::Lifecycle => 4,
            Self::RawLabel => 5,
            Self::TomlParse => 6,
            Self::ProseCite => 7,
            Self::AgentConformance => 8,
            Self::SpawnSeamSymmetry => 9,
        }
    }

    #[must_use]
    const fn display_name(self) -> &'static str {
        match self {
            Self::IdIntegrity => CATEGORY_NAME_ID_INTEGRITY,
            Self::RelationIntegrity => CATEGORY_NAME_RELATION_INTEGRITY,
            Self::SpecFk => CATEGORY_NAME_SPEC_FK,
            Self::MemoryHealth => CATEGORY_NAME_MEMORY_HEALTH,
            Self::Lifecycle => CATEGORY_NAME_LIFECYCLE,
            Self::RawLabel => CATEGORY_NAME_RAW_LABEL,
            Self::TomlParse => CATEGORY_NAME_TOML_PARSE,
            Self::ProseCite => CATEGORY_NAME_PROSE_CITE,
            Self::AgentConformance => CATEGORY_NAME_AGENT_CONFORMANCE,
            Self::SpawnSeamSymmetry => CATEGORY_NAME_SPAWN_SEAM_SYMMETRY,
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

impl Serialize for Category {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.display_name())
    }
}

/// All categories in ordinal order.
const CATEGORIES_BY_ORDINAL: [Category; 10] = [
    Category::IdIntegrity,
    Category::RelationIntegrity,
    Category::SpecFk,
    Category::MemoryHealth,
    Category::Lifecycle,
    Category::RawLabel,
    Category::TomlParse,
    Category::ProseCite,
    Category::AgentConformance,
    Category::SpawnSeamSymmetry,
];

#[derive(Debug, Clone)]
pub(crate) struct Finding {
    pub category: Category,
    pub entity: Option<String>,
    pub message: String,
}

impl Serialize for Finding {
    /// Row shape per design §5.4: `{category, severity, entity, message}`.
    /// `severity` is derived from `category.severity()` (the single source, F5) —
    /// it is not a struct field, so it cannot drift (RV-185 F-6).
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut row = serializer.serialize_struct("Finding", 4)?;
        row.serialize_field("category", &self.category)?;
        row.serialize_field("severity", &self.category.severity())?;
        row.serialize_field("entity", &self.entity)?;
        row.serialize_field("message", &self.message)?;
        row.end()
    }
}

impl Finding {
    /// Wrap each line in `lines` as a separate [`Finding`] with `entity: None`.
    pub(crate) fn from_lines(category: Category, lines: Vec<String>) -> Vec<Finding> {
        lines
            .into_iter()
            .map(|line| Finding {
                category,
                entity: None,
                message: line,
            })
            .collect()
    }
}

// ---- render ----

/// Group findings by category (ordinal order), render each non-empty group
/// with a bracketed header, then a summary line.
///
/// When `verbose` is false, `RawLabel` findings are aggregated into a single
/// informational count line rather than rendered per-item.
pub(crate) fn render_findings(findings: &[Finding], verbose: bool) -> String {
    let mut by_category: [Vec<&Finding>; 10] = [
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ];

    for f in findings {
        let idx = usize::from(f.category.ordinal());
        if let Some(bucket) = by_category.get_mut(idx) {
            bucket.push(f);
        }
    }

    let mut out = String::new();
    let mut total: usize = 0;
    let mut raw_label_count: usize = 0;

    for cat in &CATEGORIES_BY_ORDINAL {
        let idx = usize::from(cat.ordinal());
        let Some(group) = by_category.get(idx) else {
            continue;
        };
        if group.is_empty() {
            continue;
        }
        // IMP-252: in non-verbose mode, aggregate RawLabel into a count line.
        if !verbose && *cat == Category::RawLabel {
            raw_label_count = group.len();
            continue;
        }
        let _header = writeln!(out, "[{}]", cat.display_name());
        for f in group {
            let _line = writeln!(out, "  {}: {}", f.category.severity(), f.message);
            total = total.saturating_add(1);
        }
    }

    // RawLabel count line (non-verbose only).
    if raw_label_count > 0 {
        if !out.is_empty() {
            out.push('\n');
        }
        let _line = writeln!(
            out,
            "Raw Label: {raw_label_count} memory edge(s) use raw labels (expected)"
        );
    }

    if total == 0 && raw_label_count == 0 {
        out.push_str(CORPUS_CLEAN);
    } else {
        let _summary = write!(out, "{total} finding(s)");
    }

    out
}

// ---- tests ----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_mapping() {
        assert_eq!(Category::IdIntegrity.severity(), Severity::Error);
        assert_eq!(Category::RelationIntegrity.severity(), Severity::Error);
        assert_eq!(Category::SpecFk.severity(), Severity::Error);
        assert_eq!(Category::MemoryHealth.severity(), Severity::Error);
        assert_eq!(Category::Lifecycle.severity(), Severity::Warning);
        assert_eq!(Category::RawLabel.severity(), Severity::Warning);
        assert_eq!(Category::TomlParse.severity(), Severity::Warning);
        assert_eq!(Category::ProseCite.severity(), Severity::Warning);
        assert_eq!(Category::AgentConformance.severity(), Severity::Error);
        assert_eq!(Category::SpawnSeamSymmetry.severity(), Severity::Error);
    }

    #[test]
    fn test_from_lines() {
        let findings = Finding::from_lines(Category::SpecFk, vec!["a".into(), "b".into()]);
        assert_eq!(findings.len(), 2);
        assert!(findings[0].entity.is_none());
        assert!(findings[1].entity.is_none());
        assert_eq!(findings[0].message, "a");
        assert_eq!(findings[1].message, "b");
    }

    #[test]
    fn test_render_empty() {
        let out = render_findings(&[], true);
        assert!(out.contains(CORPUS_CLEAN));
        assert!(!out.contains('['));
    }

    #[test]
    fn test_render_grouped() {
        let f1 = Finding {
            category: Category::IdIntegrity,
            entity: None,
            message: "bad id".into(),
        };
        let f2 = Finding {
            category: Category::Lifecycle,
            entity: None,
            message: "stale draft".into(),
        };
        let out = render_findings(&[f1, f2], true);
        assert!(out.contains(CATEGORY_NAME_ID_INTEGRITY));
        assert!(out.contains(CATEGORY_NAME_LIFECYCLE));
        assert!(out.contains("2 finding(s)"));
    }

    #[test]
    fn test_render_all_categories() {
        let findings: Vec<Finding> = CATEGORIES_BY_ORDINAL
            .iter()
            .map(|&cat| Finding {
                category: cat,
                entity: None,
                message: format!("test {cat}"),
            })
            .collect();
        let out = render_findings(&findings, true);
        for cat in &CATEGORIES_BY_ORDINAL {
            assert!(out.contains(cat.display_name()), "missing category: {cat}");
        }
        assert!(out.contains(&format!("{} finding(s)", CATEGORIES_BY_ORDINAL.len())));
    }

    // --- IMP-252: verbose/non-verbose RawLabel rendering ---

    #[test]
    fn render_non_verbose_aggregates_raw_labels() {
        let findings: Vec<Finding> = (0..5)
            .map(|i| Finding {
                category: Category::RawLabel,
                entity: Some(format!("mem_{i}")),
                message: format!("raw label: rel{i}"),
            })
            .collect();
        let out = render_findings(&findings, false);
        assert!(
            !out.contains("[Raw Label]"),
            "non-verbose must not show RawLabel header"
        );
        assert!(
            out.contains("Raw Label: 5 memory edge(s) use raw labels (expected)"),
            "non-verbose must show RawLabel count line: {out}"
        );
        // Summary line counts only non-RawLabel findings (0 in this case).
        assert!(
            out.contains("corpus clean") || out.contains("0 finding(s)"),
            "summary should not count RawLabel findings: {out}"
        );
    }

    #[test]
    fn render_verbose_shows_raw_labels_individually() {
        let findings: Vec<Finding> = (0..3)
            .map(|i| Finding {
                category: Category::RawLabel,
                entity: Some(format!("mem_{i}")),
                message: format!("raw label: rel{i}"),
            })
            .collect();
        let out = render_findings(&findings, true);
        assert!(
            out.contains("[Raw Label]"),
            "verbose must show RawLabel header"
        );
        assert!(
            out.contains("raw label: rel0"),
            "verbose must show individual findings: {out}"
        );
        assert!(
            out.contains("3 finding(s)"),
            "verbose summary must count all findings: {out}"
        );
    }

    #[test]
    fn render_non_verbose_mixed_raw_and_error() {
        let raw = Finding {
            category: Category::RawLabel,
            entity: Some("mem_a".into()),
            message: "raw label: test".into(),
        };
        let err = Finding {
            category: Category::IdIntegrity,
            entity: Some("SL-001".into()),
            message: "bad id".into(),
        };
        let out = render_findings(&[raw, err], false);
        // Error category still shows.
        assert!(out.contains("[Id Integrity]"));
        assert!(out.contains("bad id"));
        // RawLabel aggregated.
        assert!(!out.contains("[Raw Label]"));
        assert!(out.contains("Raw Label: 1 memory edge(s) use raw labels (expected)"));
        // Summary counts only non-RawLabel.
        assert!(out.contains("1 finding(s)"));
    }
}
