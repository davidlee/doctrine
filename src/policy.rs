// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine policy` — standing rules of governance. A thin per-kind module over
//! the shared `governance` spine (SL-030): this owns only the *policy-specific*
//! parts — the `GovKind` descriptor, the clap status enum + known-set, the
//! hide-set, and the scaffold/render. All kind-agnostic CLI/status machinery
//! lives in `crate::governance`, parameterized by `POLICY_KIND`.
//!
//! A policy is a numeric directory under `.doctrine/policy/` holding a sister
//! `policy-NNN.toml` (structured, queried metadata) and a scaffolded
//! `policy-NNN.md` prose body, with an `NNN-slug` symlink alias — the ADR shape
//! exactly (design SL-030 §5.3), riding `entity::Kind` over the kind-blind engine.
//! Unlike ADR, a policy records a *standing rule*, not a decision: its status
//! vocab is `draft/required/deprecated/retired` and supersession is a
//! relationship, not a status (design D2).

use std::io::{self, Write};
use std::path::PathBuf;

use crate::entity::{Artifact, Fileset, Kind, ScaffoldCtx};
use crate::governance::{self, GovKind};
use crate::listing::{Format, ListArgs};
use crate::tomlfmt::toml_string;

/// Relative dir of the policy tree inside the project root. Distinct top-level
/// tree (project-global governance), mirroring `.doctrine/adr`.
const POLICY_DIR: &str = ".doctrine/policy";

/// The policy governance descriptor the spine binds. `prefix` is the canonical-id
/// stem (`POL-007`); `stem` is the file/JSON stem (`"policy"`) — policy is the
/// first kind where `stem != prefix.to_lowercase()` (design §10 R3), validating
/// the explicit field. `pub(crate)` so `boot` projects policy rows via
/// `governance::list_rows(&policy::POLICY_KIND, …)` (SL-030 PHASE-04).
pub(crate) const POLICY_KIND: GovKind = GovKind {
    kind: Kind {
        dir: POLICY_DIR,
        prefix: "POL",
        scaffold: policy_scaffold,
    },
    stem: "policy",
    statuses: POLICY_STATUSES,
    hidden: is_hidden,
};

/// The status transitions `policy status` writes. A standing rule's life:
/// `draft → required → deprecated / retired`. `required` is the in-force state
/// (the boot section projects only these, SL-030 PHASE-04). Supersession is a
/// relationship (`relationships.supersedes`), not a status (design D2) — so no
/// `Superseded` variant. A flat enum, no per-state stamping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum PolicyStatus {
    Draft,
    Required,
    Deprecated,
    Retired,
}

impl PolicyStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Required => "required",
            Self::Deprecated => "deprecated",
            Self::Retired => "retired",
        }
    }
}

/// The policy status known-set — the authority `governance::list_rows` checks
/// `--status` against. Mirrors `PolicyStatus`'s variants, kept in lockstep by
/// `policy_known_set_matches_variants` (a drift canary). The enum kinds cannot
/// store an out-of-vocab status, so this doubles as the complete vocabulary.
pub(crate) const POLICY_STATUSES: &[&str] = &["draft", "required", "deprecated", "retired"];

/// The `policy list` hide-set (design §5.3): `deprecated` (sunsetting but extant)
/// and `retired` (terminal off) policies no longer govern, so they drop from the
/// default list. The override (`--all` or any explicit `--status`) reveals them —
/// handled in `listing::retain`, not here. Bound as `POLICY_KIND.hidden`.
fn is_hidden(status: &str) -> bool {
    matches!(status, "deprecated" | "retired")
}

// ---------------------------------------------------------------------------
// Pure: render, scaffold (the policy-specific templates — per-kind data)
// ---------------------------------------------------------------------------

/// Render `policy-<id>.toml` from the embedded template by token substitution.
/// The `id/slug/title/status` keys round-trip into `meta::Meta` (VT-1).
fn render_policy_toml(id: u32, slug: &str, title: &str, date: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/policy.toml")?
        .replace("{{id}}", &id.to_string())
        .replace("{{slug}}", &toml_string(slug))
        .replace("{{title}}", &toml_string(title))
        .replace("{{date}}", date))
}

/// Render `policy-<id>.md` from the embedded template: `{{ref}}` (the canonical
/// id, e.g. `POL-007`) + `{{title}}`. No YAML frontmatter (D1) — metadata lives
/// in the sister toml, not the prose.
fn render_policy_md(canonical_id: &str, title: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/policy.md")?
        .replace("{{ref}}", canonical_id)
        .replace("{{title}}", title))
}

/// The policy fileset: sister TOML, prose body, and `<id>-<slug>` symlink, all
/// relative to the policy tree root. Structurally `adr_scaffold`. Bound as
/// `POLICY_KIND.kind.scaffold`.
fn policy_scaffold(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
    let id = ctx.id;
    let name = format!("{id:03}");
    Ok(vec![
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/policy-{name}.toml")),
            body: render_policy_toml(id, ctx.slug, ctx.title, ctx.date)?,
        },
        Artifact::File {
            rel_path: PathBuf::from(format!("{name}/policy-{name}.md")),
            body: render_policy_md(ctx.canonical, ctx.title)?,
        },
        Artifact::Symlink {
            rel_path: PathBuf::from(format!("{name}-{}", ctx.slug)),
            target: name,
        },
    ])
}

// ---------------------------------------------------------------------------
// CLI entry points — thin forwarders binding POLICY_KIND into the spine
// ---------------------------------------------------------------------------

/// `doctrine policy new` → `governance::run_new(&POLICY_KIND, …)`.
pub(crate) fn run_new(
    path: Option<PathBuf>,
    title: Option<String>,
    slug: Option<String>,
) -> anyhow::Result<()> {
    governance::run_new(&POLICY_KIND, path, title, slug)
}

/// `doctrine policy list` → `governance::run_list(&POLICY_KIND, …)`.
pub(crate) fn run_list(path: Option<PathBuf>, args: ListArgs) -> anyhow::Result<()> {
    governance::run_list(&POLICY_KIND, path, args)
}

/// `doctrine policy show <POL-NNN>` → `governance::run_show(&POLICY_KIND, …)`.
pub(crate) fn run_show(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
) -> anyhow::Result<()> {
    governance::run_show(&POLICY_KIND, path, reference, format)
}

/// `doctrine policy status` — bind the concrete `PolicyStatus` enum at the
/// boundary, delegate the edit-preserving transition to the spine, then print.
/// The clock is read here and passed in (the pure/imperative split).
pub(crate) fn run_status(
    path: Option<PathBuf>,
    id: u32,
    status: PolicyStatus,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let gov_root = root.join(POLICY_KIND.kind.dir);
    governance::set_status(
        &POLICY_KIND,
        &gov_root,
        id,
        status.as_str(),
        &crate::clock::today(),
    )?;
    writeln!(io::stdout(), "POL {id:03}: {}", status.as_str())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests — the policy-specific data (render, scaffold, known-set). The
// shared-spine behaviour tests (list/show/status/parse) live in `governance.rs`,
// driven by `ADR_KIND` (SL-030 PHASE-02); they parameterize identically over
// `POLICY_KIND`, so they are not re-run here.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meta::Meta;
    use std::path::Path;

    // --- VT-1: render + round-trip ---

    #[test]
    fn render_policy_toml_round_trips_to_metadata() {
        let body =
            render_policy_toml(7, "two-space-indent", "Two-space indent", "2026-06-04").unwrap();
        // The four list fields parse into meta::Meta …
        let parsed: Meta = toml::from_str(&body).unwrap();
        assert_eq!(
            parsed,
            Meta {
                id: 7,
                slug: "two-space-indent".to_string(),
                title: "Two-space indent".to_string(),
                status: "draft".to_string(),
            }
        );
        // status seeds draft, the date is injected, no token survives.
        assert!(body.contains("created = \"2026-06-04\""));
        assert!(!body.contains("{{"));
    }

    #[test]
    fn render_policy_toml_escapes_hostile_title_and_slug() {
        // A title / explicit slug carrying the quoted-literal breakers (`"`, `\`,
        // newline) must still render a parseable toml that round-trips.
        let title = crate::tomlfmt::HOSTILE_TITLE;
        let slug = crate::tomlfmt::HOSTILE_SLUG;
        let body = render_policy_toml(7, slug, title, "2026-06-04").unwrap();
        let parsed: Meta = toml::from_str(&body).unwrap();
        assert_eq!(parsed.slug, slug);
        assert_eq!(parsed.title, title);
    }

    #[test]
    fn render_policy_toml_relationships_are_preserved_and_ignored_by_meta() {
        let body = render_policy_toml(1, "s", "T", "2026-06-04").unwrap();
        // The [relationships] table parses as a whole document …
        let doc: toml::Value = toml::from_str(&body).unwrap();
        for axis in ["supersedes", "superseded_by", "related", "tags"] {
            assert!(
                doc["relationships"][axis].as_array().unwrap().is_empty(),
                "{axis} should seed empty"
            );
        }
        // … yet Meta deserialises fine, ignoring the unknown table.
        assert!(toml::from_str::<Meta>(&body).is_ok());
    }

    #[test]
    fn render_policy_md_substitutes_ref_and_title_without_frontmatter() {
        let body = render_policy_md("POL-007", "Two-space indent").unwrap();
        assert!(body.starts_with("# POL-007: Two-space indent"));
        assert!(!body.contains("{{ref}}"));
        assert!(!body.contains("{{title}}"));
        // no YAML frontmatter (D1 — metadata is in the toml, not the prose).
        assert!(!body.starts_with("---"));
        assert!(!body.contains("\n---\n"));
    }

    // --- VT-1: scaffold shape (stem != prefix — design §10 R3) ---

    #[test]
    fn policy_scaffold_lays_out_two_files_and_a_symlink() {
        let ctx = ScaffoldCtx {
            id: 7,
            canonical: "POL-007",
            slug: "two-space-indent",
            title: "Two-space indent",
            date: "2026-06-04",
        };
        let fileset = policy_scaffold(&ctx).unwrap();
        assert_eq!(fileset.len(), 3);
        // filenames derive from the "policy" stem, ids from the "POL" prefix —
        // proven independent (R3: the first kind where stem != prefix.lower()).
        assert!(matches!(&fileset[0],
            Artifact::File { rel_path, body }
            if rel_path == Path::new("007/policy-007.toml") && body.contains("2026-06-04")));
        assert!(matches!(&fileset[1],
            Artifact::File { rel_path, body }
            if rel_path == Path::new("007/policy-007.md") && body.contains("POL-007: Two-space indent")));
        assert!(matches!(&fileset[2],
            Artifact::Symlink { rel_path, target }
            if rel_path == Path::new("007-two-space-indent") && target == "007"));
    }

    /// Drift canary: the `POLICY_STATUSES` known-set must stay in lockstep with
    /// the `PolicyStatus` variants (the enum kinds cannot store an out-of-vocab
    /// value, so this is the complete vocabulary). EX-4 / VT-1.
    #[test]
    fn policy_known_set_matches_variants() {
        let variants = [
            PolicyStatus::Draft,
            PolicyStatus::Required,
            PolicyStatus::Deprecated,
            PolicyStatus::Retired,
        ];
        let from_variants: Vec<&str> = variants.iter().map(|v| v.as_str()).collect();
        assert_eq!(from_variants, POLICY_STATUSES.to_vec());
    }

    /// The hide-set must only name statuses in the known-set (design §5.5
    /// invariant: hide-set ⊆ known-set).
    #[test]
    fn policy_hide_set_is_a_subset_of_the_known_set() {
        for s in POLICY_STATUSES {
            // every hidden status is a known status — vacuously holds, but the
            // converse guard: a status flagged hidden must be in the vocab.
            let _ = is_hidden(s);
        }
        assert!(is_hidden("deprecated"));
        assert!(is_hidden("retired"));
        assert!(!is_hidden("draft"));
        assert!(!is_hidden("required"));
    }

    // --- an empty / symbol-only title bails for an explicit --slug ---

    #[test]
    fn run_new_bails_for_a_slug_on_a_symbol_only_title() {
        let dir = tempfile::tempdir().unwrap();
        let err = run_new(Some(dir.path().to_path_buf()), Some("!!!".into()), None).unwrap_err();
        assert!(err.to_string().contains("pass --slug"));
    }
}
