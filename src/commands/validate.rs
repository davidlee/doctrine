// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine validate` — corpus integrity scan.
//! SL-129: uses `entity::id_path`, `relation_graph`

/// `doctrine validate` — the corpus integrity scan. The COMMAND-LAYER composition
/// (mirrors `run_inspect`, ADR-001): the one layer allowed to depend on BOTH the
/// `integrity` id-scan AND the `relation_graph` relation-edge walk (which depends back
/// on `integrity` — composing them here keeps that edge acyclic). Resolves the root
/// ONCE, concatenates the id-integrity findings (D3 detect-half) with the SL-048
/// relation findings (danglers, `IllegalRows`, supersession drift — §5.5), prints them,
/// and exits non-zero on any. All report-only; nothing is rewritten (the reseat
/// precedent).
pub(crate) fn run_validate(path: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;

    let mut findings = crate::integrity::id_integrity_findings(&root)?;
    findings.extend(crate::relation_graph::validate_relations(&root)?);

    writeln!(
        std::io::stdout(),
        "validate: scanned {}",
        crate::integrity::scanned_kinds()
    )?;
    if findings.is_empty() {
        writeln!(std::io::stdout(), "validate: corpus clean")?;
        return Ok(());
    }
    for f in &findings {
        writeln!(std::io::stdout(), "  {f}")?;
    }
    anyhow::bail!("validate: {} finding(s)", findings.len())
}
