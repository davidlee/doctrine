// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine requirement` — the durable atom a spec is woven from.
//!
//! A requirement is a numeric directory under `.doctrine/requirement/` holding a
//! sister `requirement-NNN.toml` (structured, queried metadata: `kind`, `status`,
//! `acceptance_criteria`) and a scaffolded `requirement-NNN.md` prose body
//! (statement, rationale), with an `NNN-slug` symlink alias — the ADR/slice shape
//! exactly (design §5.1/§5.6), so it rides `entity::Kind` over the same kind-blind
//! engine as a top-level reserved `Fresh` kind.
//!
//! This module owns the *requirement-specific* parts — the `Kind`, its scaffold,
//! the two render fns, and the parse-layer `Requirement` struct. The kind-agnostic
//! machinery lives in `crate::entity`; the shared metadata-list substrate
//! (`Meta`, list reader/formatter) in `crate::meta`, which a requirement's
//! `requirement-NNN.toml` round-trips into (its `id/slug/title/status` keys match
//! `Meta`; `kind`/`tags`/`acceptance_criteria` are unknown-to-`Meta`, so they are
//! ignored on read and preserved on disk).
//!
//! A requirement has **no standalone CLI** in v1 — it is spec-mediated (§5.2): the
//! `spec` verbs (PHASE-02+) are its only callers, so the items here are
//! production-dead until that first caller lands. The `cfg_attr(not(test),
//! expect(dead_code, …))` bridge below holds the gate green meanwhile and
//! self-erases the moment `spec.rs` references them (D-2 / memory
//! `mem.pattern.lint.expect-not-allow`).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::entity::{Artifact, Fileset, Kind, ScaffoldCtx};

/// Relative dir of the requirement tree inside the project root — one global tree,
/// one reservation namespace (§5.1). Distinct top-level tree, like ADR.
const REQUIREMENT_DIR: &str = ".doctrine/requirement";

/// The top-level reserved requirement kind: `requirement-NNN.toml` +
/// `requirement-NNN.md` + slug symlink. `prefix` is the canonical-id stem
/// (`REQ-007`); the file stem is `"requirement"`.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "first prod caller PHASE-02 (spec.rs); remove then"
    )
)]
const REQUIREMENT_KIND: Kind = Kind {
    dir: REQUIREMENT_DIR,
    prefix: "REQ",
    scaffold: requirement_scaffold,
};

/// A requirement's nature: a functional behaviour or a quality attribute. Closed
/// set, kebab serde. NOT a `clap::ValueEnum` yet — `spec req add --kind` is
/// PHASE-03. Seeded `functional` by the template; overwritten post-reserve (D-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ReqKind {
    Functional,
    Quality,
}

/// A requirement's lifecycle status. Closed set, kebab serde; hand-edited, git is
/// the trail (no `created`/`updated` stamps — §5.1/§5.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ReqStatus {
    Pending,
    Active,
    Deprecated,
    Superseded,
}

/// The parse layer (entity-model tolerant-parse tier — §5.3). `title` keys the
/// shared-`Meta` convention (inquisition C2 — NOT `name`); `slug` is derived from
/// it. `description`/`tags`/`acceptance_criteria` default, so a minimal toml
/// parses and the optional facets round-trip edit-preservingly.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "first prod caller PHASE-02 (spec.rs); remove then"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct Requirement {
    pub(crate) id: u32,
    pub(crate) title: String,
    pub(crate) slug: String,
    pub(crate) status: ReqStatus,
    pub(crate) kind: ReqKind,
    #[serde(default)]
    pub(crate) description: Option<String>,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    #[serde(default)]
    pub(crate) acceptance_criteria: Vec<String>,
}

// ---------------------------------------------------------------------------
// Pure: render, scaffold
// ---------------------------------------------------------------------------

/// Render `requirement-<id>.toml` from the embedded template by token
/// substitution. The `id/slug/title/status` keys round-trip into `meta::Meta`
/// (VT-2). No `date` arg — the toml carries no date fields (§5.1/§5.3).
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "first prod caller PHASE-02 (spec.rs); remove then"
    )
)]
fn render_requirement_toml(id: u32, slug: &str, title: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/requirement.toml")?
        .replace("{{id}}", &id.to_string())
        .replace("{{slug}}", slug)
        .replace("{{title}}", title))
}

/// Render `requirement-<id>.md` from the embedded template: `{{ref}}` (the
/// canonical id, e.g. `REQ-007`) + `{{title}}`. No YAML frontmatter — metadata
/// lives in the sister toml, not the prose.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "first prod caller PHASE-02 (spec.rs); remove then"
    )
)]
fn render_requirement_md(canonical_id: &str, title: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/requirement.md")?
        .replace("{{ref}}", canonical_id)
        .replace("{{title}}", title))
}

/// The requirement fileset: sister TOML, prose body, and `<id>-<slug>` symlink,
/// all relative to the requirement-tree root — structurally `adr_scaffold` (§5.6).
/// Only reachable via `REQUIREMENT_KIND`, so it inherits its production-dead
/// status until the first spec caller.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "first prod caller PHASE-02 (spec.rs); remove then"
    )
)]
fn requirement_scaffold(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
    let id = ctx.id;
    let name = format!("{id:03}");
    Ok(vec![
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/requirement-{name}.toml")),
            body: render_requirement_toml(id, ctx.slug, ctx.title)?,
        },
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/requirement-{name}.md")),
            body: render_requirement_md(ctx.canonical, ctx.title)?,
        },
        Artifact::Symlink {
            rel_path: PathBuf::from(format!("{name}-{}", ctx.slug)),
            target: name,
        },
    ])
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{self, Inputs, LocalFs, MaterialiseRequest};
    use crate::meta::Meta;
    use std::fs;
    use std::path::Path;

    // --- VT-1: render + scaffold shape ---

    #[test]
    fn render_requirement_toml_round_trips_to_metadata() {
        let body = render_requirement_toml(7, "fast-boot", "Fast boot").unwrap();
        // VT-2: the four list fields parse into meta::Meta (the `title` proof, C2) …
        let parsed: Meta = toml::from_str(&body).unwrap();
        assert_eq!(
            parsed,
            Meta {
                id: 7,
                slug: "fast-boot".to_string(),
                title: "Fast boot".to_string(),
                status: "pending".to_string(),
            }
        );
        // … and the full requirement parses, with kind seeded `functional` (D-1).
        let req: Requirement = toml::from_str(&body).unwrap();
        assert_eq!(req.kind, ReqKind::Functional);
        assert_eq!(req.status, ReqStatus::Pending);
        // no date token, no leftover placeholder.
        assert!(!body.contains("{{"));
        assert!(!body.contains("created"));
    }

    #[test]
    fn render_requirement_md_substitutes_ref_and_title_without_frontmatter() {
        let body = render_requirement_md("REQ-007", "Fast boot").unwrap();
        assert!(body.starts_with("# REQ-007: Fast boot"));
        assert!(!body.contains("{{ref}}"));
        assert!(!body.contains("{{title}}"));
        assert!(!body.starts_with("---"));
    }

    #[test]
    fn requirement_scaffold_lays_out_toml_md() {
        let ctx = ScaffoldCtx {
            id: 7,
            canonical: "REQ-007",
            slug: "fast-boot",
            title: "Fast boot",
            date: "2026-06-05", // ignored — requirement carries no date fields
        };
        let fileset = requirement_scaffold(&ctx).unwrap();
        assert_eq!(fileset.len(), 3);
        assert!(matches!(&fileset[0],
            Artifact::File { rel_path, body }
            if rel_path == Path::new("007/requirement-007.toml") && body.contains("status = \"pending\"")));
        assert!(matches!(&fileset[1],
            Artifact::File { rel_path, body }
            if rel_path == Path::new("007/requirement-007.md") && body.contains("REQ-007: Fast boot")));
        assert!(matches!(&fileset[2],
            Artifact::Symlink { rel_path, target }
            if rel_path == Path::new("007-fast-boot") && target == "007"));
    }

    // --- VT-1: materialise(Fresh) writes the tree and reserves REQ monotonically ---

    #[test]
    fn materialise_fresh_writes_the_tree_and_allocates_monotonically() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let mk = |slug: &str, title: &str| {
            entity::materialise(
                &REQUIREMENT_KIND,
                &LocalFs,
                root,
                &MaterialiseRequest::Fresh,
                &Inputs {
                    slug,
                    title,
                    date: "2026-06-05",
                },
            )
            .unwrap()
        };

        let first = mk("fast-boot", "Fast boot");
        assert_eq!(first.eid.numeric_id(), Some(1));

        let req = root.join(REQUIREMENT_DIR);
        assert!(req.join("001/requirement-001.toml").is_file());
        assert!(req.join("001/requirement-001.md").is_file());
        assert_eq!(
            fs::read_link(req.join("001-fast-boot")).unwrap(),
            Path::new("001")
        );

        // a second Fresh lands the next id (monotonic, engine race-retry inherited).
        let second = mk("low-latency", "Low latency");
        assert_eq!(second.eid.numeric_id(), Some(2));
        assert!(req.join("002/requirement-002.toml").is_file());

        // the canonical id carries the REQ prefix.
        let body = fs::read_to_string(req.join("001/requirement-001.md")).unwrap();
        assert!(body.contains("REQ-001"));
    }

    // --- VT-2: a full requirement toml round-trips; comments/unknown keys survive ---

    #[test]
    fn requirement_toml_parses_all_facets_and_into_meta() {
        let body = "\
id = 3
slug = \"fast-boot\"
title = \"Fast boot\"
status = \"active\"
kind = \"quality\"
description = \"boot under 200ms\"
tags = [\"perf\", \"ux\"]
acceptance_criteria = [\"cold boot < 200ms\", \"warm boot < 50ms\"]
";
        let req: Requirement = toml::from_str(body).unwrap();
        assert_eq!(req.kind, ReqKind::Quality);
        assert_eq!(req.status, ReqStatus::Active);
        assert_eq!(req.description.as_deref(), Some("boot under 200ms"));
        assert_eq!(req.tags, vec!["perf", "ux"]);
        assert_eq!(req.acceptance_criteria.len(), 2);

        // C2: the same toml deserialises into shared meta::Meta — proves `title`.
        let m: Meta = toml::from_str(body).unwrap();
        assert_eq!(m.title, "Fast boot");
        assert_eq!(m.status, "active");
    }

    #[test]
    fn requirement_toml_defaults_optional_facets() {
        // the minimal required set parses; description/tags/criteria default.
        let body = "\
id = 1
slug = \"s\"
title = \"T\"
status = \"pending\"
kind = \"functional\"
";
        let req: Requirement = toml::from_str(body).unwrap();
        assert_eq!(req.description, None);
        assert!(req.tags.is_empty());
        assert!(req.acceptance_criteria.is_empty());
    }

    #[test]
    fn requirement_toml_is_edit_preserving_through_toml_edit() {
        // full toml_edit round-trip is PHASE-03; here we assert the substrate
        // preserves a hand-added comment and an unknown key on read+rewrite.
        let body = "\
id = 1
slug = \"s\"
title = \"T\"  # hand-added note
status = \"pending\"
kind = \"functional\"
future_key = \"survives\"
";
        let doc = body.parse::<toml_edit::DocumentMut>().unwrap();
        let rewritten = doc.to_string();
        assert!(rewritten.contains("# hand-added note"));
        assert!(rewritten.contains("future_key = \"survives\""));
        // and it still parses into Requirement (unknown keys ignored).
        assert!(toml::from_str::<Requirement>(&rewritten).is_ok());
    }
}
