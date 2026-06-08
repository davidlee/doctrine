// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine backlog` — lightweight work-intake items (issue / improvement /
//! chore / risk / idea), each a numeric directory under
//! `.doctrine/backlog/<kind>/` holding a sister `backlog-NNN.toml` (structured,
//! queried metadata) and a scaffolded `backlog-NNN.md` prose body, with an
//! `NNN-slug` symlink alias — the ADR/spec/requirement shape (design §5.1/§5.3).
//!
//! Five `ItemKind`s ride five `entity::Kind`s over the same kind-blind engine,
//! each its own tree + reservation namespace (`ISS-001` and `RSK-001` coexist —
//! the counters are independent). The subtypes diverge only in their prefix and
//! whether the scaffold seeds a risk `[facet]`.
//!
//! This module owns the *backlog-specific* parts — the five `Kind`s, their shared
//! scaffold, the render fns, and the three-layer parse model (`RawBacklogToml`
//! tolerant parse → validated `BacklogItem`, with the `"" -> None` validation
//! seam for the optional `resolution`/risk-level fields). The kind-agnostic engine
//! is `crate::entity` (unchanged — five new `Fresh` callers only, the R6 gate).
//!
//! PHASE-01 is the model + scaffold half: NO CLI surface yet (PHASE-02 wires
//! `new`/`list`/`show`/`edit`). The model is reachable only via its `Kind`s and
//! the tests until those verbs land, so the whole module is production-dead this
//! phase — the module-level `#![expect(dead_code)]` is the bridge, retired the
//! moment a verb consumes each item (the `requirement.rs`/`retrieve.rs`
//! precedent). The inert `KIND_PRECEDENCE` const keeps the expectation fulfilled
//! in both the lib and test builds.
#![expect(
    dead_code,
    reason = "model + scaffold consumed by PHASE-02..05 verbs; no CLI this phase"
)]

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::entity::{Artifact, Fileset, Kind, ScaffoldCtx};

/// The toml/md file stem — shared by all five kinds (`backlog-NNN.toml`). Distinct
/// from each `Kind.prefix` (`ISS`/`IMP`/…) and from the per-kind tree dirs.
const BACKLOG_STEM: &str = "backlog";

// ---------------------------------------------------------------------------
// The discriminator + its five engine `Kind`s
// ---------------------------------------------------------------------------

/// Which backlog item this is. Closed set; kebab serde (round-trips the toml's
/// `kind`) and `clap::ValueEnum` (the `backlog new` positional, PHASE-02). Selects
/// the tree, prefix, and scaffold fileset. Fixed at capture (PRD-009 §4 invariant).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ItemKind {
    Issue,
    Improvement,
    Chore,
    Risk,
    Idea,
}

/// The issue kind: a defect / problem to fix. Own tree + reservation namespace.
const ISSUE_KIND: Kind = Kind {
    dir: ".doctrine/backlog/issue",
    prefix: "ISS",
    scaffold: |c| backlog_scaffold(ItemKind::Issue, c),
};

/// The improvement kind: an enhancement to existing behaviour.
const IMPROVEMENT_KIND: Kind = Kind {
    dir: ".doctrine/backlog/improvement",
    prefix: "IMP",
    scaffold: |c| backlog_scaffold(ItemKind::Improvement, c),
};

/// The chore kind: maintenance with no user-visible behaviour change.
const CHORE_KIND: Kind = Kind {
    dir: ".doctrine/backlog/chore",
    prefix: "CHR",
    scaffold: |c| backlog_scaffold(ItemKind::Chore, c),
};

/// The risk kind: a tracked risk — the only kind carrying a `[facet]`.
const RISK_KIND: Kind = Kind {
    dir: ".doctrine/backlog/risk",
    prefix: "RSK",
    scaffold: |c| backlog_scaffold(ItemKind::Risk, c),
};

/// The idea kind: a speculative possibility, not yet committed work.
const IDEA_KIND: Kind = Kind {
    dir: ".doctrine/backlog/idea",
    prefix: "IDE",
    scaffold: |c| backlog_scaffold(ItemKind::Idea, c),
};

/// Boundary precedence for the future multi-kind resolver (PRD-009 §4): when one
/// capture could match several kinds, `risk` wins, then issue/improvement/chore/
/// idea. INERT in v1 — `new` always takes an explicit kind, so this is never
/// exercised; recorded so the order is canon when the resolver lands (PRD-011).
/// (Deliberately referenced nowhere — it keeps the module `dead_code` expectation
/// fulfilled in both the lib and test builds while the verbs are unwired.)
const KIND_PRECEDENCE: [ItemKind; 5] = [
    ItemKind::Risk,
    ItemKind::Issue,
    ItemKind::Improvement,
    ItemKind::Chore,
    ItemKind::Idea,
];

impl ItemKind {
    /// The engine `Kind` for this item kind — the single source of its tree +
    /// prefix + scaffold.
    const fn kind(self) -> &'static Kind {
        match self {
            ItemKind::Issue => &ISSUE_KIND,
            ItemKind::Improvement => &IMPROVEMENT_KIND,
            ItemKind::Chore => &CHORE_KIND,
            ItemKind::Risk => &RISK_KIND,
            ItemKind::Idea => &IDEA_KIND,
        }
    }

    /// The canonical-id prefix (`ISS`/`IMP`/`CHR`/`RSK`/`IDE`), read off the
    /// `Kind` so the prefix is never hardcoded twice.
    const fn prefix(self) -> &'static str {
        self.kind().prefix
    }

    /// The kebab `kind` string written to `backlog-NNN.toml` (matches the serde
    /// rename). Pure; the render mirror for the stored `kind` field.
    const fn as_str(self) -> &'static str {
        match self {
            ItemKind::Issue => "issue",
            ItemKind::Improvement => "improvement",
            ItemKind::Chore => "chore",
            ItemKind::Risk => "risk",
            ItemKind::Idea => "idea",
        }
    }

    /// Resolve a canonical-id prefix back to its kind (`backlog show <ID>`
    /// auto-detect, PHASE-04). Prefixes come from the `Kind`s — the single source.
    fn from_prefix(prefix: &str) -> Option<Self> {
        [
            ItemKind::Issue,
            ItemKind::Improvement,
            ItemKind::Chore,
            ItemKind::Risk,
            ItemKind::Idea,
        ]
        .into_iter()
        .find(|k| k.prefix() == prefix)
    }

    /// Whether this kind carries a risk `[facet]` (risk only). Selects the
    /// scaffold template and gates facet render.
    const fn has_facet(self) -> bool {
        matches!(self, ItemKind::Risk)
    }
}

// ---------------------------------------------------------------------------
// Closed value enums (kebab serde + an `as_str` render mirror)
// ---------------------------------------------------------------------------

/// A backlog item's lifecycle status. Closed canon set, kebab serde; hand-settable
/// and ungated (slices/ADRs/specs ship this way). `status` is always seeded a real
/// value (`open`), so it serde-parses directly — never the `"" -> None` seam.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Status {
    Open,
    Triaged,
    Started,
    Resolved,
    Closed,
}

impl Status {
    /// The kebab string for render (matches the serde rename). Pure.
    const fn as_str(self) -> &'static str {
        match self {
            Status::Open => "open",
            Status::Triaged => "triaged",
            Status::Started => "started",
            Status::Resolved => "resolved",
            Status::Closed => "closed",
        }
    }

    /// Whether this status is terminal (`resolved`/`closed`). A **backlog-local**
    /// predicate — explicitly NOT `slice::is_terminal_status` (R4): backlog and
    /// slice lifecycles are independent vocabularies. Drives the `resolution ⟺
    /// terminal` coupling (`edit`, PHASE-05) and the hide-terminal `list` rule.
    const fn is_terminal(self) -> bool {
        matches!(self, Status::Resolved | Status::Closed)
    }
}

/// Why a terminal item was closed. One generic, kind-agnostic set (PRD-009): a
/// resolution is never a close *reason* hidden in a facet. Optional — present only
/// on a terminal item (the `"" -> None` seam).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Resolution {
    Fixed,
    Done,
    Mitigated,
    Accepted,
    Expired,
    Duplicate,
    WontDo,
    Obsolete,
    Promoted,
}

impl Resolution {
    /// The kebab string for render (matches the serde rename). Pure.
    const fn as_str(self) -> &'static str {
        match self {
            Resolution::Fixed => "fixed",
            Resolution::Done => "done",
            Resolution::Mitigated => "mitigated",
            Resolution::Accepted => "accepted",
            Resolution::Expired => "expired",
            Resolution::Duplicate => "duplicate",
            Resolution::WontDo => "wont-do",
            Resolution::Obsolete => "obsolete",
            Resolution::Promoted => "promoted",
        }
    }
}

/// A risk facet axis level. Closed set, kebab serde; tech of the risk `[facet]`,
/// optional (the `"" -> None` seam — seeded empty until assessed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    /// The kebab string for render (matches the serde rename). Pure.
    const fn as_str(self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        }
    }
}

// ---------------------------------------------------------------------------
// Three-layer parse model (the entity-model tolerant-parse tier — §5.3)
// ---------------------------------------------------------------------------

/// The tolerant parse layer. `resolution` and the risk levels are read as raw
/// `String` (they are seeded `""`, which is no enum variant — serde would reject
/// a direct `Option<Resolution>`), so the `"" -> None` mapping is a separate
/// `validate` pass, not a serde derive. `status`/`kind` carry real values and
/// parse to their enums directly. `#[serde(default)]` lets the seeded-empty
/// collections and the absent (non-risk) `[facet]` parse.
#[derive(Debug, Deserialize)]
struct RawBacklogToml {
    id: u32,
    slug: String,
    title: String,
    kind: ItemKind,
    status: Status,
    #[serde(default)]
    resolution: String,
    created: String,
    updated: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    facet: Option<RawRiskFacet>,
    #[serde(default)]
    relationships: Relationships,
}

/// The tolerant risk-facet layer: the two assessable axes as raw `String` (the
/// `"" -> None` seam), `origin` as raw `String` (empty → absent), `controls` a
/// free list.
#[derive(Debug, Deserialize)]
struct RawRiskFacet {
    #[serde(default)]
    likelihood: String,
    #[serde(default)]
    impact: String,
    #[serde(default)]
    origin: String,
    #[serde(default)]
    controls: Vec<String>,
}

/// The validated entity (design §5.2). `id/slug/title/status` are top-level in the
/// toml so the file also round-trips into the shared `meta::Meta`. `kind` is stored
/// AND implied by the tree dir — stored so one read yields the entity without path
/// inspection. The `"" -> None` optionals are resolved off the raw layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BacklogItem {
    id: u32,
    slug: String,
    title: String,
    kind: ItemKind,
    status: Status,
    resolution: Option<Resolution>,
    created: String,
    updated: String,
    tags: Vec<String>,
    facet: Option<RiskFacet>,
    relationships: Relationships,
}

/// The validated risk facet (risk only). Every axis typed — no untyped bag
/// (PRD-009 invariant). The assessable axes are optional until assessed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RiskFacet {
    likelihood: Option<RiskLevel>,
    impact: Option<RiskLevel>,
    origin: Option<String>,
    controls: Vec<String>,
}

/// Outbound-only relations (ADR-004): a backlog item points OUT at the slices,
/// specs, and drift it touches; the reverse view is derived (deferred, PRD-011).
/// Shared verbatim by the raw and validated layers (no `"" -> None` seam — these
/// are plain lists), seeded empty so `#[serde(default)]` parses a virgin item.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
struct Relationships {
    #[serde(default)]
    slices: Vec<String>,
    #[serde(default)]
    specs: Vec<String>,
    #[serde(default)]
    drift: Vec<String>,
}

/// Parse a kebab token into its closed enum via the serde derive — the single
/// source of the variant↔string mapping (the `as_str` mirrors render only).
/// Errors with serde's "unknown variant" message on a bad token (`what` names the
/// field for the message).
fn parse_enum<T: serde::de::DeserializeOwned>(token: &str, what: &str) -> anyhow::Result<T> {
    use serde::de::IntoDeserializer;
    let de: serde::de::value::StrDeserializer<'_, serde::de::value::Error> =
        token.into_deserializer();
    T::deserialize(de).map_err(|e| anyhow::anyhow!("invalid {what} `{token}`: {e}"))
}

/// The `"" -> None` seam for an optional closed enum: an empty token is absent; a
/// non-empty token parses to its variant (erroring on an unknown one).
fn optional_enum<T: serde::de::DeserializeOwned>(
    token: &str,
    what: &str,
) -> anyhow::Result<Option<T>> {
    if token.is_empty() {
        Ok(None)
    } else {
        parse_enum(token, what).map(Some)
    }
}

/// The `"" -> None` seam for an optional free-text field.
fn optional_text(text: String) -> Option<String> {
    if text.is_empty() { None } else { Some(text) }
}

/// Validate a tolerant `RawBacklogToml` into a typed `BacklogItem` — the second
/// layer of the parse model. Maps the seeded-`""` optionals to `None`, parses any
/// non-empty value to its enum (erroring on an unknown token), and validates the
/// risk facet when present. Consumes the raw layer (its owned strings move across).
fn validate(raw: RawBacklogToml) -> anyhow::Result<BacklogItem> {
    let resolution = optional_enum(&raw.resolution, "resolution")?;
    let facet = match raw.facet {
        Some(f) => Some(validate_facet(f)?),
        None => None,
    };
    Ok(BacklogItem {
        id: raw.id,
        slug: raw.slug,
        title: raw.title,
        kind: raw.kind,
        status: raw.status,
        resolution,
        created: raw.created,
        updated: raw.updated,
        tags: raw.tags,
        facet,
        relationships: raw.relationships,
    })
}

/// Validate a tolerant risk facet: the two axes through the `"" -> None` enum seam,
/// `origin` through the text seam, `controls` passed through.
fn validate_facet(raw: RawRiskFacet) -> anyhow::Result<RiskFacet> {
    Ok(RiskFacet {
        likelihood: optional_enum(&raw.likelihood, "likelihood")?,
        impact: optional_enum(&raw.impact, "impact")?,
        origin: optional_text(raw.origin),
        controls: raw.controls,
    })
}

// ---------------------------------------------------------------------------
// Pure: render, scaffold
// ---------------------------------------------------------------------------

/// Render `backlog-<id>.toml` from the kind's embedded template by token
/// substitution. Risk picks the `[facet]` template; the four plain kinds the
/// light one. The `id/slug/title/status` keys round-trip into `meta::Meta` (VT-2);
/// `{{kind}}` is the stored discriminator (also the tree dir).
fn render_backlog_toml(
    item_kind: ItemKind,
    id: u32,
    slug: &str,
    title: &str,
    date: &str,
) -> anyhow::Result<String> {
    let template = if item_kind.has_facet() {
        "templates/backlog-risk.toml"
    } else {
        "templates/backlog.toml"
    };
    Ok(crate::install::asset_text(template)?
        .replace("{{id}}", &id.to_string())
        .replace("{{slug}}", slug)
        .replace("{{title}}", title)
        .replace("{{kind}}", item_kind.as_str())
        .replace("{{date}}", date))
}

/// Render `backlog-<id>.md` from the embedded prose template: `{{ref}}` (the
/// canonical id, e.g. `ISS-007`) + `{{title}}`. No frontmatter — metadata lives in
/// the sister toml.
fn render_backlog_md(canonical_id: &str, title: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/backlog.md")?
        .replace("{{ref}}", canonical_id)
        .replace("{{title}}", title))
}

/// The backlog fileset: sister TOML, prose body, and `<id>-<slug>` symlink, all
/// relative to the kind's tree root — structurally `requirement_scaffold` (§5.6).
/// The `item_kind` decides only the toml template (risk vs plain); the md and
/// symlink are kind-uniform. Shared by all five `Kind`s via their scaffold closure.
fn backlog_scaffold(item_kind: ItemKind, ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
    let id = ctx.id;
    let name = format!("{id:03}");
    Ok(vec![
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/{BACKLOG_STEM}-{name}.toml")),
            body: render_backlog_toml(item_kind, id, ctx.slug, ctx.title, ctx.date)?,
        },
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/{BACKLOG_STEM}-{name}.md")),
            body: render_backlog_md(ctx.canonical, ctx.title)?,
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

    /// Every `ItemKind` — the table the per-kind assertions iterate.
    const ALL_KINDS: [ItemKind; 5] = [
        ItemKind::Issue,
        ItemKind::Improvement,
        ItemKind::Chore,
        ItemKind::Risk,
        ItemKind::Idea,
    ];

    fn ctx_for(item_kind: ItemKind) -> ScaffoldCtx<'static> {
        let canonical: &'static str = match item_kind {
            ItemKind::Issue => "ISS-003",
            ItemKind::Improvement => "IMP-003",
            ItemKind::Chore => "CHR-003",
            ItemKind::Risk => "RSK-003",
            ItemKind::Idea => "IDE-003",
        };
        ScaffoldCtx {
            id: 3,
            canonical,
            slug: "token-expiry",
            title: "Token expiry",
            date: "2026-06-08",
        }
    }

    fn fresh(root: &Path, item_kind: ItemKind, slug: &str, title: &str) -> entity::Materialised {
        entity::materialise(
            item_kind.kind(),
            &LocalFs,
            root,
            &MaterialiseRequest::Fresh,
            &Inputs {
                slug,
                title,
                date: "2026-06-08",
            },
        )
        .unwrap()
    }

    // --- VT-1: per-kind scaffold fileset ---

    #[test]
    fn backlog_scaffold_lays_out_toml_md_symlink() {
        for kind in ALL_KINDS {
            let ctx = ctx_for(kind);
            let fileset = backlog_scaffold(kind, &ctx).unwrap();
            assert_eq!(fileset.len(), 3, "{kind:?}: toml + md + symlink");

            assert!(
                matches!(&fileset[0],
                    Artifact::File { rel_path, body }
                    if rel_path == Path::new("003/backlog-003.toml")
                        && body.contains(&format!("kind = \"{}\"", kind.as_str()))),
                "{kind:?}: toml at tree-relative path with the stored kind"
            );
            assert!(
                matches!(&fileset[1],
                    Artifact::File { rel_path, body }
                    if rel_path == Path::new("003/backlog-003.md")
                        && body.contains(&format!("{}: Token expiry", ctx.canonical))),
                "{kind:?}: md carries the canonical ref"
            );
            assert!(
                matches!(&fileset[2],
                    Artifact::Symlink { rel_path, target }
                    if rel_path == Path::new("003-token-expiry") && target == "003"),
                "{kind:?}: NNN-slug alias last"
            );

            // risk carries `[facet]`; the four plain kinds omit it.
            let toml_body = match &fileset[0] {
                Artifact::File { body, .. } => body,
                Artifact::Symlink { .. } => panic!("first artifact is the toml"),
            };
            assert_eq!(
                toml_body.contains("[facet]"),
                kind.has_facet(),
                "{kind:?}: [facet] iff risk"
            );
        }
    }

    // --- VT-3: every kind seeds the mutable keys (the edit-in-place precondition) ---

    #[test]
    fn all_five_kinds_seed_status_resolution_updated_tags() {
        for kind in ALL_KINDS {
            let body = render_backlog_toml(kind, 1, "s", "T", "2026-06-08").unwrap();
            assert!(
                body.contains("status = \"open\""),
                "{kind:?}: status seeded"
            );
            assert!(
                body.contains("resolution = \"\""),
                "{kind:?}: resolution seeded"
            );
            assert!(
                body.contains("updated = \"2026-06-08\""),
                "{kind:?}: updated seeded"
            );
            assert!(body.contains("tags = []"), "{kind:?}: tags seeded");
            assert!(!body.contains("{{"), "{kind:?}: no token survives render");
        }
    }

    // --- VT-2: the shared-Meta + full-entity round-trip, and the "" -> None seam ---

    #[test]
    fn rendered_toml_round_trips_into_meta_and_backlog_item() {
        let body = render_backlog_toml(ItemKind::Issue, 7, "fast-boot", "Fast boot", "2026-06-08")
            .unwrap();

        // the four list fields parse into the shared meta::Meta (status is a String there).
        let meta: Meta = toml::from_str(&body).unwrap();
        assert_eq!(
            meta,
            Meta {
                id: 7,
                slug: "fast-boot".to_string(),
                title: "Fast boot".to_string(),
                status: "open".to_string(),
            }
        );

        // the full entity validates; the seeded resolution `""` maps to None.
        let item = validate(toml::from_str::<RawBacklogToml>(&body).unwrap()).unwrap();
        assert_eq!(item.kind, ItemKind::Issue);
        assert_eq!(item.status, Status::Open);
        assert_eq!(item.resolution, None);
        assert!(item.facet.is_none(), "a plain kind has no facet");
        assert_eq!(item.relationships, Relationships::default());
    }

    #[test]
    fn risk_facet_levels_map_empty_to_none_and_parse_non_empty() {
        // a seeded risk toml: every facet axis empty → None.
        let seeded = render_backlog_toml(ItemKind::Risk, 1, "r", "R", "2026-06-08").unwrap();
        let item = validate(toml::from_str::<RawBacklogToml>(&seeded).unwrap()).unwrap();
        let facet = item.facet.expect("risk carries a facet");
        assert_eq!(facet.likelihood, None);
        assert_eq!(facet.impact, None);
        assert_eq!(facet.origin, None);
        assert!(facet.controls.is_empty());

        // an assessed risk: non-empty axes parse to their levels.
        let assessed = "\
id = 1
slug = \"r\"
title = \"R\"
kind = \"risk\"
status = \"open\"
resolution = \"\"
created = \"2026-06-08\"
updated = \"2026-06-08\"
tags = []

[facet]
likelihood = \"high\"
impact = \"critical\"
origin = \"audit\"
controls = [\"rate-limit\"]

[relationships]
slices = [\"SL-020\"]
specs = []
drift = []
";
        let item = validate(toml::from_str::<RawBacklogToml>(assessed).unwrap()).unwrap();
        let facet = item.facet.unwrap();
        assert_eq!(facet.likelihood, Some(RiskLevel::High));
        assert_eq!(facet.impact, Some(RiskLevel::Critical));
        assert_eq!(facet.origin.as_deref(), Some("audit"));
        assert_eq!(facet.controls, vec!["rate-limit"]);
        assert_eq!(item.relationships.slices, vec!["SL-020"]);
    }

    #[test]
    fn validate_errors_on_an_unknown_enum_token() {
        let body = "\
id = 1
slug = \"s\"
title = \"T\"
kind = \"issue\"
status = \"open\"
resolution = \"bogus\"
created = \"2026-06-08\"
updated = \"2026-06-08\"
tags = []
";
        let raw: RawBacklogToml = toml::from_str(body).unwrap();
        assert!(
            validate(raw).is_err(),
            "an unknown resolution token is rejected"
        );
    }

    // --- the value mirrors + discriminator helpers ---

    #[test]
    fn status_is_terminal_is_backlog_local() {
        assert!(Status::Resolved.is_terminal());
        assert!(Status::Closed.is_terminal());
        assert!(!Status::Open.is_terminal());
        assert!(!Status::Triaged.is_terminal());
        assert!(!Status::Started.is_terminal());
    }

    #[test]
    fn item_kind_from_prefix_round_trips_each_kind() {
        for kind in ALL_KINDS {
            assert_eq!(ItemKind::from_prefix(kind.prefix()), Some(kind));
        }
        assert_eq!(ItemKind::from_prefix("REQ"), None);
        // the five prefixes are distinct.
        let prefixes: std::collections::BTreeSet<&str> =
            ALL_KINDS.iter().map(|k| k.prefix()).collect();
        assert_eq!(prefixes.len(), 5);
    }

    #[test]
    fn resolution_and_risk_level_render_mirror_serde() {
        assert_eq!(Resolution::WontDo.as_str(), "wont-do");
        assert_eq!(Resolution::Promoted.as_str(), "promoted");
        assert_eq!(RiskLevel::Critical.as_str(), "critical");
        // the mirror matches the parse direction.
        assert_eq!(
            parse_enum::<Resolution>("wont-do", "resolution").unwrap(),
            Resolution::WontDo
        );
    }

    // --- EX-1 / VT-1: materialise(Fresh) reserves per-kind, counters independent ---

    #[test]
    fn materialise_fresh_reserves_each_kind_in_its_own_namespace() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // issue and risk both start at 001 — independent reservation namespaces.
        let i1 = fresh(root, ItemKind::Issue, "auth", "Auth");
        let r1 = fresh(root, ItemKind::Risk, "expiry", "Expiry");
        assert_eq!(i1.eid.numeric_id(), Some(1));
        assert_eq!(r1.eid.numeric_id(), Some(1));

        assert!(
            root.join(".doctrine/backlog/issue/001/backlog-001.toml")
                .is_file()
        );
        assert!(
            root.join(".doctrine/backlog/issue/001/backlog-001.md")
                .is_file()
        );
        assert_eq!(
            fs::read_link(root.join(".doctrine/backlog/issue/001-auth")).unwrap(),
            Path::new("001")
        );

        // the risk item on disk carries the `[facet]`; the issue item does not.
        let risk_toml =
            fs::read_to_string(root.join(".doctrine/backlog/risk/001/backlog-001.toml")).unwrap();
        assert!(risk_toml.contains("[facet]"));
        let issue_toml =
            fs::read_to_string(root.join(".doctrine/backlog/issue/001/backlog-001.toml")).unwrap();
        assert!(!issue_toml.contains("[facet]"));

        // a second issue lands 002; the risk counter is untouched (separate dirs).
        let i2 = fresh(root, ItemKind::Issue, "login", "Login");
        assert_eq!(i2.eid.numeric_id(), Some(2));
        let r2 = fresh(root, ItemKind::Risk, "leak", "Leak");
        assert_eq!(r2.eid.numeric_id(), Some(2));

        // the materialised toml round-trips through validate end-to-end.
        let item = validate(toml::from_str::<RawBacklogToml>(&risk_toml).unwrap()).unwrap();
        assert_eq!(item.kind, ItemKind::Risk);
        assert_eq!(item.id, 1);
    }
}
