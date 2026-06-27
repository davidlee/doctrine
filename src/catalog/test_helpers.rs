// SPDX-License-Identifier: GPL-3.0-only
//! Shared test helpers for catalog sub-modules (SL-071 PHASE-07).
//!
//! Compiles only under `#[cfg(test)]` — pulled in by scan, hydrate, and graph
//! test modules, and by the map-server route tests (SL-072 PHASE-05).
//! `pub(crate)` visibility since PHASE-05.

use std::fs;
use std::path::Path;

use crate::test_support::SCHEMA_KNOWLEDGE;

/// Write `root/<rel>` with `body`, creating parents.
#[allow(dead_code)]
pub(crate) fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

#[allow(dead_code)]
pub(crate) fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

/// Format `[[relation]]` rows from (label, targets) pairs.
/// Uses the `Vec<String>` + `concat()` pattern (house style) —
/// compatible with clippy's `push_str(&format!(…))` deny in bin/lib.
#[allow(dead_code)]
pub(crate) fn relation_rows(edges: &[(&str, &[&str])]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for (label, targets) in edges {
        // SL-149: a `references(<role>)` label string expands to a roled row
        // (`label = "references"` + `role = "<role>"`); any other label stays roleless.
        let (label, role) = match label
            .strip_prefix("references(")
            .and_then(|s| s.strip_suffix(')'))
        {
            Some(role) => ("references", Some(role)),
            None => (*label, None),
        };
        let role_line = role
            .map(|r| format!("role = \"{r}\"\n"))
            .unwrap_or_default();
        for t in *targets {
            parts.push(format!(
                "[[relation]]\nlabel = \"{label}\"\n{role_line}target = \"{t}\"\n"
            ));
        }
    }
    parts.concat()
}

/// Seed a slice entity (toml + md) with the given `[[relation]]` edges.
#[allow(dead_code)]
pub(crate) fn seed_slice(root: &Path, id: u32, edges: &[(&str, &[&str])]) {
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

/// Seed an ADR entity (toml + md) with the given `[[relation]]` edges (SL-026
/// PHASE-02, RV-093 F-2). Generalised from a `supersedes`-only array to arbitrary
/// `(label, targets)` edges — mirrors [`seed_slice`]'s signature and routes through
/// the same [`relation_rows`] emitter (the golden corpus needs a non-`supersedes`
/// ADR edge, e.g. `related`). Behaviour-preserving: the old
/// `seed_adr(root, 1, &["ADR-001"])` is now
/// `seed_adr(root, 1, &[("supersedes", &["ADR-001"])])`, parsing to the same
/// `supersedes` edge; an empty edge slice writes a relation-free toml.
#[allow(dead_code)]
pub(crate) fn seed_adr(root: &Path, id: u32, edges: &[(&str, &[&str])]) {
    write(
        root,
        &format!(".doctrine/adr/{id:03}/adr-{id:03}.toml"),
        &format!(
            "id = {id}\nslug = \"a{id}\"\ntitle = \"A{id}\"\nstatus = \"accepted\"\n\
             created = \"2026-01-01\"\nupdated = \"2026-01-01\"\n{}",
            relation_rows(edges)
        ),
    );
    write(
        root,
        &format!(".doctrine/adr/{id:03}/adr-{id:03}.md"),
        "body\n",
    );
}

/// Seed a requirement entity. Carries the full `Requirement` shape — including the
/// `kind` field — so it loads through BOTH the lenient scan/`relation_edges` path
/// (an edge target) AND the strict `requirement::load`/`load_with_prose` path the
/// spec `render()` inline pass walks (SL-026 PHASE-03). Behaviour-preserving for the
/// pre-existing edge-target callers (they never read `kind`).
#[allow(dead_code)]
pub(crate) fn seed_requirement(root: &Path, id: u32) {
    write(
        root,
        &format!(".doctrine/requirement/{id:03}/requirement-{id:03}.toml"),
        &format!(
            "id = {id}\nslug = \"r{id}\"\ntitle = \"R{id}\"\n\
             status = \"active\"\nkind = \"functional\"\n"
        ),
    );
    write(
        root,
        &format!(".doctrine/requirement/{id:03}/requirement-{id:03}.md"),
        "r\n",
    );
}

/// Seed a knowledge record entity (assumption/decision/question/constraint).
#[allow(dead_code)]
pub(crate) fn seed_knowledge(root: &Path, prefix: &str, id: u32, title: &str, status: &str) {
    let record_kind = crate::knowledge::RecordKind::from_prefix(prefix)
        .unwrap_or_else(|| panic!("unknown knowledge prefix: {prefix}"));
    let kind_dir = record_kind.as_str();
    write(
        root,
        &format!(".doctrine/knowledge/{kind_dir}/{id:03}/record-{id:03}.toml"),
        &format!(
            "schema = \"{SCHEMA_KNOWLEDGE}\"\nversion = 1\nid = {id}\nslug = \"k{id}\"\ntitle = \"{title}\"\nstatus = \"{status}\"\nrecord_kind = \"{kind_dir}\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\ntags = []\n"
        ),
    );
    write(
        root,
        &format!(".doctrine/knowledge/{kind_dir}/{id:03}/record-{id:03}.md"),
        "body\n",
    );
}

/// Seed a spec entity loadable by the REAL spec readers (SL-026 PHASE-02). Mirrors
/// the `spec.rs` scaffold exactly so `crate::spec::relation_edges` /
/// `crate::spec::member_reqs` parse it: a `spec-NNN.toml` (identity + the tech-only
/// flat `descends_from`/`parent`), a `spec-NNN.md` prose body, a `members.toml`
/// (`[[member]]` rows for both subtypes), and — TECH only — an `interactions.toml`
/// (`[[edge]]` rows).
///
/// - `subtype` selects the tree (`PRODUCT_SPEC_KIND.dir` / `TECH_SPEC_KIND.dir`),
///   the `schema`/`kind` keys, and whether interactions are emitted.
/// - `members` are requirement FKs (`REQ-NNN`); a label/order is synthesised per row
///   (matching the real `members.toml` shape `req add` writes).
/// - `interactions` are spec FKs (`SPEC-NNN`); ignored for a product subtype (no
///   `interactions.toml` — absent, not empty; spec.rs §5.4).
/// - `lineage` carries the tech-only single-valued `descends_from`/`parent` flat
///   fields as `(field, target)` pairs; emit none for a product spec.
#[allow(dead_code)]
pub(crate) fn seed_spec(
    root: &Path,
    subtype: crate::spec::SpecSubtype,
    id: u32,
    members: &[&str],
    interactions: &[&str],
    lineage: &[(&str, &str)],
) {
    use crate::spec::SpecSubtype;
    let (dir, schema, kind) = match subtype {
        SpecSubtype::Product => (
            crate::spec::PRODUCT_SPEC_KIND.dir,
            "doctrine.spec.product",
            "product",
        ),
        SpecSubtype::Tech => (
            crate::spec::TECH_SPEC_KIND.dir,
            "doctrine.spec.tech",
            "tech",
        ),
    };
    // identity head + the (tech-only) single-valued lineage flat fields.
    let mut head: Vec<String> = vec![format!(
        "schema = \"{schema}\"\nversion = 1\nid = {id}\nslug = \"s{id}\"\n\
         title = \"S{id}\"\nstatus = \"draft\"\nkind = \"{kind}\"\ntags = []\n"
    )];
    for (field, target) in lineage {
        head.push(format!("{field} = \"{target}\"\n"));
    }
    write(
        root,
        &format!("{dir}/{id:03}/spec-{id:03}.toml"),
        &head.concat(),
    );
    write(root, &format!("{dir}/{id:03}/spec-{id:03}.md"), "scope\n");
    // members.toml — `[[member]]` rows, the shape `req add` appends; synthetic
    // label/order so a bare FK list is enough to author membership.
    let mut member_doc: Vec<String> = Vec::new();
    for (i, req) in members.iter().enumerate() {
        let order = i + 1;
        member_doc.push(format!(
            "[[member]]\nrequirement = \"{req}\"\nlabel = \"FR-{order:03}\"\norder = {order}\n"
        ));
    }
    write(
        root,
        &format!("{dir}/{id:03}/members.toml"),
        &member_doc.concat(),
    );
    // interactions.toml — tech-only `[[edge]]` rows (absent, not empty, on product).
    if subtype == SpecSubtype::Tech {
        let mut edge_doc: Vec<String> = Vec::new();
        for target in interactions {
            edge_doc.push(format!(
                "[[edge]]\ntarget = \"{target}\"\ntype = \"uses\"\n"
            ));
        }
        write(
            root,
            &format!("{dir}/{id:03}/interactions.toml"),
            &edge_doc.concat(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relation::RelationLabel;
    use crate::spec::SpecSubtype;

    /// SL-026 PHASE-02 exit criterion: a tree seeded by the promoted/new fixture
    /// helpers loads back THROUGH THE REAL LOADERS into the expected entities — the
    /// seeded files mirror the real scaffolds, not a bespoke format. Asserts the
    /// tech-spec members + interactions + lineage, the product-spec member, the
    /// backlog item, and the ADR's NON-`supersedes` (`related`) edge.
    #[test]
    fn seeded_fixture_round_trips_through_the_real_loaders() {
        let dir = tmp();
        let root = dir.path();

        // Product spec PRD-001 with one member (a requirement).
        seed_requirement(root, 5);
        seed_spec(root, SpecSubtype::Product, 1, &["REQ-005"], &[], &[]);
        // Tech spec SPEC-002: one member, one outbound interaction, and the
        // single-valued tech lineage flat fields.
        seed_spec(
            root,
            SpecSubtype::Tech,
            2,
            &["REQ-005"],
            &["SPEC-003"],
            &[("descends_from", "PRD-001"), ("parent", "SPEC-004")],
        );
        // ADR-001 with a NON-`supersedes` edge (the golden-corpus need, RV-093 F-2).
        seed_adr(root, 1, &[("related", &["ADR-002"])]);
        // A backlog item, seeded through the PROMOTED fixture builder.
        crate::backlog::test_support::write_fixture(
            root,
            crate::backlog::test_support::Fixture {
                kind: crate::backlog::ItemKind::Issue,
                id: 7,
                slug: "round-trip",
                title: "Round trip",
                status: "open",
                resolution: "",
                tags: &[],
                facet: None,
                rels: None,
            },
        );

        // Spec: the REAL relation reader parses both subtypes' on-disk files.
        let prd = crate::spec::relation_edges(SpecSubtype::Product, root, 1).unwrap();
        assert!(
            prd.iter()
                .any(|e| e.label == RelationLabel::Members && e.target == "REQ-005"),
            "product spec member read back: {prd:?}"
        );

        let spec = crate::spec::relation_edges(SpecSubtype::Tech, root, 2).unwrap();
        let has = |label: RelationLabel, target: &str| {
            spec.iter().any(|e| e.label == label && e.target == target)
        };
        assert!(
            has(RelationLabel::Members, "REQ-005"),
            "tech member: {spec:?}"
        );
        assert!(
            has(RelationLabel::Interactions, "SPEC-003"),
            "tech interaction: {spec:?}"
        );
        assert!(
            has(RelationLabel::DescendsFrom, "PRD-001"),
            "tech descends_from: {spec:?}"
        );
        assert!(
            has(RelationLabel::Parent, "SPEC-004"),
            "tech parent: {spec:?}"
        );

        // ADR: the REAL governance relation reader yields the `related` edge.
        let adr = crate::governance::relation_edges(&crate::adr::ADR_KIND, root, 1).unwrap();
        assert!(
            adr.iter()
                .any(|e| e.label == RelationLabel::Related && e.target == "ADR-002"),
            "ADR non-supersedes (related) edge read back: {adr:?}"
        );

        // Backlog: the REAL reader loads the item seeded via the promoted builder.
        let items = crate::backlog::read_all(root).unwrap();
        let item = items
            .iter()
            .find(|i| i.kind.canonical_id(i.id) == "ISS-007")
            .expect("seeded backlog item read back");
        assert_eq!(item.title, "Round trip");
    }
}
