// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine doctor` — corpus health scan.
//!
//! Runs all nine checks (id integrity, relation integrity, spec FK, memory health,
//! lifecycle, raw label, TOML parse, prose citation, agent conformance) over the
//! corpus, renders them
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

    // #9 — Agent Conformance (Error) — SL-198 RSK-225: worker tool-surface is a
    // jail wall; scan authored agent-defs under install/agents + .doctrine/agents.
    findings.extend(crate::doctor_checks::agent_conformance_findings(&root));

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

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    //! VT-1 (SL-198 PHASE-04): the #9 conformance lint fails an unmarked worker
    //! def, a marked def with an extra writable MCP token, and a bare
    //! `mcp__doctrine` grant; it passes the pinned dispatch-worker (marked, only
    //! `mcp__doctrine__worker_commit`). Scan roots are the authored trees
    //! `install/agents` + `.doctrine/agents`, never the installed `.claude` copy.
    use crate::doctor_checks::agent_conformance_findings;
    use crate::finding::{Category, Severity};

    fn write_def(root: &std::path::Path, rel: &str, body: &str) {
        let path = root.join("install/agents").join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, body).unwrap();
    }

    #[test]
    fn conformance_lint_pins_worker_tool_surface() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // PASS: the pinned dispatch-worker — marked, only the sanctioned token.
        write_def(
            root,
            "claude/dispatch-worker.md",
            "---\nname: dispatch-worker\ndoctrine-role: worker\ntools: Read, Edit, Write, Bash, Grep, Glob, mcp__doctrine__worker_commit\n---\nbody\n",
        );
        // FAIL a: unmarked def (deny-by-default).
        write_def(
            root,
            "claude/unmarked.md",
            "---\nname: unmarked\ntools: Read, Edit\n---\nbody\n",
        );
        // FAIL b: marked def with an extra writable MCP token.
        write_def(
            root,
            "claude/extra.md",
            "---\nname: extra\ndoctrine-role: worker\ntools: Read, mcp__doctrine__worker_commit, mcp__slack__post\n---\nbody\n",
        );
        // FAIL c: bare mcp__doctrine server grant.
        write_def(
            root,
            "claude/bare.md",
            "---\nname: bare\ndoctrine-role: worker\ntools: Read, mcp__doctrine\n---\nbody\n",
        );

        let findings = agent_conformance_findings(root);
        assert_eq!(
            findings.len(),
            3,
            "three defs violate, one passes: {findings:?}"
        );
        assert!(
            findings
                .iter()
                .all(|f| f.category == Category::AgentConformance)
        );
        assert!(
            findings
                .iter()
                .all(|f| f.category.severity() == Severity::Error)
        );

        let joined: String = findings.iter().filter_map(|f| f.entity.clone()).collect();
        assert!(joined.contains("unmarked.md"));
        assert!(joined.contains("extra.md"));
        assert!(joined.contains("bare.md"));
        assert!(
            !joined.contains("dispatch-worker.md"),
            "pinned worker must pass"
        );
    }
}
