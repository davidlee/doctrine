// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine adr` — architecture decision records, doctrine's first unit of
//! governance. A thin per-kind module over the shared `governance` spine
//! (SL-030 PHASE-02): this owns only the *ADR-specific* parts — the `GovKind`
//! descriptor, the clap status enum + known-set, the hide-set, and the
//! scaffold/render. All kind-agnostic CLI/status machinery lives in
//! `crate::governance`, parameterized by `ADR_KIND`.
//!
//! An ADR is a numeric directory under `.doctrine/adr/` holding a sister
//! `adr-NNN.toml` (structured, queried metadata) and a scaffolded `adr-NNN.md`
//! prose body, with an `NNN-slug` symlink alias — the slice shape exactly (design
//! SL-006 D1/D2), riding `entity::Kind` over the kind-blind engine.

use std::io::{self, Write};
use std::path::PathBuf;

use crate::entity::{Artifact, Fileset, Kind, ScaffoldCtx};
use crate::governance::{self, GovKind};
use crate::listing::{Format, ListArgs};
use crate::tomlfmt::toml_string;

/// Relative dir of the ADR tree inside the project root. Distinct top-level tree,
/// not nested under slice (D2 — ADRs are project-global governance).
const ADR_DIR: &str = ".doctrine/adr";

/// The ADR governance descriptor the spine binds. `prefix` is the canonical-id
/// stem (`ADR-007`); `stem` is the file/JSON stem (`"adr"`) — see `meta` on why
/// prefix ≠ stem. `pub(crate)` so `boot` projects ADR rows via
/// `governance::list_rows(&adr::ADR_KIND, …)`.
pub(crate) const ADR_KIND: GovKind = GovKind {
    kind: Kind {
        dir: ADR_DIR,
        prefix: "ADR",
        scaffold: adr_scaffold,
    },
    stem: "adr",
    statuses: ADR_STATUSES,
    hidden: is_hidden,
};

/// The status transitions `adr status` writes. Distinct from the `proposed`
/// scaffold seed: these are the moves an ADR makes over its life. A flat enum —
/// no lifecycle ladder (unlike `state::PhaseStatus`), so no per-state stamping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum AdrStatus {
    Proposed,
    Accepted,
    Rejected,
    Superseded,
    Deprecated,
}

impl AdrStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Superseded => "superseded",
            Self::Deprecated => "deprecated",
        }
    }
}

/// The ADR status known-set — the authority `governance::list_rows` checks
/// `--status` against (A-2). Mirrors `AdrStatus`'s variants, kept in lockstep by
/// `adr_known_set_matches_variants` (a drift canary). The enum kinds cannot store
/// an out-of-vocab status, so this doubles as the complete vocabulary.
pub(crate) const ADR_STATUSES: &[&str] = &[
    "proposed",
    "accepted",
    "rejected",
    "superseded",
    "deprecated",
];

/// The `adr list` hide-set (design §5.3): superseded / rejected / deprecated ADRs
/// are decisions that no longer govern, so they drop from the default list. The
/// override (`--all` or any explicit `--status`) reveals them — handled in
/// `listing::retain`, not here. Bound as `ADR_KIND.hidden`.
fn is_hidden(status: &str) -> bool {
    matches!(status, "rejected" | "superseded" | "deprecated")
}

// ---------------------------------------------------------------------------
// Supersession capability boundary (SL-062 PHASE-03)
// ---------------------------------------------------------------------------

/// The per-kind supersession field/status names the `supersede` verb composes its
/// transaction over. Owned here because every field is ADR-specific governance
/// vocabulary — the outbound `supersedes` edge (ADR-004 legit), the single
/// sanctioned reverse carve-out `superseded_by` (written ONLY by `supersede`), and
/// the terminal `superseded` status this kind flips OLD into.
pub(crate) struct SupersedePolicy {
    /// NEW's outbound edge array — `[relationships].supersedes` (ADR-004 §5).
    pub(crate) supersedes_field: &'static str,
    /// OLD's reverse carve-out array — `[relationships].superseded_by`.
    pub(crate) carveout_field: &'static str,
    /// The terminal status OLD is flipped into.
    pub(crate) superseded_status: &'static str,
}

/// The supersession capability boundary (SL-062 PHASE-03 / EX-1, D4): supersession
/// is supported for **ADR only** today. A hardcoded kind MATCH on the canonical
/// prefix, NOT a `GovKind` data field — POL/STD/slice (and every other kind) return
/// `None`, and the verb refuses them with the ADR-first message. A later phase that
/// widens supersession to POL/STD adds the arms here.
pub(crate) fn supersede_policy(kind: &Kind) -> Option<SupersedePolicy> {
    match kind.prefix {
        "ADR" => Some(SupersedePolicy {
            supersedes_field: "supersedes",
            carveout_field: "superseded_by",
            superseded_status: "superseded",
        }),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Pure: render, scaffold (the ADR-specific templates — per-kind data)
// ---------------------------------------------------------------------------

/// Render `adr-<id>.toml` from the embedded template by token substitution. The
/// `id/slug/title/status` keys round-trip into `meta::Meta` (VT-3).
fn render_adr_toml(id: u32, slug: &str, title: &str, date: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/adr.toml")?
        .replace("{{id}}", &id.to_string())
        .replace("{{slug}}", &toml_string(slug))
        .replace("{{title}}", &toml_string(title))
        .replace("{{date}}", date))
}

/// Render `adr-<id>.md` from the embedded template: `{{ref}}` (the canonical id,
/// e.g. `ADR-007`) + `{{title}}`. No YAML frontmatter (D1) — metadata lives in
/// the sister toml, not the prose.
fn render_adr_md(canonical_id: &str, title: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/adr.md")?
        .replace("{{ref}}", canonical_id)
        .replace("{{title}}", title))
}

/// The ADR fileset: sister TOML, prose body, and `<id>-<slug>` symlink, all
/// relative to the ADR tree root — structurally `slice_scaffold` (D2). Bound as
/// `ADR_KIND.kind.scaffold`.
fn adr_scaffold(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
    let id = ctx.id;
    let name = format!("{id:03}");
    Ok(vec![
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/adr-{name}.toml")),
            body: render_adr_toml(id, ctx.slug, ctx.title, ctx.date)?,
        },
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/adr-{name}.md")),
            body: render_adr_md(ctx.canonical, ctx.title)?,
        },
        Artifact::Symlink {
            rel_path: PathBuf::from(format!("{name}-{}", ctx.slug)),
            target: name,
        },
    ])
}

// ---------------------------------------------------------------------------
// CLI entry points — thin forwarders binding ADR_KIND into the spine
// ---------------------------------------------------------------------------

/// `doctrine adr new` → `governance::run_new(&ADR_KIND, …)`.
pub(crate) fn run_new(
    path: Option<PathBuf>,
    title: Option<String>,
    slug: Option<String>,
) -> anyhow::Result<()> {
    governance::run_new(&ADR_KIND, path, title, slug)
}

/// `doctrine adr list` → `governance::run_list(&ADR_KIND, …)`.
pub(crate) fn run_list(path: Option<PathBuf>, args: ListArgs) -> anyhow::Result<()> {
    governance::run_list(&ADR_KIND, path, args)
}

/// `doctrine adr show <ADR-NNN>` → `governance::run_show(&ADR_KIND, …)`.
pub(crate) fn run_show(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
) -> anyhow::Result<()> {
    governance::run_show(&ADR_KIND, path, reference, format)
}

/// `doctrine adr status` — bind the concrete `AdrStatus` enum at the boundary,
/// delegate the edit-preserving transition to the spine, then print. The clock is
/// read here and passed in (the pure/imperative split).
pub(crate) fn run_status(
    path: Option<PathBuf>,
    id: u32,
    status: AdrStatus,
    color: bool,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let gov_root = root.join(ADR_KIND.kind.dir);
    governance::set_status(
        &ADR_KIND,
        &gov_root,
        id,
        status.as_str(),
        &crate::clock::today(),
    )?;
    writeln!(
        io::stdout(),
        "ADR {id:03}: {}",
        crate::listing::status_colored(status.as_str(), color)
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests — the ADR-specific data (render, scaffold, known-set). The shared-spine
// behaviour tests (list/show/status/parse) live in `governance.rs`, driven by
// `ADR_KIND` (SL-030 PHASE-02 VT-2).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta::Meta;
    use std::path::Path;

    // --- VT-1 / VT-3: render + round-trip ---

    #[test]
    fn render_adr_toml_round_trips_to_metadata() {
        let body = render_adr_toml(7, "use-rust", "Use Rust", "2026-06-04").unwrap();
        // VT-3: the four list fields parse into meta::Meta …
        let parsed: Meta = toml::from_str(&body).unwrap();
        assert_eq!(
            parsed,
            Meta {
                id: 7,
                slug: "use-rust".to_string(),
                title: "Use Rust".to_string(),
                status: "proposed".to_string(),
            }
        );
        // VT-1: status seeds proposed, the date is injected, no token survives.
        assert!(body.contains("created = \"2026-06-04\""));
        assert!(!body.contains("{{"));
    }

    #[test]
    fn render_adr_toml_escapes_hostile_title_and_slug() {
        // SL-024: a title / explicit slug carrying the quoted-literal breakers
        // (`"`, `\`, newline) must still render a parseable toml that round-trips.
        let title = crate::tomlfmt::HOSTILE_TITLE;
        let slug = crate::tomlfmt::HOSTILE_SLUG;
        let body = render_adr_toml(7, slug, title, "2026-06-04").unwrap();
        let parsed: Meta = toml::from_str(&body).unwrap();
        assert_eq!(parsed.slug, slug);
        assert_eq!(parsed.title, title);
    }

    #[test]
    fn render_adr_toml_relationships_are_preserved_and_ignored_by_meta() {
        let body = render_adr_toml(1, "s", "T", "2026-06-04").unwrap();
        // VT-3: the [relationships] table parses as a whole document …
        let doc: toml::Value = toml::from_str(&body).unwrap();
        // SL-095: `supersedes` is no longer a typed field; it's now a `[[relation]]` row.
        assert!(
            doc["relationships"]["superseded_by"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(doc["relationships"]["tags"].as_array().unwrap().is_empty());
        // … yet Meta deserialises fine, ignoring the unknown table.
        assert!(toml::from_str::<Meta>(&body).is_ok());
    }

    #[test]
    fn render_adr_md_substitutes_ref_and_title_without_frontmatter() {
        let body = render_adr_md("ADR-007", "Use Rust").unwrap();
        assert!(body.starts_with("# ADR-007: Use Rust"));
        assert!(!body.contains("{{ref}}"));
        assert!(!body.contains("{{title}}"));
        // VT-1: no YAML frontmatter (D1 — metadata is in the toml, not the prose).
        assert!(!body.starts_with("---"));
        assert!(!body.contains("\n---\n"));
    }

    // --- VT-2: scaffold shape ---

    #[test]
    fn adr_scaffold_lays_out_two_files_and_a_symlink() {
        let ctx = ScaffoldCtx {
            id: 7,
            canonical: "ADR-007",
            slug: "use-rust",
            title: "Use Rust",
            date: "2026-06-04",
        };
        let fileset = adr_scaffold(&ctx).unwrap();
        assert_eq!(fileset.len(), 3);
        assert!(matches!(&fileset[0],
            Artifact::File { rel_path, body }
            if rel_path == Path::new("007/adr-007.toml") && body.contains("2026-06-04")));
        assert!(matches!(&fileset[1],
            Artifact::File { rel_path, body }
            if rel_path == Path::new("007/adr-007.md") && body.contains("ADR-007: Use Rust")));
        assert!(matches!(&fileset[2],
            Artifact::Symlink { rel_path, target }
            if rel_path == Path::new("007-use-rust") && target == "007"));
    }

    /// Drift canary: the `ADR_STATUSES` known-set must stay in lockstep with the
    /// `AdrStatus` variants (the enum kinds cannot store an out-of-vocab value, so
    /// this is the complete vocabulary).
    #[test]
    fn adr_known_set_matches_variants() {
        let variants = [
            AdrStatus::Proposed,
            AdrStatus::Accepted,
            AdrStatus::Rejected,
            AdrStatus::Superseded,
            AdrStatus::Deprecated,
        ];
        let from_variants: Vec<&str> = variants.iter().map(|v| v.as_str()).collect();
        assert_eq!(from_variants, ADR_STATUSES.to_vec());
    }

    // --- VT-2: an empty / symbol-only title bails for an explicit --slug ---

    #[test]
    fn run_new_bails_for_a_slug_on_a_symbol_only_title() {
        let dir = tempfile::tempdir().unwrap();
        let err = run_new(Some(dir.path().to_path_buf()), Some("!!!".into()), None).unwrap_err();
        assert!(err.to_string().contains("pass --slug"));
    }
}
