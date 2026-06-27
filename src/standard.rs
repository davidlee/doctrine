// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine standard` — standing conventions of practice. A thin per-kind module
//! over the shared `governance` spine (SL-030): this owns only the
//! *standard-specific* parts — the `GovKind` descriptor, the clap status enum +
//! known-set, the hide-set, and the scaffold/render. All kind-agnostic CLI/status
//! machinery lives in `crate::governance`, parameterized by `STANDARD_KIND`.
//!
//! A standard is a numeric directory under `.doctrine/standard/` holding a sister
//! `standard-NNN.toml` (structured, queried metadata) and a scaffolded
//! `standard-NNN.md` prose body, with an `NNN-slug` symlink alias — the policy
//! shape exactly (the third governance kind, SL-033), riding `entity::Kind` over
//! the kind-blind engine. Like a policy a standard records a *standing rule*, not a
//! decision; unlike a policy its vocab carries `default` — a recommended-unless-
//! justified convention — alongside `draft/required/deprecated/retired`. The
//! in-force set is `default` + `required` (boot PHASE-02 projects both).
//! Supersession is a relationship, not a status (design D2).

use std::io::{self, Write};
use std::path::PathBuf;

use crate::entity::{self, Artifact, Fileset, Kind, ScaffoldCtx};
use crate::governance::{self, GovKind};
use crate::listing::{Format, ListArgs};
use crate::tomlfmt::toml_string;

/// Relative dir of the standard tree inside the project root. Distinct top-level
/// tree (project-global governance), mirroring `.doctrine/policy`.
const STANDARD_DIR: &str = ".doctrine/standard";

/// The standard governance descriptor the spine binds. `prefix` is the
/// canonical-id stem (`STD-007`); `stem` is the file/JSON stem (`"standard"`) —
/// here `stem == prefix.to_lowercase()`, but POL proved the fields independent so
/// the explicit field carries no risk. `pub(crate)` so `boot` projects standard
/// rows via `governance::list_rows(&standard::STANDARD_KIND, …)` (SL-033 PHASE-02).
pub(crate) const STANDARD_KIND: GovKind = GovKind {
    kind: Kind {
        dir: STANDARD_DIR,
        prefix: crate::kinds::STD,
        stem: "standard",
        scaffold: standard_scaffold,
    },
    statuses: STANDARD_STATUSES,
    hidden: is_hidden,
};

/// The status transitions `standard status` writes. A standing convention's life:
/// `draft → default / required → deprecated / retired`. `default` (recommended
/// unless justified) and `required` (mandated) are the in-force states (the boot
/// section projects both, SL-033 PHASE-02). `superseded` is a terminal state
/// set ONLY by `doctrine supersede` (SL-095 PHASE-03), NOT an authoring-surface
/// status. A flat enum, no per-state stamping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum StandardStatus {
    Draft,
    Default,
    Required,
    Superseded,
    Deprecated,
    Retired,
}

impl StandardStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Default => "default",
            Self::Required => "required",
            Self::Superseded => "superseded",
            Self::Deprecated => "deprecated",
            Self::Retired => "retired",
        }
    }
}

/// The standard status known-set — the authority `governance::list_rows` checks
/// `--status` against. Mirrors `StandardStatus`'s variants, kept in lockstep by
/// `standard_known_set_matches_variants` (a drift canary). The enum kinds cannot
/// store an out-of-vocab status, so this doubles as the complete vocabulary.
pub(crate) const STANDARD_STATUSES: &[&str] = &[
    "draft",
    "default",
    "required",
    "superseded",
    "deprecated",
    "retired",
];

/// The `standard list` hide-set (design §5.3): `deprecated` (sunsetting but extant)
/// and `retired` (terminal off) standards no longer govern, so they drop from the
/// default list. The override (`--all` or any explicit `--status`) reveals them —
/// handled in `listing::retain`, not here. Bound as `STANDARD_KIND.hidden`.
fn is_hidden(status: &str) -> bool {
    matches!(status, "superseded" | "deprecated" | "retired")
}

// ---------------------------------------------------------------------------
// Pure: render, scaffold (the standard-specific templates — per-kind data)
// ---------------------------------------------------------------------------

/// Render `standard-<id>.toml` from the embedded template by token substitution.
/// The `id/slug/title/status` keys round-trip into `meta::Meta` (VT-1).
fn render_standard_toml(id: u32, slug: &str, title: &str, date: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/standard.toml")?
        .replace("{{id}}", &id.to_string())
        .replace("{{slug}}", &toml_string(slug))
        .replace("{{title}}", &toml_string(title))
        .replace("{{date}}", date))
}

/// Render `standard-<id>.md` from the embedded template: `{{ref}}` (the canonical
/// id, e.g. `STD-007`) + `{{title}}`. No YAML frontmatter (D1) — metadata lives
/// in the sister toml, not the prose.
fn render_standard_md(canonical_id: &str, title: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/standard.md")?
        .replace("{{ref}}", canonical_id)
        .replace("{{title}}", title))
}

/// The standard fileset: sister TOML, prose body, and `<id>-<slug>` symlink, all
/// relative to the standard tree root. Structurally `policy_scaffold`. Bound as
/// `STANDARD_KIND.kind.scaffold`.
fn standard_scaffold(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
    let id = ctx.id;
    let name = format!("{id:03}");
    Ok(vec![
        Artifact::File {
            rel_path: entity::rel_path(&STANDARD_KIND.kind, id, entity::Ext::Toml),
            body: render_standard_toml(id, ctx.slug, ctx.title, ctx.date)?,
        },
        Artifact::File {
            rel_path: entity::rel_path(&STANDARD_KIND.kind, id, entity::Ext::Md),
            body: render_standard_md(ctx.canonical, ctx.title)?,
        },
        Artifact::Symlink {
            rel_path: PathBuf::from(format!("{name}-{}", ctx.slug)),
            target: name,
        },
    ])
}

// ---------------------------------------------------------------------------
// CLI entry points — thin forwarders binding STANDARD_KIND into the spine
// ---------------------------------------------------------------------------

/// `doctrine standard new` → `governance::run_new(&STANDARD_KIND, …)`.
pub(crate) fn run_new(
    path: Option<PathBuf>,
    title: Option<String>,
    slug: Option<String>,
) -> anyhow::Result<()> {
    governance::run_new(&STANDARD_KIND, path, title, slug)
}

/// `doctrine standard list` → `governance::run_list(&STANDARD_KIND, …)`.
pub(crate) fn run_list(path: Option<PathBuf>, args: ListArgs) -> anyhow::Result<()> {
    governance::run_list(&STANDARD_KIND, path, args)
}

/// `doctrine standard show <STD-NNN>` → `governance::run_show(&STANDARD_KIND, …)`.
pub(crate) fn run_show(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
) -> anyhow::Result<()> {
    governance::run_show(&STANDARD_KIND, path, reference, format)
}

/// Parse a standard reference — accepts both `STD-007` and bare `7`.
pub(crate) fn parse_ref(reference: &str) -> anyhow::Result<u32> {
    let digits = reference
        .strip_prefix("STD-")
        .or_else(|| reference.strip_prefix("std-"))
        .unwrap_or(reference);
    digits.parse::<u32>().with_context(|| {
        format!("not a standard reference: `{reference}` (expected `STD-007` or `7`)")
    })
}

/// Clap `value_parser` wrapper for [`parse_ref`].
fn parse_cli_id(s: &str) -> Result<u32, String> {
    parse_ref(s).map_err(|e| format!("{e:#}"))
}

/// `doctrine standard status` — bind the concrete `StandardStatus` enum at the
/// boundary, delegate the edit-preserving transition to the spine, then print.
/// The clock is read here and passed in (the pure/imperative split).
pub(crate) fn run_status(
    path: Option<PathBuf>,
    id: u32,
    status: StandardStatus,
    color: bool,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    governance::set_status(
        &STANDARD_KIND,
        &root,
        id,
        status.as_str(),
        &crate::clock::today(),
    )?;
    writeln!(
        io::stdout(),
        "STD {id:03}: {}",
        crate::listing::status_colored(status.as_str(), color)
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests — the standard-specific data (render, scaffold, known-set). The
// shared-spine behaviour tests (list/show/status/parse) live in `governance.rs`,
// driven by `ADR_KIND` (SL-030 PHASE-02); they parameterize identically over
// `STANDARD_KIND`, so they are not re-run here.
// ---------------------------------------------------------------------------

// ── CLI dispatch ───────────────────────────────────────────────────────────

use std::str::FromStr;

use crate::CommonListArgs;
use anyhow::Context;
use clap::Subcommand;

#[derive(Subcommand)]
pub(crate) enum StandardCommand {
    /// Allocate the next id and scaffold a new standard.
    New {
        title: Option<String>,
        #[arg(long)]
        slug: Option<String>,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
    /// List standards.
    List {
        #[command(flatten)]
        list: CommonListArgs,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
    /// Show one standard.
    Show {
        reference: String,
        #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
        format: Format,
        #[arg(long)]
        json: bool,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
    /// Set a standard's status.
    Status {
        #[arg(value_parser = parse_cli_id)]
        id: u32,
        #[arg(long)]
        status: StandardStatus,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Print the file paths of each standard entity directory.
    Paths {
        /// Standard reference(s) — `STD-007` or the bare id `7`.
        refs: Vec<String>,
        #[arg(short = 't', long)]
        toml: bool,
        #[arg(short = 'm', long)]
        md: bool,
        #[arg(short = 'e', long)]
        entity: bool,
        #[arg(short = 's', long)]
        single: bool,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

pub(crate) fn dispatch(cmd: StandardCommand, color: bool) -> anyhow::Result<()> {
    match cmd {
        StandardCommand::New { title, slug, path } => run_new(path, title, slug),
        StandardCommand::List { list, path } => run_list(path, list.into_list_args(color)),
        StandardCommand::Show {
            reference,
            format,
            json,
            path,
        } => run_show(path, &reference, if json { Format::Json } else { format }),
        StandardCommand::Status { id, status, path } => run_status(path, id, status, color),
        StandardCommand::Paths {
            refs,
            toml,
            md,
            entity,
            single,
            path,
        } => governance::run_paths(
            &STANDARD_KIND,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta::Meta;
    use std::path::Path;

    // --- VT-1: render + round-trip ---

    #[test]
    fn render_standard_toml_round_trips_to_metadata() {
        let body =
            render_standard_toml(7, "two-space-indent", "Two-space indent", "2026-06-04").unwrap();
        // The four list fields parse into meta::Meta …
        let parsed: Meta = toml::from_str(&body).unwrap();
        assert_eq!(
            parsed,
            Meta {
                id: 7,
                slug: "two-space-indent".to_string(),
                title: "Two-space indent".to_string(),
                status: "draft".to_string(),
                tags: vec![],
            }
        );
        // status seeds draft, the date is injected, no token survives.
        assert!(body.contains("created = \"2026-06-04\""));
        assert!(!body.contains("{{"));
    }

    #[test]
    fn render_standard_toml_escapes_hostile_title_and_slug() {
        // A title / explicit slug carrying the quoted-literal breakers (`"`, `\`,
        // newline) must still render a parseable toml that round-trips.
        let title = crate::tomlfmt::HOSTILE_TITLE;
        let slug = crate::tomlfmt::HOSTILE_SLUG;
        let body = render_standard_toml(7, slug, title, "2026-06-04").unwrap();
        let parsed: Meta = toml::from_str(&body).unwrap();
        assert_eq!(parsed.slug, slug);
        assert_eq!(parsed.title, title);
    }

    #[test]
    fn render_standard_toml_relationships_are_preserved_and_ignored_by_meta() {
        let body = render_standard_toml(1, "s", "T", "2026-06-04").unwrap();
        // The [relationships] table parses as a whole document …
        let doc: toml::Value = toml::from_str(&body).unwrap();
        // SL-095: `supersedes` is no longer a typed field; it's now a `[[relation]]` row.
        for axis in ["superseded_by"] {
            assert!(
                doc["relationships"][axis].as_array().unwrap().is_empty(),
                "{axis} should seed empty"
            );
        }
        // … yet Meta deserialises fine, ignoring the unknown table.
        assert!(toml::from_str::<Meta>(&body).is_ok());
    }

    #[test]
    fn render_standard_md_substitutes_ref_and_title_without_frontmatter() {
        let body = render_standard_md("STD-007", "Two-space indent").unwrap();
        assert!(body.starts_with("# STD-007: Two-space indent"));
        assert!(!body.contains("{{ref}}"));
        assert!(!body.contains("{{title}}"));
        // no YAML frontmatter (D1 — metadata is in the toml, not the prose).
        assert!(!body.starts_with("---"));
        assert!(!body.contains("\n---\n"));
        // the supekku `default` = recommended-unless-justified note is carried.
        assert!(body.contains("status \"default\""));
    }

    // --- VT-1: scaffold shape ---

    #[test]
    fn standard_scaffold_lays_out_two_files_and_a_symlink() {
        let ctx = ScaffoldCtx {
            id: 7,
            canonical: "STD-007",
            slug: "two-space-indent",
            title: "Two-space indent",
            date: "2026-06-04",
        };
        let fileset = standard_scaffold(&ctx).unwrap();
        assert_eq!(fileset.len(), 3);
        // filenames derive from the "standard" stem, ids from the "STD" prefix.
        assert!(matches!(&fileset[0],
            Artifact::File { rel_path, body }
            if rel_path == Path::new("007/standard-007.toml") && body.contains("2026-06-04")));
        assert!(matches!(&fileset[1],
            Artifact::File { rel_path, body }
            if rel_path == Path::new("007/standard-007.md") && body.contains("STD-007: Two-space indent")));
        assert!(matches!(&fileset[2],
            Artifact::Symlink { rel_path, target }
            if rel_path == Path::new("007-two-space-indent") && target == "007"));
    }

    /// Drift canary: the `STANDARD_STATUSES` known-set must stay in lockstep with
    /// the `StandardStatus` variants (the enum kinds cannot store an out-of-vocab
    /// value, so this is the complete vocabulary). EX-2 / VT-2.
    #[test]
    fn standard_known_set_matches_variants() {
        let variants = [
            StandardStatus::Draft,
            StandardStatus::Default,
            StandardStatus::Required,
            StandardStatus::Superseded,
            StandardStatus::Deprecated,
            StandardStatus::Retired,
        ];
        let from_variants: Vec<&str> = variants.iter().map(|v| v.as_str()).collect();
        assert_eq!(from_variants, STANDARD_STATUSES.to_vec());
    }

    /// The hide-set must only name statuses in the known-set (design §5.5
    /// invariant: hide-set ⊆ known-set), and the in-force pair (default/required)
    /// plus draft stay visible. EX-2 / VT-2.
    #[test]
    fn standard_hide_set_is_a_subset_of_the_known_set() {
        for s in STANDARD_STATUSES {
            // every hidden status is a known status — vacuously holds, but the
            // converse guard: a status flagged hidden must be in the vocab.
            let _ = is_hidden(s);
        }
        assert!(is_hidden("superseded"));
        assert!(is_hidden("deprecated"));
        assert!(is_hidden("retired"));
        assert!(!is_hidden("draft"));
        assert!(!is_hidden("default"));
        assert!(!is_hidden("required"));
    }

    // --- an empty / symbol-only title bails for an explicit --slug ---

    #[test]
    fn run_new_bails_for_a_slug_on_a_symbol_only_title() {
        let dir = tempfile::tempdir().unwrap();
        let err = run_new(Some(dir.path().to_path_buf()), Some("!!!".into()), None).unwrap_err();
        assert!(err.to_string().contains("pass --slug"));
    }

    // --- parse_ref ---

    #[test]
    fn parse_ref_accepts_prefixed_padded_and_bare_ids() {
        assert_eq!(parse_ref("STD-007").unwrap(), 7);
        assert_eq!(parse_ref("std-7").unwrap(), 7);
        assert_eq!(parse_ref("7").unwrap(), 7);
        assert_eq!(parse_ref("007").unwrap(), 7);
        let err = parse_ref("nope").unwrap_err().to_string();
        assert!(err.contains("standard"), "error should name standard kind: {err}");
    }
}
