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
//! A requirement has **no standalone CLI** in v1 — it is spec-mediated (§5.2):
//! `spec req add` (PHASE-03) is the producer, calling `reserve` → `set_kind`
//! (the D-1 overwrite) below. The parse-layer `Requirement` is read in production
//! by `load` (PHASE-04), the by-FK reader `spec show` resolves each member
//! through (and `spec validate` PHASE-05 reuses) — the last D-2 `dead_code` bridge
//! erased.

use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::entity::{self, Artifact, Fileset, Kind, ScaffoldCtx};
use crate::tomlfmt::toml_string;

/// Relative dir of the requirement tree inside the project root — one global tree,
/// one reservation namespace (§5.1). Distinct top-level tree, like ADR.
const REQUIREMENT_DIR: &str = ".doctrine/requirement";

/// The top-level reserved requirement kind: `requirement-NNN.toml` +
/// `requirement-NNN.md` + slug symlink. `prefix` is the canonical-id stem
/// (`REQ-007`); the file stem is `"requirement"`.
pub(crate) const REQUIREMENT_KIND: Kind = Kind {
    dir: REQUIREMENT_DIR,
    prefix: "REQ",
    scaffold: requirement_scaffold,
};

/// A requirement's nature: a functional behaviour or a quality attribute. Closed
/// set, kebab serde + `clap::ValueEnum` (the `spec req add --kind` selector).
/// Seeded `functional` by the template; overwritten post-reserve (D-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ReqKind {
    Functional,
    Quality,
}

impl ReqKind {
    /// The kebab `kind` string written to `requirement-NNN.toml` (matches the
    /// serde rename). Used by the D-1 post-reserve overwrite.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            ReqKind::Functional => "functional",
            ReqKind::Quality => "quality",
        }
    }
}

/// A requirement's lifecycle status — the **authored / normative** half of the
/// two-enum truth model (ADR-009 §3, SL-028 design §5.2). Closed set, kebab serde;
/// hand-edited, git is the trail (no `created`/`updated` stamps — §5.1/§5.3).
///
/// Vocabulary and meanings (in transition order):
/// - `Pending` — declared, not started.
/// - `InProgress` — under active work.
/// - `Active` — in force, verified.
/// - `Deprecated` — soft withdrawal: still honoured, discouraged.
/// - `Retired` — hard withdrawal: withdrawn with no successor.
/// - `Superseded` — replaced by a named successor (the `supersedes` edge).
///
/// `InProgress` and `Retired` are SL-028 additions; what *sets* them (the change
/// process / reconcile engine) is deferred (design §8 unset-variant risk, OQ-2).
///
/// **No derivation by design (ADR-009 §3, ADR-003 §4–§5; SL-028 D4/F17).** There is
/// deliberately no `ReqStatus = f(CoverageStatus)` mapping — no `reconcile`/`sync`/
/// `from_coverage` function, and the two enums never reference each other. Authored
/// status is reconciled against observed coverage by **explicit authorship**, never
/// derived by precedence (the named spec-driver divergence). The reconcile writer
/// is a deferred follow-on; the absence here is the contract, not an omission.
///
/// Derives `clap::ValueEnum` so it binds directly as the `spec req status --to`
/// selector (mirroring `ReqKind`'s `spec req add --kind` shape) — a closed set, so
/// an out-of-vocab `--to` is rejected by clap before the verb runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ReqStatus {
    Pending,
    InProgress,
    Active,
    Deprecated,
    Retired,
    Superseded,
}

impl ReqStatus {
    /// The kebab string for `spec show` render (matches the serde rename). Pure.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            ReqStatus::Pending => "pending",
            ReqStatus::InProgress => "in-progress",
            ReqStatus::Active => "active",
            ReqStatus::Deprecated => "deprecated",
            ReqStatus::Retired => "retired",
            ReqStatus::Superseded => "superseded",
        }
    }
}

/// The authored-status known-set for `--status` validation on requirement-rostering
/// list surfaces (`spec req list`), mirroring `spec`'s `SPEC_STATUSES`. Must stay in
/// lockstep with the [`ReqStatus`] variants — the `req_statuses_matches_the_variants`
/// drift canary pins it.
pub(crate) const REQ_STATUSES: &[&str] = &[
    "pending",
    "in-progress",
    "active",
    "deprecated",
    "retired",
    "superseded",
];

/// A requirement's **observed-evidence** status — the second half of the two-enum
/// truth model (ADR-009 §3; SL-028 design §5.2/§5.3). Closed set, kebab serde,
/// mirroring `ReqKind`'s derive shape.
///
/// Vocabulary: `Planned` (coverage intended) · `InProgress` (evidence being
/// gathered) · `Verified` (evidence confirms the requirement) · `Failed` (evidence
/// contradicts it) · `Blocked` (evidence cannot be gathered).
///
/// The observed-evidence half is consumed by the slice-side coverage substrate
/// (`crate::coverage`, SL-042 P2): a `CoverageEntry` carries its `CoverageStatus`.
/// The status is **observed**, never derived into `ReqStatus` (NF-001 / ADR-009 §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CoverageStatus {
    Planned,
    InProgress,
    Verified,
    Failed,
    Blocked,
}

/// The parse layer (entity-model tolerant-parse tier — §5.3). `title` keys the
/// shared-`Meta` convention (inquisition C2 — NOT `name`); `slug` is derived from
/// it. `description`/`tags`/`acceptance_criteria` default, so a minimal toml
/// parses and the optional facets round-trip edit-preservingly. Read in
/// production by `load` (the `spec show` reader, PHASE-04).
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
fn render_requirement_toml(id: u32, slug: &str, title: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/requirement.toml")?
        .replace("{{id}}", &id.to_string())
        .replace("{{slug}}", &toml_string(slug))
        .replace("{{title}}", &toml_string(title)))
}

/// Render `requirement-<id>.md` from the embedded template: `{{ref}}` (the
/// canonical id, e.g. `REQ-007`) + `{{title}}`. No YAML frontmatter — metadata
/// lives in the sister toml, not the prose.
fn render_requirement_md(canonical_id: &str, title: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/requirement.md")?
        .replace("{{ref}}", canonical_id)
        .replace("{{title}}", title))
}

/// The requirement fileset: sister TOML, prose body, and `<id>-<slug>` symlink,
/// all relative to the requirement-tree root — structurally `adr_scaffold` (§5.6).
/// Only reachable via `REQUIREMENT_KIND`, so it inherits its production-dead
/// status until the first spec caller.
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
// Imperative: reserve + the D-1 kind overwrite (spec-mediated entry points)
// ---------------------------------------------------------------------------

/// Reserve the next `REQ-NNN` and scaffold its fileset — the engine `Fresh`
/// claim (atomic; collision-proof). Step 2 of the `spec req add` two-tree write
/// (§5.4); the only requirement producer in v1 (no standalone CLI — spec-mediated).
pub(crate) fn reserve(
    root: &Path,
    slug: &str,
    title: &str,
    date: &str,
) -> anyhow::Result<entity::Materialised> {
    let trunk_ids = crate::git::trunk_entity_ids(root, REQUIREMENT_KIND.dir)?;
    entity::materialise(
        &REQUIREMENT_KIND,
        &entity::LocalFs,
        root,
        &entity::MaterialiseRequest::Fresh,
        &entity::Inputs { slug, title, date },
        &trunk_ids,
    )
}

/// The canonical FK string for a reserved requirement id (`REQ-007`). The `Kind`
/// is the single source of the prefix.
pub(crate) fn canonical_id(id: u32) -> String {
    format!("{}-{id:03}", REQUIREMENT_KIND.prefix)
}

/// Best-effort FK canonicalizer: parse-and-reformat if the FK is a valid (possibly
/// non-canonical) requirement ref, else pass it through unchanged.
///
/// `"REQ-1"` → `"REQ-001"`, `"REQ-007"` → `"REQ-007"`, `"garbage"` → `"garbage"`.
/// Unresolvable junk (bad prefix, non-numeric suffix, unknown prefix) is returned
/// verbatim so the registry still flags it as dangling.
pub(crate) fn canonicalize_fk(fk: &str) -> String {
    id_from_fk(fk).map_or_else(|_| fk.to_string(), canonical_id)
}

/// The requirement tree root under a project `root` — the dir `entity::scan_ids`
/// enumerates for the `spec validate` corpus scan. Keeps `REQUIREMENT_DIR` private.
pub(crate) fn tree_root(root: &Path) -> PathBuf {
    root.join(REQUIREMENT_DIR)
}

/// Parse a canonical requirement FK (`REQ-NNN`) into its numeric id. The prefix
/// is required and matched against `REQUIREMENT_KIND.prefix` — the single source,
/// never hardcoded at the call site (mirrors `spec::resolve_spec_ref`).
pub(crate) fn id_from_fk(canonical_fk: &str) -> anyhow::Result<u32> {
    let (prefix, num) = canonical_fk.rsplit_once('-').with_context(|| {
        format!("`{canonical_fk}` is not a canonical requirement ref (expected REQ-NNN)")
    })?;
    anyhow::ensure!(
        prefix == REQUIREMENT_KIND.prefix,
        "unexpected requirement prefix `{prefix}` in `{canonical_fk}` (expected {})",
        REQUIREMENT_KIND.prefix
    );
    num.parse()
        .with_context(|| format!("`{num}` is not a numeric id in `{canonical_fk}`"))
}

/// Read and parse a requirement by its canonical FK (`REQ-NNN`) — the production
/// reader `spec show` (PHASE-04) resolves each member through, and `spec validate`
/// (PHASE-05) reuses. Reads `requirement/NNN/requirement-NNN.toml` only (no
/// corpus scan). An absent dir or unparsable toml is an error (a dangling FK is
/// `validate`'s concern; here it surfaces as a read failure).
pub(crate) fn load(root: &Path, canonical_fk: &str) -> anyhow::Result<Requirement> {
    let id = id_from_fk(canonical_fk)?;
    let name = format!("{id:03}");
    let path = root
        .join(REQUIREMENT_DIR)
        .join(&name)
        .join(format!("requirement-{name}.toml"));
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("requirement {canonical_fk} not found at {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("Failed to parse {}", path.display()))
}

/// Overwrite the template-seeded `kind` on a reserved requirement (D-1). Edit-
/// preserving `toml_edit` on `requirement-NNN.toml`, mirroring
/// `adr::set_adr_status`: parse → mutate in place → write, so comments / unknown
/// keys survive. `kind` is scaffold-seeded, so its absence is a malformed file.
pub(crate) fn set_kind(root: &Path, id: u32, kind: ReqKind) -> anyhow::Result<()> {
    let name = format!("{id:03}");
    let path = root
        .join(REQUIREMENT_DIR)
        .join(&name)
        .join(format!("requirement-{name}.toml"));
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("requirement {name} not found at {}", path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    let table = doc.as_table_mut();
    if !table.contains_key("kind") {
        anyhow::bail!("malformed requirement {name}: missing `kind` (regenerate via the scaffold)");
    }
    table.insert("kind", toml_edit::value(kind.as_str()));
    std::fs::write(&path, doc.to_string())
        .with_context(|| format!("Failed to write {}", path.display()))
}

/// Edit-preserving `status` transition on a requirement's authored
/// `requirement-NNN.toml` (SL-044 B·P1, design §5.2 / D-B4 / D-B6). FREE any→any —
/// `ReqStatus` enforces NO lifecycle order: any status → any status is accepted,
/// including backward (`active`→`pending`), same-state (no-op), and leaving
/// `retired` (a mis-`retired` requirement MUST be un-retirable). This mirrors
/// `governance::set_status`'s free shape, NOT the ordered slice FSM.
///
/// `toml_edit` mutates the file in place — the `[relationships]` table, comments,
/// and unknown keys all survive verbatim (never a parse→serialise round-trip).
///
/// **Writes `status` and nothing else.** The requirement entity deliberately
/// carries no `created`/`updated` field (§5.1/§5.3 — git is the trail), so unlike
/// `governance::set_status` there is NO stamp; only the one managed key moves.
///
/// Carries the no-op guard (an unchanged status writes nothing — content + mtime
/// hold) before the F-1 malformed-refuse (a missing scaffold-seeded `status` would
/// otherwise tail-insert *inside* the trailing `[relationships]` subtable = silent
/// corruption, so `bail!` instead).
pub(crate) fn set_status(root: &Path, id: u32, status: ReqStatus) -> anyhow::Result<()> {
    let name = format!("{id:03}");
    let path = root
        .join(REQUIREMENT_DIR)
        .join(&name)
        .join(format!("requirement-{name}.toml"));
    // STATUS-ONLY: one managed key, NO `updated` stamp (the requirement carries no
    // updated field — §5.1/§5.3, git is the trail). Delegate the write-core to the
    // shared seam; the single-element `managed` slice proves the variable-length
    // shape. The F-1 hint is non-destructive (EX-4): restore the seeded key, never
    // regenerate.
    let hint = format!(
        "malformed requirement {name}: missing seeded `status` — restore the seeded key before the transition; the file is left untouched"
    );
    crate::dep_seq::set_authored_status(&path, &[("status", status.as_str())], &hint)?;
    Ok(())
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

    /// Drift canary: `REQ_STATUSES` must stay in lockstep with the `ReqStatus`
    /// variants' kebab serde — adding a variant without the const (or vice versa)
    /// would desync the `spec req list --status` known-set (mirrors spec's
    /// `spec_statuses_matches_the_variants`).
    #[test]
    fn req_statuses_matches_the_variants() {
        let from_variants: Vec<&str> = [
            ReqStatus::Pending,
            ReqStatus::InProgress,
            ReqStatus::Active,
            ReqStatus::Deprecated,
            ReqStatus::Retired,
            ReqStatus::Superseded,
        ]
        .iter()
        .map(|s| s.as_str())
        .collect();
        assert_eq!(from_variants, REQ_STATUSES);
    }

    #[test]
    fn render_requirement_toml_escapes_hostile_title_and_slug() {
        // SL-024: quoted-literal breakers (`"`, `\`, newline) round-trip.
        let title = crate::tomlfmt::HOSTILE_TITLE;
        let slug = crate::tomlfmt::HOSTILE_SLUG;
        let body = render_requirement_toml(7, slug, title).unwrap();
        let parsed: Meta = toml::from_str(&body).unwrap();
        assert_eq!(parsed.slug, slug);
        assert_eq!(parsed.title, title);
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
                &[],
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

    // --- PHASE-03: reserve + the D-1 kind overwrite (the spec-mediated seam) ---

    #[test]
    fn reserve_then_set_kind_overwrites_seed_edit_preservingly() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        let m = reserve(root, "fast-boot", "Fast boot", "2026-06-05").unwrap();
        let id = m.eid.numeric_id().unwrap();
        assert_eq!(id, 1);
        assert_eq!(canonical_id(id), "REQ-001");

        let toml = root.join(REQUIREMENT_DIR).join("001/requirement-001.toml");
        // the template seeds `functional` (D-1) …
        assert!(
            fs::read_to_string(&toml)
                .unwrap()
                .contains("kind = \"functional\"")
        );

        set_kind(root, id, ReqKind::Quality).unwrap();
        let body = fs::read_to_string(&toml).unwrap();
        // … overwritten to the real kind, edit-preservingly (unrelated comments +
        // keys survive — toml_edit, not a reserialize).
        assert!(body.contains("kind = \"quality\""));
        assert!(!body.contains("kind = \"functional\""));
        assert!(body.contains("# description — optional"));
        assert!(body.contains("acceptance_criteria = []"));
    }

    #[test]
    fn load_reads_requirement_by_canonical_fk() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        reserve(root, "fast-boot", "Fast boot", "2026-06-05").unwrap();
        set_kind(root, 1, ReqKind::Quality).unwrap();

        let req = load(root, "REQ-001").unwrap();
        assert_eq!(req.id, 1);
        assert_eq!(req.title, "Fast boot");
        assert_eq!(req.slug, "fast-boot");
        assert_eq!(req.kind, ReqKind::Quality); // the D-1 overwrite is observed

        // wrong prefix, missing id, and a non-numeric tail all error.
        assert!(load(root, "PRD-001").is_err());
        assert!(load(root, "REQ-099").is_err());
        assert!(load(root, "REQ-x").is_err());
        assert!(load(root, "001").is_err());
    }

    #[test]
    fn set_kind_on_a_malformed_requirement_missing_kind_errors() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let id = reserve(root, "s", "T", "2026-06-05")
            .unwrap()
            .eid
            .numeric_id()
            .unwrap();
        // strip the seeded `kind` line → malformed; the verb refuses (no blind insert).
        let toml = root.join(REQUIREMENT_DIR).join("001/requirement-001.toml");
        let stripped: String = fs::read_to_string(&toml)
            .unwrap()
            .lines()
            .filter(|l| !l.trim_start().starts_with("kind ="))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&toml, stripped).unwrap();
        assert!(set_kind(root, id, ReqKind::Quality).is_err());
    }

    // --- SL-044 B·P1: the `status` setter (VT-1/VT-2/VT-3) ---

    /// Reserve a requirement and hand-add a comment, an inert `[relationships]`
    /// table, and an unknown key — the surfaces edit-preservation must keep.
    fn reserve_with_extras(root: &Path) -> std::path::PathBuf {
        let id = reserve(root, "fast-boot", "Fast boot", "2026-06-05")
            .unwrap()
            .eid
            .numeric_id()
            .unwrap();
        let toml = root
            .join(REQUIREMENT_DIR)
            .join(format!("{id:03}/requirement-{id:03}.toml"));
        let augmented = format!(
            "{}\n# hand-added note\nfuture_key = \"survives\"\n\n[relationships]\nsupersedes = \"REQ-009\"\n",
            fs::read_to_string(&toml).unwrap()
        );
        fs::write(&toml, augmented).unwrap();
        toml
    }

    #[test]
    fn set_status_round_trips_edit_preservingly_with_no_updated_stamp() {
        // VT-1: status moves; comments, [relationships], and unknown keys all
        // survive; NO `updated` key is introduced (the requirement carries none).
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let toml = reserve_with_extras(root);

        set_status(root, 1, ReqStatus::Active).unwrap();
        let body = fs::read_to_string(&toml).unwrap();
        assert!(body.contains("status = \"active\""));
        assert!(!body.contains("status = \"pending\""));
        // edit-preservation: the inert table, comment, and unknown key all held.
        assert!(body.contains("# hand-added note"));
        assert!(body.contains("future_key = \"survives\""));
        assert!(body.contains("[relationships]"));
        assert!(body.contains("supersedes = \"REQ-009\""));
        // the entity carries no created/updated stamp — none must appear.
        assert!(!body.contains("updated"));
        assert!(!body.contains("created"));
    }

    #[test]
    fn set_status_is_free_any_to_any_backward_and_same_state() {
        // VT-2 / D-B6: no FSM order, no terminal guard. A backward move
        // (active→pending) and a same-state set both succeed.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let toml = reserve_with_extras(root);

        set_status(root, 1, ReqStatus::Active).unwrap();
        // backward: active → pending (an ordered FSM would refuse this).
        set_status(root, 1, ReqStatus::Pending).unwrap();
        assert!(
            fs::read_to_string(&toml)
                .unwrap()
                .contains("status = \"pending\"")
        );

        // leaving a "terminal" status is allowed — a mis-retired req is un-retirable.
        set_status(root, 1, ReqStatus::Retired).unwrap();
        set_status(root, 1, ReqStatus::Active).unwrap();
        assert!(
            fs::read_to_string(&toml)
                .unwrap()
                .contains("status = \"active\"")
        );

        // same-state set is accepted (no-op): it writes nothing but does not error.
        set_status(root, 1, ReqStatus::Active).unwrap();
        assert!(
            fs::read_to_string(&toml)
                .unwrap()
                .contains("status = \"active\"")
        );
    }

    #[test]
    fn set_status_no_op_writes_nothing() {
        // the no-op guard holds content + mtime when status is unchanged.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let toml = reserve_with_extras(root);
        let before = fs::read_to_string(&toml).unwrap();

        set_status(root, 1, ReqStatus::Pending).unwrap(); // seeded pending → no-op
        assert_eq!(fs::read_to_string(&toml).unwrap(), before);
    }

    #[test]
    fn set_status_on_unknown_id_errors() {
        // VT-3: an id with no requirement dir is a clear read failure (by id only).
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        assert!(set_status(root, 99, ReqStatus::Active).is_err());
    }

    #[test]
    fn set_status_on_malformed_requirement_missing_status_refuses() {
        // F-1: a hand-stripped `status` key is malformed — refuse, never blind-insert.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let toml = reserve_with_extras(root);
        let stripped: String = fs::read_to_string(&toml)
            .unwrap()
            .lines()
            .filter(|l| !l.trim_start().starts_with("status ="))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&toml, stripped).unwrap();
        let err = set_status(root, 1, ReqStatus::Active).unwrap_err();
        let msg = err.to_string().to_lowercase();
        assert!(msg.contains("malformed"), "{msg}");
        // EX-4: the refuse is non-destructive — never instructs regeneration.
        assert!(
            !msg.contains("regenerate") && !msg.contains("scaffold") && !msg.contains(" new`"),
            "F-1 refuse must be non-destructive: {msg}"
        );
    }

    #[test]
    fn id_from_fk_rejects_slug_and_wrong_prefix() {
        // VT-3: resolution is by id only — no slug/title derivation path exists.
        assert_eq!(id_from_fk("REQ-007").unwrap(), 7);
        assert_eq!(id_from_fk("REQ-1").unwrap(), 1);
        assert!(id_from_fk("REQ-fast-boot").is_err()); // slug, not an id
        assert!(id_from_fk("PRD-001").is_err()); // wrong prefix
        assert!(id_from_fk("007").is_err()); // bare, no prefix
    }

    #[test]
    fn canonicalize_fk_normalises_and_passes_through_garbage() {
        // non-canonical (short number) → canonical zero-padded.
        assert_eq!(canonicalize_fk("REQ-1"), "REQ-001");
        assert_eq!(canonicalize_fk("REQ-007"), "REQ-007");
        // unresolvable junk is returned verbatim.
        assert_eq!(canonicalize_fk("garbage"), "garbage");
        assert_eq!(canonicalize_fk("REQ-x"), "REQ-x");
        // wrong prefix is also verbatim.
        assert_eq!(canonicalize_fk("PRD-001"), "PRD-001");
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

    // --- SL-028 PHASE-03: the two-enum lifecycle vocabulary (VT-1, VT-2) ---

    /// A toml-round-trip harness for a closed status enum: wrap a value in a
    /// single-field table, serialise, parse back, and compare. Exercises the
    /// serde rename in both directions through real toml (not just `as_str`).
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct StatusProbe<T> {
        status: T,
    }

    fn status_round_trips<T>(value: T, kebab: &str)
    where
        T: std::fmt::Debug + PartialEq + Copy + Serialize + serde::de::DeserializeOwned,
    {
        let body = toml::to_string(&StatusProbe { status: value }).unwrap();
        // the kebab token appears on the wire …
        assert!(
            body.contains(&format!("status = \"{kebab}\"")),
            "expected kebab `{kebab}` in: {body}"
        );
        // … and parses back to the same variant.
        let back: StatusProbe<T> = toml::from_str(&body).unwrap();
        assert_eq!(back, StatusProbe { status: value });
    }

    #[test]
    fn req_status_new_variants_serde_round_trip_and_render() {
        // VT-1: the two additive variants wire to their kebab tokens (serde
        // both directions) and render the same string via `as_str`.
        status_round_trips(ReqStatus::InProgress, "in-progress");
        status_round_trips(ReqStatus::Retired, "retired");
        assert_eq!(ReqStatus::InProgress.as_str(), "in-progress");
        assert_eq!(ReqStatus::Retired.as_str(), "retired");
        // and `as_str` matches the serde token across the whole six-token vocabulary.
        for s in [
            ReqStatus::Pending,
            ReqStatus::InProgress,
            ReqStatus::Active,
            ReqStatus::Deprecated,
            ReqStatus::Retired,
            ReqStatus::Superseded,
        ] {
            status_round_trips(s, s.as_str());
        }
    }

    #[test]
    fn coverage_status_serde_round_trips_all_five_variants() {
        // VT-2: the observed-evidence half round-trips through toml across every
        // variant — the only behaviour the stub carries (no producer/consumer yet).
        status_round_trips(CoverageStatus::Planned, "planned");
        status_round_trips(CoverageStatus::InProgress, "in-progress");
        status_round_trips(CoverageStatus::Verified, "verified");
        status_round_trips(CoverageStatus::Failed, "failed");
        status_round_trips(CoverageStatus::Blocked, "blocked");
    }
}
