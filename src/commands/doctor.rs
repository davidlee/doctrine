// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine doctor` — corpus health scan.
//!
//! Runs all eight checks (id integrity, relation integrity, spec FK, memory health,
//! lifecycle, raw label, TOML parse, prose citation) over the corpus, renders them
//! grouped by category with severity, and exits non-zero on any error-severity
//! finding. The `--json` flag emits a flat JSON array of finding objects.

use std::io::Write;
use std::path::PathBuf;

use crate::finding::{Category, Finding};

pub(crate) fn run_doctor(path: Option<PathBuf>, json: bool) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;

    let mut findings: Vec<Finding> = Vec::new();

    // #1 — Id Integrity (Error)
    findings.extend(crate::integrity::id_integrity_findings_native(&root)?);

    // #2 — Relation Integrity (Error)
    let rel_lines = crate::relation_graph::validate_relations(&root)?;
    findings.extend(Finding::from_lines(Category::RelationIntegrity, rel_lines));

    // #3 — Spec Foreign Key (Error)
    let fk_lines = crate::spec::spec_fk_findings(&root);
    findings.extend(Finding::from_lines(Category::SpecFk, fk_lines));

    // #4 — Memory Health (Error)
    let today = crate::clock::today();
    let mem_findings = match crate::memory::collect_memories(&root) {
        Ok(memories) => crate::memory::memory_health_findings_native(&root, &memories, &today),
        Err(_) => Vec::new(),
    };
    findings.extend(mem_findings);

    // #5 — Lifecycle (Warning)
    findings.extend(crate::backlog::lifecycle_findings(&root));

    // #6 — Raw Label (Warning)
    findings.extend(crate::doctor_checks::raw_label_findings(&root));

    // #7 — TOML Parse (Warning)
    findings.extend(crate::doctor_checks::toml_parse_findings(&root));

    // #8 — Prose Citation (Warning)
    findings.extend(crate::doctor_checks::prose_cite_findings(&root));

    if json {
        // Reuse the shared list envelope `{kind, rows}` (design §5.4 / F7) so the
        // doctor's --json matches the rest of the CLI's report surfaces (RV-185 F-5).
        let json_out = crate::listing::json_envelope("doctor", &findings)?;
        writeln!(std::io::stdout(), "{json_out}")?;
    } else {
        let rendered = crate::finding::render_findings(&findings);
        writeln!(std::io::stdout(), "{rendered}")?;
    }

    let has_errors = findings
        .iter()
        .any(|f| f.category.severity() == crate::finding::Severity::Error);
    if has_errors {
        anyhow::bail!("{} finding(s)", findings.len());
    }

    Ok(())
}
