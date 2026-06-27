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

use std::str::FromStr;

use anyhow::Context;

use clap::Subcommand;

// ---------------------------------------------------------------------------
// CLI enum & dispatch (PHASE-03 relocation from main.rs)
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
pub(crate) enum AdrCommand {
    /// Allocate the next id and scaffold a new ADR.
    New {
        /// ADR title (prompted for if omitted).
        title: Option<String>,

        /// Explicit slug (default: derived from the title).
        #[arg(long)]
        slug: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// List ADRs by id: ADR-id, status, slug, title.
    List {
        #[command(flatten)]
        list: crate::CommonListArgs,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Show one ADR: its metadata, relationships, and prose body.
    Show {
        /// ADR reference — `ADR-007` or the bare id `7`.
        reference: String,

        /// Output format.
        #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
        format: Format,

        /// Shorthand for `--format json`.
        #[arg(long)]
        json: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Set an ADR's status (edit-preserving; a no-op if unchanged).
    Status {
        /// ADR id (numeric).
        #[arg(value_parser = parse_cli_id)]
        id: u32,

        /// New status (required): proposed|accepted|rejected|superseded|deprecated.
        #[arg(long)]
        status: AdrStatus,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Print the file paths of each ADR entity directory.
    Paths {
        /// ADR reference(s) — `ADR-007` or the bare id `7`.
        refs: Vec<String>,

        /// Show only the identity TOML file.
        #[arg(short = 't', long)]
        toml: bool,
        /// Show only the identity Markdown body.
        #[arg(short = 'm', long)]
        md: bool,
        /// Show the identity TOML + Markdown (equivalent to -t -m).
        #[arg(short = 'e', long)]
        entity: bool,
        /// Return only the first (primary) path per ref.
        #[arg(short = 's', long)]
        single: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

pub(crate) fn dispatch(cmd: AdrCommand, color: bool) -> anyhow::Result<()> {
    match cmd {
        AdrCommand::New { title, slug, path } => run_new(path, title, slug),
        AdrCommand::List { list, path } => run_list(path, list.into_list_args(color)),
        AdrCommand::Show {
            reference,
            format,
            json,
            path,
        } => run_show(path, &reference, if json { Format::Json } else { format }),
        AdrCommand::Status { id, status, path } => run_status(path, id, status, color),
        AdrCommand::Paths {
            refs,
            toml,
            md,
            entity,
            single,
            path,
        } => governance::run_paths(
            &ADR_KIND,
            path,
            &refs,
            &crate::paths::PathSelection {
                toml,
                md,
                entity,
                single,
            },
        ),
    }
}

// ---------------------------------------------------------------------------

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
        prefix: crate::kinds::ADR,
        stem: "adr",
        scaffold: adr_scaffold,
    },
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

/// Parse an ADR reference — accepts both `ADR-007` and bare `7`.
pub(crate) fn parse_ref(reference: &str) -> anyhow::Result<u32> {
    let digits = reference
        .strip_prefix("ADR-")
        .or_else(|| reference.strip_prefix("adr-"))
        .unwrap_or(reference);
    digits
        .parse::<u32>()
        .with_context(|| format!("not an ADR reference: `{reference}` (expected `ADR-007` or `7`)"))
}

/// Clap `value_parser` wrapper for [`parse_ref`].
fn parse_cli_id(s: &str) -> Result<u32, String> {
    parse_ref(s).map_err(|e| format!("{e:#}"))
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
    governance::set_status(
        &ADR_KIND,
        &root,
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
                tags: vec![],
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
        assert!(doc["tags"].as_array().unwrap().is_empty());
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

    // --- parse_ref ---

    #[test]
    fn parse_ref_accepts_prefixed_padded_and_bare_ids() {
        assert_eq!(parse_ref("ADR-007").unwrap(), 7);
        assert_eq!(parse_ref("adr-7").unwrap(), 7);
        assert_eq!(parse_ref("7").unwrap(), 7);
        assert_eq!(parse_ref("007").unwrap(), 7);
        let err = parse_ref("nope").unwrap_err().to_string();
        assert!(err.contains("ADR"), "error should name ADR kind: {err}");
    }
}
