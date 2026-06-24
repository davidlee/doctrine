// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine rfc` — request-for-comment discussion artifacts, doctrine's
//! governance-neutral deliberation kind. A thin per-kind module over the shared
//! `governance` spine: this owns only the *RFC-specific* parts — the `GovKind`
//! descriptor, the clap status enum + known-set, the hide-set, and the
//! scaffold/render. All kind-agnostic CLI/status machinery lives in
//! `crate::governance`, parameterized by `RFC_KIND`.
//!
//! An RFC is a numeric directory under `.doctrine/rfc/` holding a sister
//! `rfc-NNN.toml` (structured, queried metadata) and a scaffolded `rfc-NNN.md`
//! prose body — the ADR shape exactly, riding `entity::Kind` over the
//! kind-blind engine. Unlike ADR, an RFC asserts NO governance position: it
//! is deliberation-only, structurally absent from governance surfaces.

use std::io::{self, Write};
use std::path::PathBuf;

use crate::entity::{self, Artifact, Fileset, Kind, ScaffoldCtx};
use crate::governance::{self, GovKind};
use crate::listing::{Format, ListArgs};
use crate::tomlfmt::toml_string;

/// Relative dir of the RFC tree inside the project root. Distinct top-level
/// tree — singular per design §3 (.doctrine/rfc).
const RFC_DIR: &str = ".doctrine/rfc";

/// The RFC governance descriptor the spine binds. `prefix` is the canonical-id
/// stem (`RFC-007`); `stem` is the file/JSON stem (`"rfc"`). `pub(crate)` so
/// `boot` etc. can access it.
pub(crate) const RFC_KIND: GovKind = GovKind {
    kind: Kind {
        dir: RFC_DIR,
        prefix: crate::kinds::RFC,
        stem: "rfc",
        scaffold: rfc_scaffold,
    },
    statuses: RFC_STATUSES,
    hidden: is_hidden,
};

/// The status transitions `rfc status` writes. A minimal, governance-neutral
/// machine (design §2): `open → resolved | withdrawn`. `open` is the scaffold
/// seed; `resolved` is outcome-blind ("concluded", not "concluded yes").
/// A flat enum — no per-state stamping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum RfcStatus {
    Open,
    Resolved,
    Withdrawn,
}

impl RfcStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Resolved => "resolved",
            Self::Withdrawn => "withdrawn",
        }
    }
}

/// The RFC status known-set — the authority `governance::list_rows` checks
/// `--status` against. Mirrors `RfcStatus`'s variants, kept in lockstep by
/// `rfc_known_set_matches_variants` (a drift canary).
pub(crate) const RFC_STATUSES: &[&str] = &["open", "resolved", "withdrawn"];

/// The `rfc list` hide-set: `resolved` and `withdrawn` are terminal and drop
/// from the default list (live-set-by-default idiom, design §2 F4). The
/// override (`--all` or any explicit `--status`) reveals them — handled in
/// `listing::retain`, not here. Bound as `RFC_KIND.hidden`.
fn is_hidden(status: &str) -> bool {
    matches!(status, "resolved" | "withdrawn")
}

// ---------------------------------------------------------------------------
// Pure: render, scaffold (the RFC-specific templates — per-kind data)
// ---------------------------------------------------------------------------

/// Render `rfc-<id>.toml` from the embedded template by token substitution.
/// The `id/slug/title/status` keys round-trip into `meta::Meta` (VT-1).
fn render_rfc_toml(id: u32, slug: &str, title: &str, date: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/rfc.toml")?
        .replace("{{id}}", &id.to_string())
        .replace("{{slug}}", &toml_string(slug))
        .replace("{{title}}", &toml_string(title))
        .replace("{{date}}", date))
}

/// Render `rfc-<id>.md` from the embedded template: `{{ref}}` (the canonical
/// id, e.g. `RFC-007`) + `{{title}}`. No YAML frontmatter (D1) — metadata lives
/// in the sister toml, not the prose.
fn render_rfc_md(canonical_id: &str, title: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/rfc.md")?
        .replace("{{ref}}", canonical_id)
        .replace("{{title}}", title))
}

/// The RFC fileset: sister TOML, prose body — no symlink (RFCs are simpler than
/// governance kinds; design §3 omits a slug symlink as unnecessary). Bound as
/// `RFC_KIND.kind.scaffold`.
fn rfc_scaffold(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
    let id = ctx.id;
    Ok(vec![
        Artifact::File {
            rel_path: entity::rel_path(&RFC_KIND.kind, id, entity::Ext::Toml),
            body: render_rfc_toml(id, ctx.slug, ctx.title, ctx.date)?,
        },
        Artifact::File {
            rel_path: entity::rel_path(&RFC_KIND.kind, id, entity::Ext::Md),
            body: render_rfc_md(ctx.canonical, ctx.title)?,
        },
    ])
}

// ---------------------------------------------------------------------------
// CLI entry points — thin forwarders binding RFC_KIND into the spine
// ---------------------------------------------------------------------------

/// `doctrine rfc new` → `governance::run_new(&RFC_KIND, …)`.
pub(crate) fn run_new(
    path: Option<PathBuf>,
    title: Option<String>,
    slug: Option<String>,
) -> anyhow::Result<()> {
    governance::run_new(&RFC_KIND, path, title, slug)
}

/// `doctrine rfc list` → `governance::run_list(&RFC_KIND, …)`.
pub(crate) fn run_list(path: Option<PathBuf>, args: ListArgs) -> anyhow::Result<()> {
    governance::run_list(&RFC_KIND, path, args)
}

/// `doctrine rfc show <RFC-NNN>` → `governance::run_show(&RFC_KIND, …)`.
pub(crate) fn run_show(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
) -> anyhow::Result<()> {
    governance::run_show(&RFC_KIND, path, reference, format)
}

/// `doctrine rfc status` — bind the concrete `RfcStatus` enum at the boundary,
/// delegate the edit-preserving transition to the spine, then print.
pub(crate) fn run_status(
    path: Option<PathBuf>,
    id: u32,
    status: RfcStatus,
    color: bool,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    governance::set_status(
        &RFC_KIND,
        &root,
        id,
        status.as_str(),
        &crate::clock::today(),
    )?;
    writeln!(
        io::stdout(),
        "RFC {id:03}: {}",
        crate::listing::status_colored(status.as_str(), color)
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests — the RFC-specific data (render, scaffold, known-set). The shared-spine
// behaviour tests (list/show/status/parse) live in `governance.rs`.
// ---------------------------------------------------------------------------

// ── CLI dispatch ───────────────────────────────────────────────────────────

use std::str::FromStr;

use crate::CommonListArgs;
use clap::Subcommand;

#[derive(Subcommand)]
pub(crate) enum RfcCommand {
    /// Allocate the next id and scaffold a new RFC.
    New {
        title: Option<String>,
        #[arg(long)]
        slug: Option<String>,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
    /// List RFCs.
    List {
        #[command(flatten)]
        list: CommonListArgs,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
    /// Show one RFC.
    Show {
        reference: String,
        #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
        format: Format,
        #[arg(long)]
        json: bool,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
    /// Set an RFC's status.
    Status {
        id: u32,
        #[arg(long)]
        status: RfcStatus,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Print the file paths of each RFC entity directory.
    Paths {
        /// RFC reference(s) — `RFC-007` or the bare id `7`.
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

pub(crate) fn dispatch(cmd: RfcCommand, color: bool) -> anyhow::Result<()> {
    match cmd {
        RfcCommand::New { title, slug, path } => run_new(path, title, slug),
        RfcCommand::List { list, path } => run_list(path, list.into_list_args(color)),
        RfcCommand::Show {
            reference,
            format,
            json,
            path,
        } => run_show(path, &reference, if json { Format::Json } else { format }),
        RfcCommand::Status { id, status, path } => run_status(path, id, status, color),
        RfcCommand::Paths {
            refs,
            toml,
            md,
            entity,
            single,
            path,
        } => governance::run_paths(
            &RFC_KIND,
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
    fn render_rfc_toml_round_trips_to_metadata() {
        let body = render_rfc_toml(1, "use-rust", "Use Rust?", "2026-06-04").unwrap();
        let parsed: Meta = toml::from_str(&body).unwrap();
        assert_eq!(
            parsed,
            Meta {
                id: 1,
                slug: "use-rust".to_string(),
                title: "Use Rust?".to_string(),
                status: "open".to_string(),
                tags: vec![],
            }
        );
        // VT-1: status seeds open, the date is injected, no token survives.
        assert!(body.contains("created = \"2026-06-04\""));
        assert!(!body.contains("{{"));
    }

    #[test]
    fn render_rfc_toml_escapes_hostile_title_and_slug() {
        let title = crate::tomlfmt::HOSTILE_TITLE;
        let slug = crate::tomlfmt::HOSTILE_SLUG;
        let body = render_rfc_toml(1, slug, title, "2026-06-04").unwrap();
        let parsed: Meta = toml::from_str(&body).unwrap();
        assert_eq!(parsed.slug, slug);
        assert_eq!(parsed.title, title);
    }

    #[test]
    fn render_rfc_md_substitutes_ref_and_title_without_frontmatter() {
        let body = render_rfc_md("RFC-001", "Use Rust?").unwrap();
        assert!(body.starts_with("# RFC-001: Use Rust?"));
        assert!(!body.contains("{{ref}}"));
        assert!(!body.contains("{{title}}"));
        assert!(!body.starts_with("---"));
        assert!(!body.contains("\n---\n"));
    }

    // --- VT-2: scaffold shape ---

    #[test]
    fn rfc_scaffold_lays_out_two_files() {
        let ctx = ScaffoldCtx {
            id: 1,
            canonical: "RFC-001",
            slug: "use-rust",
            title: "Use Rust?",
            date: "2026-06-04",
        };
        let fileset = rfc_scaffold(&ctx).unwrap();
        assert_eq!(fileset.len(), 2);
        assert!(matches!(&fileset[0],
            Artifact::File { rel_path, body }
            if rel_path == Path::new("001/rfc-001.toml") && body.contains("2026-06-04")));
        assert!(matches!(&fileset[1],
            Artifact::File { rel_path, body }
            if rel_path == Path::new("001/rfc-001.md") && body.contains("RFC-001: Use Rust?")));
    }

    /// Drift canary: the `RFC_STATUSES` known-set must stay in lockstep with the
    /// `RfcStatus` variants. Also proves EX-1: the defined known-status set is
    /// exactly {open, resolved, withdrawn} with NO "accepted".
    #[test]
    fn rfc_known_set_matches_variants() {
        let variants = [RfcStatus::Open, RfcStatus::Resolved, RfcStatus::Withdrawn];
        let from_variants: Vec<&str> = variants.iter().map(|v| v.as_str()).collect();
        assert_eq!(from_variants, RFC_STATUSES.to_vec());
        // EX-1: the set must NOT contain "accepted" — RFC is outcome-blind (design §2).
        assert!(!RFC_STATUSES.contains(&"accepted"));
    }

    // --- helpers -------------------------------------------------------------

    /// Write an RFC's authored toml directly at an explicit id (creating its dir).
    /// Bypasses the monotonic `Fresh` allocator so fixtures don't need a git
    /// trunk ref — the `adr_at` precedent. Only writes the fields `meta::read_meta`
    /// and `set_status` consume.
    fn rfc_at(root: &Path, id: u32, status: &str, slug: &str, title: &str) {
        let name = format!("{id:03}");
        let dir = root.join(RFC_DIR).join(&name);
        std::fs::create_dir_all(&dir).unwrap();
        let toml = format!(
            "schema = \"doctrine.rfc\"\nversion = 1\n\n\
             id = {id}\nslug = \"{slug}\"\ntitle = \"{title}\"\n\
             status = \"{status}\"\ncreated = \"2026-06-04\"\nupdated = \"2026-06-04\"\n"
        );
        let toml_path = entity::id_path(root, &RFC_KIND.kind, id, entity::Ext::Toml);
        std::fs::write(&toml_path, toml).unwrap();
    }

    /// The RFC tree root for a project root.
    fn rfc_root(root: &Path) -> PathBuf {
        root.join(RFC_DIR)
    }

    // --- VT-1 mint→show round-trip (integration) ---

    #[test]
    fn mint_show_round_trip_rfc_new_writes_both_tiers_rfc_show_renders_them() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // mint
        run_new(Some(root.to_path_buf()), Some("Use Rust?".into()), None).unwrap();
        let rfc_dir = root.join(".doctrine/rfc");
        assert!(rfc_dir.join("001/rfc-001.toml").is_file());
        assert!(rfc_dir.join("001/rfc-001.md").is_file());
        // show — prove the status-bearing path synthesises both tiers.
        // Default columns: id, status, title (no slug unless --columns …,slug).
        let out = governance::list_rows(&RFC_KIND, root, ListArgs::default()).unwrap();
        assert!(out.contains("RFC-001"));
        assert!(out.contains("open"));
        assert!(out.contains("Use Rust?"));
    }

    // --- VT-5: status field ---

    #[test]
    fn freshly_minted_rfc_reads_status_open_via_status_bearing_path() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        run_new(Some(root.to_path_buf()), Some("Use Rust?".into()), None).unwrap();
        // Read via the status-bearing path (meta::Meta parses the full authored
        // fields including status), NOT the status-less meta::read_id — VT-5.
        let rfc_root = root.join(RFC_KIND.kind.dir);
        let toml_path = rfc_root.join("001/rfc-001.toml");
        let text = std::fs::read_to_string(&toml_path).unwrap();
        let parsed: crate::meta::Meta = toml::from_str(&text).unwrap();
        assert_eq!(parsed.status, "open");
    }

    // --- PHASE-02 VT-1: status transitions ----------------------------------

    #[test]
    fn status_transition_open_to_resolved_persists() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        rfc_at(root, 1, "open", "think-rust", "Think Rust?");

        governance::set_status(
            &RFC_KIND,
            root,
            1,
            RfcStatus::Resolved.as_str(),
            "2099-01-01",
        )
        .unwrap();

        let meta = crate::meta::read_meta(&rfc_root(root), "rfc", 1, "RFC").unwrap();
        assert_eq!(meta.status, "resolved");
        // The file carries the stamp; `set_status` preserved the rest.
        let body = std::fs::read_to_string(rfc_root(root).join("001/rfc-001.toml")).unwrap();
        assert!(body.contains("updated = \"2099-01-01\""));
        assert!(body.contains("id = 1"));
        assert!(body.contains("slug = \"think-rust\""));
    }

    #[test]
    fn status_transition_open_to_withdrawn_persists() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        rfc_at(root, 3, "open", "too-early", "Too Early");

        governance::set_status(
            &RFC_KIND,
            root,
            3,
            RfcStatus::Withdrawn.as_str(),
            "2099-01-01",
        )
        .unwrap();

        let meta = crate::meta::read_meta(&rfc_root(root), "rfc", 3, "RFC").unwrap();
        assert_eq!(meta.status, "withdrawn");
    }

    #[test]
    fn status_transition_all_three_known_statuses_accepted_by_set_status() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        // Prove each of the three known statuses is a legal transition target
        // (the set_status seam accepts every known value; the RFC has no FSM gate).
        for (id, target) in [
            (1, RfcStatus::Open.as_str()),
            (2, RfcStatus::Resolved.as_str()),
            (3, RfcStatus::Withdrawn.as_str()),
        ] {
            rfc_at(
                root,
                id,
                "open",
                &format!("slug-{id}"),
                &format!("Title {id}"),
            );
            governance::set_status(&RFC_KIND, root, id, target, "2099-01-01").unwrap();
            let meta = crate::meta::read_meta(&rfc_root(root), "rfc", id, "RFC").unwrap();
            assert_eq!(meta.status, target, "id {id}");
        }
    }

    #[test]
    fn status_transition_set_status_on_a_missing_rfc_errors() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        rfc_at(root, 1, "open", "exists", "Exists");
        let err = governance::set_status(
            &RFC_KIND,
            root,
            9,
            RfcStatus::Resolved.as_str(),
            "2099-01-01",
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("not found"),
            "missing RFC is a hard error: {err}"
        );
    }

    // --- PHASE-02 VT-2: list visibility -------------------------------------

    #[test]
    fn rfc_list_default_hides_resolved_and_withdrawn() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        rfc_at(root, 1, "open", "alpha", "Alpha");
        rfc_at(root, 2, "resolved", "beta", "Beta");
        rfc_at(root, 3, "withdrawn", "gamma", "Gamma");

        // Default list: only open RFC-001 visible.
        let out = governance::list_rows(&RFC_KIND, root, ListArgs::default()).unwrap();
        assert!(out.contains("RFC-001"), "default shows open: {out}");
        assert!(!out.contains("RFC-002"), "default hides resolved: {out}");
        assert!(!out.contains("RFC-003"), "default hides withdrawn: {out}");
    }

    #[test]
    fn rfc_list_explicit_status_resolved_surfaces_it_and_filters_to_it() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        rfc_at(root, 1, "open", "alpha", "Alpha");
        rfc_at(root, 2, "resolved", "beta", "Beta");

        let out = governance::list_rows(
            &RFC_KIND,
            root,
            ListArgs {
                status: vec!["resolved".into()],
                ..ListArgs::default()
            },
        )
        .unwrap();
        assert!(out.contains("RFC-002"), "--status resolved surfaces: {out}");
        assert!(
            !out.contains("RFC-001"),
            "and filters to resolved only: {out}"
        );
    }

    #[test]
    fn rfc_list_all_reveals_every_status() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        rfc_at(root, 1, "open", "alpha", "Alpha");
        rfc_at(root, 2, "resolved", "beta", "Beta");
        rfc_at(root, 3, "withdrawn", "gamma", "Gamma");

        let out = governance::list_rows(
            &RFC_KIND,
            root,
            ListArgs {
                all: true,
                ..ListArgs::default()
            },
        )
        .unwrap();
        assert!(out.contains("RFC-001"), "--all reveals open: {out}");
        assert!(out.contains("RFC-002"), "--all reveals resolved: {out}");
        assert!(out.contains("RFC-003"), "--all reveals withdrawn: {out}");
    }

    // --- PHASE-02 VT-3: bogus --status errors -------------------------------

    #[test]
    fn rfc_list_rejects_an_unknown_status_with_a_clear_error() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        rfc_at(root, 1, "open", "alpha", "Alpha");

        let err = governance::list_rows(
            &RFC_KIND,
            root,
            ListArgs {
                status: vec!["bogus".into()],
                ..ListArgs::default()
            },
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("bogus"), "names the bad value: {err}");
        // The error must list the RFC known-status set, not a silent empty result.
        assert!(err.contains("open"), "lists the known set: {err}");
        assert!(
            err.contains("resolved"),
            "known set includes resolved: {err}"
        );
        assert!(
            err.contains("withdrawn"),
            "known set includes withdrawn: {err}"
        );
    }

    #[test]
    fn rfc_list_accepts_every_known_status_value() {
        let dir = tempfile::tempdir().unwrap();
        for s in RFC_STATUSES {
            assert!(
                governance::list_rows(
                    &RFC_KIND,
                    dir.path(),
                    ListArgs {
                        status: vec![(*s).to_string()],
                        ..ListArgs::default()
                    },
                )
                .is_ok(),
                "known status `{s}` accepted by --status"
            );
        }
    }
}
