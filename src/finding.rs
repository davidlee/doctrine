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
}

impl Category {
    /// Single severity source (F5) — IdIntegrity/RelationIntegrity/SpecFk/MemoryHealth
    /// are errors; Lifecycle/RawLabel/TomlParse/ProseCite are warnings.
    #[must_use]
    pub(crate) const fn severity(self) -> Severity {
        match self {
            Self::IdIntegrity | Self::RelationIntegrity | Self::SpecFk | Self::MemoryHealth => {
                Severity::Error
            }
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
const CATEGORIES_BY_ORDINAL: [Category; 8] = [
    Category::IdIntegrity,
    Category::RelationIntegrity,
    Category::SpecFk,
    Category::MemoryHealth,
    Category::Lifecycle,
    Category::RawLabel,
    Category::TomlParse,
    Category::ProseCite,
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
pub(crate) fn render_findings(findings: &[Finding]) -> String {
    let mut by_category: [Vec<&Finding>; 8] = [
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

    for cat in &CATEGORIES_BY_ORDINAL {
        let idx = usize::from(cat.ordinal());
        let Some(group) = by_category.get(idx) else {
            continue;
        };
        if group.is_empty() {
            continue;
        }
        let _header = writeln!(out, "[{}]", cat.display_name());
        for f in group {
            let _line = writeln!(out, "  {}: {}", f.category.severity(), f.message);
            total = total.saturating_add(1);
        }
    }

    if total == 0 {
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
        let out = render_findings(&[]);
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
        let out = render_findings(&[f1, f2]);
        assert!(out.contains(CATEGORY_NAME_ID_INTEGRITY));
        assert!(out.contains(CATEGORY_NAME_LIFECYCLE));
        assert!(out.contains("2 finding(s)"));
    }
}
