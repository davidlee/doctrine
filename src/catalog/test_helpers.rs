// SPDX-License-Identifier: GPL-3.0-only
//! Shared test helpers for catalog sub-modules (SL-071 PHASE-07).
//!
//! Compiles only under `#[cfg(test)]` — pulled in by scan, hydrate, and graph
//! test modules. `pub(in crate::catalog)` visibility keeps these internal to
//! the catalog module while accessible from sibling sub-module test blocks.

use std::fs;
use std::path::Path;

/// Write `root/<rel>` with `body`, creating parents.
#[allow(dead_code)]
pub(in crate::catalog) fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

#[allow(dead_code)]
pub(in crate::catalog) fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

/// Format `[[relation]]` rows from (label, targets) pairs.
/// Uses the `Vec<String>` + `concat()` pattern (house style) —
/// compatible with clippy's `push_str(&format!(…))` deny in bin/lib.
#[allow(dead_code)]
pub(in crate::catalog) fn relation_rows(edges: &[(&str, &[&str])]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for (label, targets) in edges {
        for t in *targets {
            parts.push(format!(
                "[[relation]]\nlabel = \"{label}\"\ntarget = \"{t}\"\n"
            ));
        }
    }
    parts.concat()
}

/// Seed a slice entity (toml + md) with the given `[[relation]]` edges.
#[allow(dead_code)]
pub(in crate::catalog) fn seed_slice(root: &Path, id: u32, edges: &[(&str, &[&str])]) {
    write(
        root,
        &format!(".doctrine/slice/{id:03}/slice-{id:03}.toml"),
        &format!(
            "id = {id}\nslug = \"s{id}\"\ntitle = \"S{id}\"\nstatus = \"proposed\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n{}",
            relation_rows(edges)
        ),
    );
    write(
        root,
        &format!(".doctrine/slice/{id:03}/slice-{id:03}.md"),
        "scope\n",
    );
}

/// Seed an ADR entity (toml + md) with optional `supersedes` array.
#[allow(dead_code)]
pub(in crate::catalog) fn seed_adr(root: &Path, id: u32, supersedes: &[&str]) {
    let rels = if supersedes.is_empty() {
        String::new()
    } else {
        let refs: Vec<String> = supersedes.iter().map(|s| format!("\"{s}\"")).collect();
        format!("\n[relationships]\nsupersedes = [{}]\n", refs.join(", "))
    };
    write(
        root,
        &format!(".doctrine/adr/{id:03}/adr-{id:03}.toml"),
        &format!(
            "id = {id}\nslug = \"a{id}\"\ntitle = \"A{id}\"\nstatus = \"accepted\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"{rels}"
        ),
    );
    write(
        root,
        &format!(".doctrine/adr/{id:03}/adr-{id:03}.md"),
        "body\n",
    );
}

/// Seed a requirement entity (edge target only).
#[allow(dead_code)]
pub(in crate::catalog) fn seed_requirement(root: &Path, id: u32) {
    write(
        root,
        &format!(".doctrine/requirement/{id:03}/requirement-{id:03}.toml"),
        &format!("id = {id}\nslug = \"r{id}\"\ntitle = \"R{id}\"\nstatus = \"active\"\n"),
    );
    write(
        root,
        &format!(".doctrine/requirement/{id:03}/requirement-{id:03}.md"),
        "r\n",
    );
}
