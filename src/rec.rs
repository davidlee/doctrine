// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine rec` — the REC reconciliation-record kind (SPEC-002, SL-042 P1).
//! A REC is the immutable ledger of ONE reconciliation act: the requirement-status
//! deltas it applied, the `move` it represents, and the coverage evidence it rests
//! on. It is **status-less** (design D-Q3): one REC per act, no lifecycle, no
//! transition verb — the commit is the act boundary. The reconcile *writer* that
//! populates deltas from observed coverage/drift is the dependent Slice B; P1
//! stands up the kind itself (schema + scaffold/show/list + `validate` wiring).
//!
//! REC rides the SL-040 review-kind seam verbatim (no parallel impl): a numbered
//! authored kind with an eager-materialised fileset (`rec-NNN.toml` + `rec-NNN.md`
//! plus the `NNN-slug` symlink), a `KINDS` row, and the status-less scan-path
//! reader (`meta::read_id`) it uses because it has no authored `status` field. Its
//! fields exceed `ScaffoldCtx`, so like review it materialises eagerly rather than
//! via `Kind.scaffold`.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::entity::{self, Kind, Materialised};
use crate::listing::{self, Column, Format, ListArgs};
use crate::tomlfmt::toml_string;

// ---------------------------------------------------------------------------
// Pure core — the one closed vocabulary REC owns (`move`), with an `as_str`
// render mirror + a `&[&str]` known-set kept in lockstep by a drift canary test
// (the review.rs / adr.rs pattern).
// ---------------------------------------------------------------------------

/// The reconciliation move a REC represents (design §5.3, D-Q3). The closed 3-set:
/// `accept` (evidence confirms authored status), `revise` (authored status moves to
/// match evidence), `redesign` (the drift escalates to a design change — carries
/// **empty** `status_deltas`, F7: it records the escalation, not an instance-truth
/// write).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecMove {
    Accept,
    Revise,
    Redesign,
}

impl RecMove {
    /// The on-disk render mirror — lockstep-guarded against [`MOVES`] by
    /// `move_known_set_matches_variants`.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::Revise => "revise",
            Self::Redesign => "redesign",
        }
    }

    /// Parse a `--move` token against the closed 3-set (the review.rs `Facet::parse`
    /// pattern — keeps the pure-core enum clap-free). The error names every valid
    /// move.
    pub(crate) fn parse(s: &str) -> Result<Self, String> {
        match s {
            "accept" => Ok(Self::Accept),
            "revise" => Ok(Self::Revise),
            "redesign" => Ok(Self::Redesign),
            other => Err(format!(
                "unknown move `{other}` (known: {})",
                MOVES.join(", ")
            )),
        }
    }
}

/// The `RecMove` known-set. Lockstep-guarded against the enum by
/// `move_known_set_matches_variants`.
const MOVES: &[&str] = &["accept", "revise", "redesign"];

// ---------------------------------------------------------------------------
// Schema — the authored `rec-NNN.toml` shape, read/written as data (design §5.3).
// status_deltas / evidence_refs are array-of-tables (`[[status_delta]]` /
// `[[evidence_ref]]`), the review-finding idiom — extensible and readable.
// ---------------------------------------------------------------------------

/// One requirement-status fact this act applied (design §5.3): the requirement and
/// the `from → to` transition. Stored as strings — a REC is a ledger of facts the
/// writer (Slice B) already validated; the read path takes them verbatim.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct StatusDelta {
    pub(crate) requirement: String,
    pub(crate) from: String,
    pub(crate) to: String,
}

/// One coverage entry this act rests on, cited by the stable 4-tuple key
/// `(slice, requirement, contributing_change, mode)` (design §5.3 F3) — never a
/// `file#line` anchor (those rot). The key is **owned by coverage** (the cited
/// thing), not rec (the citer): P2 relocated the 4-tuple to `coverage::CoverageKey`
/// as its owner; rec keeps the `EvidenceRef` name via this alias so its ledger
/// schema and tests read byte-unchanged.
use crate::coverage::CoverageKey as EvidenceRef;

/// The `[rec]` metadata table (design §5.3): the `move` and the two optional edges.
/// `owning_slice` is **optional** — its optionality is *why* a freestanding REC
/// survives its slice's close (the act outlives the change).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct RecMeta {
    /// The reconciliation move (`accept` | `revise` | `redesign`). `move` is a Rust
    /// keyword, so the field is `r#move`; serde maps it to the bare `move` on disk.
    #[serde(rename = "move")]
    pub(crate) r#move: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) owning_slice: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) decision_ref: Option<String>,
}

/// The full `rec-NNN.toml` read/written as data (design §5.3). No authored `status`
/// field (D-Q3) — REC scans via the status-less `meta::read_id` path. The deltas
/// and evidence default to empty (the redesign-REC shape, F7, and the skeleton a
/// fresh `rec new` writes before the writer populates it).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct RecDoc {
    pub(crate) id: u32,
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) rec: RecMeta,
    #[serde(default)]
    pub(crate) status_delta: Vec<StatusDelta>,
    #[serde(default)]
    pub(crate) evidence_ref: Vec<EvidenceRef>,
    #[serde(default, deserialize_with = "deserialize_tags_lenient")]
    pub(crate) tags: Vec<String>,
}

/// Lenient [`tags`] deserializer: absent → empty vec; non-array value
/// (e.g. a scalar `tags = "not-an-array"`) → empty vec. Only a real
/// `toml::Value::Array` is read, with non-string elements silently
/// dropped. This keeps a single malformed tag value from crashing the
/// entity parse (SL-169).
fn deserialize_tags_lenient<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let value = toml::Value::deserialize(deserializer)?;
    match value {
        toml::Value::Array(arr) => Ok(arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()),
        _ => Ok(Vec::new()),
    }
}

// ---------------------------------------------------------------------------
// Kind row + eager render (design §5.1; the review.rs eager-materialise shape).
// ---------------------------------------------------------------------------

/// Relative dir of the REC tree inside the project root — a distinct top-level
/// authored tree (design §5.1), parallel to `.doctrine/review`.
pub(crate) const REC_DIR: &str = ".doctrine/rec";

/// The REC kind: `rec-NNN.toml` + `rec-NNN.md` + `NNN-slug` symlink, riding the
/// kind-blind engine. The scaffold is inert — REC's `[rec]` fields exceed
/// `ScaffoldCtx`, so it renders its fileset eagerly in [`run_new`] (the
/// `review` rationale); this stub exists only to satisfy the `Kind` descriptor
/// `integrity::KINDS` references.
pub(crate) const REC_KIND: Kind = Kind {
    dir: REC_DIR,
    prefix: crate::kinds::REC,
    stem: "rec",
    scaffold: rec_scaffold_unused,
};

/// Inert scaffold — see [`REC_KIND`]. REC never rides `Kind.scaffold`; this is the
/// descriptor stub.
fn rec_scaffold_unused(_ctx: &entity::ScaffoldCtx<'_>) -> anyhow::Result<entity::Fileset> {
    anyhow::bail!("rec materialises eagerly, not via Kind.scaffold")
}

/// Render `rec-NNN.toml` from the embedded template (design §5.1). Every
/// user-supplied string (`slug`/`title`/`owning_slice`/`decision_ref`) and the
/// closed-vocab `move` is spliced through `toml_string`, so a hostile value can
/// neither break the document nor inject a key
/// (mem.pattern.render.toml-splice-escape-user-values). A fresh REC writes an empty
/// ledger — deltas/evidence are appended later by the reconcile writer (Slice B).
fn render_rec_toml(id: u32, slug: &str, title: &str, meta: &RecMeta) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/rec.toml")?
        .replace("{{id}}", &id.to_string())
        .replace("{{slug}}", &toml_string(slug))
        .replace("{{title}}", &toml_string(title))
        .replace("{{move}}", &toml_string(&meta.r#move))
        .replace(
            "{{owning_slice}}",
            &optional_line("owning_slice", meta.owning_slice.as_deref()),
        )
        .replace(
            "{{decision_ref}}",
            &optional_line("decision_ref", meta.decision_ref.as_deref()),
        ))
}

/// An optional `key = "value"\n` line for the template, or the empty string when
/// the value is absent (the review `target_phase` pattern). The value rides
/// `toml_string` so a hostile ref cannot break the table.
fn optional_line(key: &str, value: Option<&str>) -> String {
    match value {
        Some(v) => {
            let mut line = String::from(key);
            line.push_str(" = ");
            line.push_str(&toml_string(v));
            line.push('\n');
            line
        }
        None => String::new(),
    }
}

/// Render `rec-NNN.md` — the rationale companion (design §5.1). Plain markdown
/// token substitution (no toml-splice escaping: markdown body, not a structured
/// value).
fn render_rec_md(canonical: &str, r#move: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/rec.md")?
        .replace("{{ref}}", canonical)
        .replace("{{move}}", r#move))
}

/// Render the POPULATED `rec-NNN.toml` for the atomic reconcile-writer path
/// (SL-044 B·P2, D-B8): the template head (`id`/`slug`/`title`/`[rec]`) plus the
/// `[[status_delta]]` and `[[evidence_ref]]` array-of-tables appended for each
/// recorded fact. One materialise, never `rec new` + append (REC immutability,
/// SL-042 D-Q3). Every spliced value rides `toml_string` so a hostile delta/key
/// can neither break the document nor inject a key
/// (mem.pattern.render.toml-splice-escape-user-values) — the array-of-tables idiom
/// already used by the read schema, hand-emitted so no naive `toml::to_string`
/// bypasses the escaping seam.
fn render_rec_toml_populated(doc: &RecDoc) -> anyhow::Result<String> {
    let mut out = render_rec_toml(doc.id, &doc.slug, &doc.title, &doc.rec)?;
    for d in &doc.status_delta {
        out.push_str(&status_delta_table(d));
    }
    for e in &doc.evidence_ref {
        out.push_str(&evidence_ref_table(e));
    }
    Ok(out)
}

/// One `[[status_delta]]` table (the read-schema idiom), every value escaped.
fn status_delta_table(d: &StatusDelta) -> String {
    [
        "\n[[status_delta]]\n".to_owned(),
        format!("requirement = {}\n", toml_string(&d.requirement)),
        format!("from = {}\n", toml_string(&d.from)),
        format!("to = {}\n", toml_string(&d.to)),
    ]
    .concat()
}

/// One `[[evidence_ref]]` table — the stable 4-tuple coverage key, every value
/// escaped (never a raw splice).
fn evidence_ref_table(e: &EvidenceRef) -> String {
    [
        "\n[[evidence_ref]]\n".to_owned(),
        format!("slice = {}\n", toml_string(&e.slice)),
        format!("requirement = {}\n", toml_string(&e.requirement)),
        format!(
            "contributing_change = {}\n",
            toml_string(&e.contributing_change)
        ),
        format!("mode = {}\n", toml_string(&e.mode)),
    ]
    .concat()
}

/// Atomically materialise a POPULATED REC from a pre-composed [`RecDoc`] — the sole
/// author seam the reconcile writer (SL-044 B·P2) calls. Mirrors [`run_new`]'s
/// claim-retry materialise but writes the populated ledger (deltas + evidence) in
/// ONE shot (D-B8): one CLI invocation = one move = one atomic REC. The caller has
/// already composed and validated the doc and resolved any `owning_slice` forward
/// edge; this fn owns only the id-claim + write. `doc.id` is a placeholder — the
/// engine assigns the real reserved id and rewrites the `id`/`{{ref}}` tokens.
/// Returns the materialised id.
pub(crate) fn materialise_populated(root: &Path, doc: &RecDoc) -> anyhow::Result<u32> {
    let trunk_ids = crate::git::trunk_entity_ids(root, REC_DIR)?;
    let (backend, mut reserved) =
        crate::reserve::backend(root, REC_KIND.prefix, crate::install::prompt_confirm)?;
    let out: Materialised = entity::materialise_fresh_prebuilt(
        &*backend,
        root,
        REC_DIR,
        REC_KIND.prefix,
        &trunk_ids,
        &mut reserved,
        |id, canonical| {
            let name = format!("{id:03}");
            // The claimed id overrides the placeholder so the rendered `id =` and
            // every id-bearing path match the reserved id.
            let placed = RecDoc { id, ..doc.clone() };
            Ok(vec![
                entity::Artifact::File {
                    rel_path: PathBuf::from(format!("{name}/rec-{name}.toml")),
                    body: render_rec_toml_populated(&placed)?,
                },
                entity::Artifact::File {
                    rel_path: PathBuf::from(format!("{name}/rec-{name}.md")),
                    body: render_rec_md(canonical, &doc.rec.r#move)?,
                },
                entity::Artifact::Symlink {
                    rel_path: PathBuf::from(format!("{name}-{}", doc.slug)),
                    target: name,
                },
            ])
        },
    )?;
    out.eid
        .numeric_id()
        .context("rec kind must yield a numeric id")
}

// ---------------------------------------------------------------------------
// CLI: `rec new`
// ---------------------------------------------------------------------------

/// The bundled `rec new` arguments — one struct to dodge the clippy arg-ceiling
/// (mem.pattern.lint.cli-handler-args-struct).
pub(crate) struct NewArgs {
    pub(crate) r#move: RecMove,
    pub(crate) owning_slice: Option<String>,
    pub(crate) decision_ref: Option<String>,
    pub(crate) title: Option<String>,
}

/// `doctrine rec new --move M [--owning-slice SL-NNN] [--decision DEC-NNN]` —
/// allocate a fresh REC and write its skeleton ledger (empty deltas/evidence) plus
/// the rationale md. The reconcile writer (Slice B) populates the deltas; P1 stands
/// up the kind. Optional edges (`owning_slice`/`decision_ref`) are validated up
/// front (design §7 forward-edge guard): a dangling ref is refused BEFORE any id is
/// claimed, so a bad edge never mints an entity.
pub(crate) fn run_new(path: Option<PathBuf>, args: &NewArgs) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;

    // Forward-edge validation: refuse a dangling `owning_slice` BEFORE claiming an
    // id (reusing the corpus id table, integrity::KINDS) — a slice is a numbered
    // doctrine entity, so the edge must resolve. `decision_ref` is NOT validated:
    // a DEC is now a 2-part numbered kind, but `decision_ref` carries an *external*
    // 3-part decision cite (e.g. `DEC-005-C`), not a doctrine entity in `KINDS`,
    // so it carries as free-text (design §5.3).
    if let Some(owning) = &args.owning_slice {
        crate::integrity::ensure_ref_resolves(&root, owning)?;
    }

    let title = args
        .title
        .clone()
        .unwrap_or_else(|| format!("{} reconciliation", args.r#move.as_str()));
    let slug = crate::input::resolve_slug(&title, None)?;
    let meta = RecMeta {
        r#move: args.r#move.as_str().to_owned(),
        owning_slice: args.owning_slice.clone(),
        decision_ref: args.decision_ref.clone(),
    };

    let trunk_ids = crate::git::trunk_entity_ids(&root, REC_DIR)?;
    let (backend, mut reserved) =
        crate::reserve::backend(&root, REC_KIND.prefix, crate::install::prompt_confirm)?;
    let out: Materialised = entity::materialise_fresh_prebuilt(
        &*backend,
        &root,
        REC_DIR,
        REC_KIND.prefix,
        &trunk_ids,
        &mut reserved,
        |id, canonical| {
            let name = format!("{id:03}");
            Ok(vec![
                entity::Artifact::File {
                    rel_path: PathBuf::from(format!("{name}/rec-{name}.toml")),
                    body: render_rec_toml(id, &slug, &title, &meta)?,
                },
                entity::Artifact::File {
                    rel_path: PathBuf::from(format!("{name}/rec-{name}.md")),
                    body: render_rec_md(canonical, &meta.r#move)?,
                },
                entity::Artifact::Symlink {
                    rel_path: PathBuf::from(format!("{name}-{slug}")),
                    target: name,
                },
            ])
        },
    )?;

    let id = out
        .eid
        .numeric_id()
        .context("rec kind must yield a numeric id")?;
    writeln!(io::stdout(), "Created rec {id:03}: {}", out.dir.display())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// show / list — REC is status-less, so the readers surface the facts as-authored
// (no derived status, unlike review).
// ---------------------------------------------------------------------------

/// The `REC-NNN` canonical id for a numeric rec id, via the single id-form authority.
fn canonical_id(id: u32) -> String {
    listing::canonical_id(REC_KIND.prefix, id)
}

/// Parse a rec reference — `REC-007`, `rec-7`, or the bare id `7` — to its id.
fn parse_ref(reference: &str) -> anyhow::Result<u32> {
    let digits = reference
        .strip_prefix("REC-")
        .or_else(|| reference.strip_prefix("rec-"))
        .unwrap_or(reference);
    digits
        .parse::<u32>()
        .with_context(|| format!("not a rec reference: `{reference}` (expected `REC-007` or `7`)"))
}

/// Read one REC's `rec-NNN.toml` as data.
fn read_rec(rec_root: &Path, id: u32) -> anyhow::Result<RecDoc> {
    let name = format!("{id:03}");
    let path = rec_root.join(&name).join(format!("rec-{name}.toml"));
    let text = fs::read_to_string(&path)
        .with_context(|| format!("rec {name} not found at {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("Failed to parse {}", path.display()))
}

/// A rec's authored outbound relations (SL-046 §5.2/§5.3): the optional
/// `owning_slice` → [`RelationLabel::OwningSlice`] (→ SL) and the optional
/// `decision_ref` → [`RelationLabel::DecisionRef`]. `decision_ref` is a free-text DEC
/// ref with no `DEC` kind in `KINDS`, so it is a TARGET-UNVALIDATED label (ADR-010
/// Decision 2): carried so the data is preserved, but its target never resolves and
/// surfaces as a dangler at the scan (PHASE-03), never an edge. Reads via the
/// existing `read_rec` reader (no new TOML parse). An absent edge emits nothing.
pub(crate) fn relation_edges(
    root: &Path,
    id: u32,
) -> anyhow::Result<Vec<crate::relation::RelationEdge>> {
    use crate::relation::{RelationEdge, RelationLabel};
    let doc = read_rec(&root.join(REC_DIR), id)?;
    let mut edges = Vec::new();
    if let Some(s) = &doc.rec.owning_slice {
        edges.push(RelationEdge::new(RelationLabel::OwningSlice, s.clone()));
    }
    if let Some(d) = &doc.rec.decision_ref {
        edges.push(RelationEdge::new(RelationLabel::DecisionRef, d.clone()));
    }
    Ok(edges)
}

/// Read every `rec-NNN.toml` under the REC tree as data (for `list`).
fn read_recs(rec_root: &Path) -> anyhow::Result<Vec<RecDoc>> {
    let mut docs = Vec::new();
    for id in entity::scan_ids(rec_root)? {
        docs.push(read_rec(rec_root, id)?);
    }
    Ok(docs)
}

/// The REC corpus owned by `slice` (canonical `SL-NNN`) — every `rec-NNN.toml`
/// whose `[rec].owning_slice` matches. The reverse req→REC lookup the closure-gate
/// drift discharge needs is an **on-demand scan** of this corpus (D-B3 / ADR-004:
/// no stored `req→last_rec` reverse index — a denormalization that desyncs), so the
/// gate shell resolves "R's latest owning-slice REC" by filtering this list on a
/// `status_delta` naming R and taking the MAX id. An absent REC tree → empty (no
/// reconciliation has happened yet). One-way coupling: the `slice`-close shell
/// queries `rec`; `rec` never imports `slice` (ADR-001). O(#REC)/close (RSK-006 —
/// index later, never now).
pub(crate) fn recs_owned_by(root: &Path, slice: &str) -> anyhow::Result<Vec<RecDoc>> {
    let rec_root = root.join(REC_DIR);
    if !rec_root.is_dir() {
        return Ok(Vec::new());
    }
    Ok(read_recs(&rec_root)?
        .into_iter()
        .filter(|doc| doc.rec.owning_slice.as_deref() == Some(slice))
        .collect())
}

/// The `owning_slice` edge label for display, or `—` when the REC is freestanding.
fn owning_label(doc: &RecDoc) -> String {
    doc.rec
        .owning_slice
        .clone()
        .unwrap_or_else(|| "—".to_owned())
}

/// `doctrine rec show <REC-NNN>` — read the REC as data and render the readable
/// whole (`Table`) or the faithful toml-as-data + rationale (`Json`).
pub(crate) fn run_show(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let rec_root = root.join(REC_DIR);
    let id = parse_ref(reference)?;
    let doc = read_rec(&rec_root, id)?;
    let body = read_rationale(&rec_root, id)?;
    let out = match format {
        Format::Table => format_show(&doc, &body),
        Format::Json => show_json(&doc, &body)?,
    };
    write!(io::stdout(), "{out}")?;
    Ok(())
}

/// Read the `rec-NNN.md` rationale body (the prose companion).
fn read_rationale(rec_root: &Path, id: u32) -> anyhow::Result<String> {
    let name = format!("{id:03}");
    let path = rec_root.join(&name).join(format!("rec-{name}.md"));
    fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))
}

/// Render the `Table` show: identity header, the `move` + edges, the delta/evidence
/// counts, then the rationale body. House style — `Vec<String>` joined by `concat`
/// (avoids the `push_str(&format!)` lint).
fn format_show(doc: &RecDoc, body: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("{} — {}\n", canonical_id(doc.id), doc.title));
    parts.push(format!(
        "move={} · owning={}\n",
        doc.rec.r#move,
        owning_label(doc)
    ));
    if let Some(decision) = &doc.rec.decision_ref {
        parts.push(format!("decision: {decision}\n"));
    }
    parts.push(format!(
        "deltas: {} · evidence: {}\n",
        doc.status_delta.len(),
        doc.evidence_ref.len()
    ));
    parts.push(format!("\n{body}"));
    parts.concat()
}

/// The faithful JSON `show` row — the toml-as-data plus the rationale body.
#[derive(Debug, Serialize)]
struct ShowJson<'a> {
    #[serde(flatten)]
    doc: &'a RecDoc,
}

/// Render the `Json` show under the shared `{kind, …}` envelope.
fn show_json(doc: &RecDoc, body: &str) -> anyhow::Result<String> {
    let row = ShowJson { doc };
    let value = serde_json::json!({ "kind": "rec", "rec": row, "body": body });
    serde_json::to_string_pretty(&value).context("failed to serialize rec show JSON")
}

const REC_COLUMNS: [Column<RecDoc>; 5] = [
    Column {
        name: "id",
        header: "id",
        cell: |d| canonical_id(d.id),
        paint: listing::ColumnPaint::Fixed(owo_colors::DynColors::Ansi(
            owo_colors::AnsiColors::Cyan,
        )),
    },
    Column {
        name: "move",
        header: "move",
        cell: |d| d.rec.r#move.clone(),
        paint: listing::ColumnPaint::None,
    },
    Column {
        name: "owning",
        header: "owning",
        cell: owning_label,
        paint: listing::ColumnPaint::None,
    },
    Column {
        name: "tags",
        header: "tags",
        cell: |d| d.tags.join(", "),
        paint: listing::ColumnPaint::PerToken {
            split: |d| d.tags.clone(),
            render: listing::paint_tag,
        },
    },
    Column {
        name: "title",
        header: "title",
        cell: |d| d.title.clone(),
        paint: listing::ColumnPaint::Alternate([listing::TITLE_EVEN, listing::TITLE_ODD]),
    },
];

/// The default visible column set for `rec list`.
const REC_DEFAULT: &[&str] = &["id", "move", "owning", "title"];

/// A REC's filterable projection. REC is status-less, so the `status` axis is empty
/// — a `--status` filter is rejected by [`list_rows`] against the empty known-set.
fn key(d: &RecDoc) -> listing::FilterFields {
    listing::FilterFields {
        canonical: canonical_id(d.id),
        slug: d.slug.clone(),
        title: d.title.clone(),
        status: String::new(),
        tags: d.tags.clone(),
    }
}

/// `rec list` rows as a string — the compute half of [`run_list`]. No hide-set
/// (REC has no lifecycle), sorted by id. `--status` is rejected (no status axis).
fn list_rows(root: &Path, mut args: ListArgs) -> anyhow::Result<String> {
    listing::validate_statuses(&args.status, &[])?;
    let render = args.render;
    let columns = args.columns.take();
    let (filter, format) = listing::build(args)?;
    let rec_root = root.join(REC_DIR);
    if !rec_root.is_dir() {
        // No tree yet ⇒ no recs; render an empty result for the chosen format.
        let effective_default = listing::default_with_tags(REC_DEFAULT, false);
        return match format {
            Format::Table => Ok(listing::render_columns::<RecDoc>(
                &[],
                &listing::select_columns(&REC_COLUMNS, &effective_default, columns.as_deref())?,
                render,
            )),
            Format::Json => listing::json_envelope::<ListRow>("rec", &[]),
        };
    }
    let mut docs = listing::retain(read_recs(&rec_root)?, &filter, |_| false, key);
    docs.sort_by_key(|d| d.id);
    let any_tagged = docs.iter().any(|d| !d.tags.is_empty());
    match format {
        Format::Table => {
            let effective_default = listing::default_with_tags(REC_DEFAULT, any_tagged);
            let sel =
                listing::select_columns(&REC_COLUMNS, &effective_default, columns.as_deref())?;
            Ok(listing::render_columns(&docs, &sel, render))
        }
        Format::Json => listing::json_envelope("rec", &json_rows(&docs)),
    }
}

/// Faithful JSON rows for `list` — the prefixed id, move, owning edge, and title.
#[derive(Debug, Serialize)]
struct ListRow {
    id: String,
    r#move: String,
    owning: String,
    tags: Vec<String>,
    title: String,
}

fn json_rows(docs: &[RecDoc]) -> Vec<ListRow> {
    docs.iter()
        .map(|d| ListRow {
            id: canonical_id(d.id),
            r#move: d.rec.r#move.clone(),
            owning: owning_label(d),
            tags: d.tags.clone(),
            title: d.title.clone(),
        })
        .collect()
}

/// `doctrine rec list` — list reconciliation records by id with move, owning edge,
/// and title.
pub(crate) fn run_list(path: Option<PathBuf>, args: ListArgs) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let mut out = io::stdout();
    write!(out, "{}", list_rows(&root, args)?)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// CLI dispatch
// ---------------------------------------------------------------------------

use std::str::FromStr;

use crate::CommonListArgs;
use clap::Subcommand;

#[derive(Subcommand)]
pub(crate) enum RecCommand {
    /// Create a new reconciliation record.
    New {
        #[arg(long = "move", value_parser = RecMove::parse)]
        r#move: RecMove,
        #[arg(long)]
        owning_slice: Option<String>,
        #[arg(long = "decision")]
        decision_ref: Option<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
    /// List reconciliation records.
    List {
        #[command(flatten)]
        list: CommonListArgs,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
    /// Show one reconciliation record.
    Show {
        reference: String,
        #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
        format: Format,
        #[arg(long)]
        json: bool,
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Print the file paths of each REC entity directory.
    Paths {
        /// REC reference(s) — `REC-007` or the bare id `7`.
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

pub(crate) fn dispatch(cmd: RecCommand, color: bool) -> anyhow::Result<()> {
    match cmd {
        RecCommand::New {
            r#move,
            owning_slice,
            decision_ref,
            title,
            path,
        } => run_new(
            path,
            &NewArgs {
                r#move,
                owning_slice,
                decision_ref,
                title,
            },
        ),
        RecCommand::List { list, path } => run_list(path, list.into_list_args(color)),
        RecCommand::Show {
            reference,
            format,
            json,
            path,
        } => run_show(path, &reference, if json { Format::Json } else { format }),
        RecCommand::Paths {
            refs,
            toml,
            md,
            entity,
            single,
            path,
        } => {
            let root = crate::root::find(path, &crate::root::default_markers())?;
            let rec_root = root.join(REC_DIR);
            let sel = crate::paths::PathSelection {
                toml,
                md,
                entity,
                single,
            };
            let mut all_lines: Vec<String> = Vec::new();
            for r in &refs {
                let id = parse_ref(r)?;
                let name = format!("{id:03}");
                let entity_dir = rec_root.join(&name);
                let toml_name = format!("rec-{name}.toml");
                let md_name = format!("rec-{name}.md");
                let set = crate::paths::scan_entity_dir(
                    &entity_dir,
                    &entity_dir.join(&toml_name),
                    Some(&entity_dir.join(&md_name)),
                    &root,
                )?;
                let lines = crate::paths::select_paths(&set, &sel)?;
                all_lines.extend(lines);
            }
            write!(io::stdout(), "{}", all_lines.join("\n"))?;
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_as_str_round_trips_through_parse() {
        for m in [RecMove::Accept, RecMove::Revise, RecMove::Redesign] {
            assert_eq!(RecMove::parse(m.as_str()), Ok(m));
        }
    }

    #[test]
    fn move_parse_rejects_unknown_naming_the_known_set() {
        let err = RecMove::parse("supersede").unwrap_err();
        assert!(err.contains("supersede"), "names the bad token: {err}");
        assert!(
            err.contains("accept, revise, redesign"),
            "names the set: {err}"
        );
    }

    /// Drift canary (the review.rs `*_known_set_matches_variants` pattern): every
    /// enum variant's `as_str` is in the known-set and vice versa, in order.
    #[test]
    fn move_known_set_matches_variants() {
        let variants = [RecMove::Accept, RecMove::Revise, RecMove::Redesign];
        let from_variants: Vec<&str> = variants.iter().map(|m| m.as_str()).collect();
        assert_eq!(
            from_variants, MOVES,
            "MOVES drifted from the RecMove variants"
        );
    }

    /// The schema round-trips a fully-populated REC through serde (design VT-1):
    /// deltas, evidence (the 4-tuple), move, and both optional edges survive
    /// toml → struct → toml.
    #[test]
    fn schema_round_trips_a_populated_rec() {
        let doc = RecDoc {
            id: 7,
            slug: "accept-req-108".to_owned(),
            title: "accept REQ-108".to_owned(),
            rec: RecMeta {
                r#move: "accept".to_owned(),
                owning_slice: Some("SL-042".to_owned()),
                decision_ref: Some("DEC-005-C".to_owned()),
            },
            status_delta: vec![StatusDelta {
                requirement: "REQ-108".to_owned(),
                from: "pending".to_owned(),
                to: "active".to_owned(),
            }],
            evidence_ref: vec![EvidenceRef {
                slice: "SL-042".to_owned(),
                requirement: "REQ-108".to_owned(),
                contributing_change: "SL-042".to_owned(),
                mode: "VT".to_owned(),
            }],
            tags: Vec::new(),
        };
        let text = toml::to_string(&doc).unwrap();
        let back: RecDoc = toml::from_str(&text).unwrap();
        assert_eq!(back, doc);
    }

    /// A `redesign` REC carries EMPTY `status_deltas` (design F7) — the schema must
    /// admit an empty delta list and round-trip it.
    #[test]
    fn schema_admits_an_empty_delta_list() {
        let doc = RecDoc {
            id: 1,
            slug: "redesign-escalation".to_owned(),
            title: "redesign escalation".to_owned(),
            rec: RecMeta {
                r#move: "redesign".to_owned(),
                owning_slice: None,
                decision_ref: None,
            },
            status_delta: Vec::new(),
            evidence_ref: Vec::new(),
            tags: Vec::new(),
        };
        let text = toml::to_string(&doc).unwrap();
        let back: RecDoc = toml::from_str(&text).unwrap();
        assert!(back.status_delta.is_empty(), "empty deltas admitted (F7)");
        assert_eq!(back, doc);
    }

    /// The populated atomic renderer (B·P2): the head + one `[[status_delta]]` +
    /// one `[[evidence_ref]]` round-trip back into a faithful `RecDoc` (D-B8).
    #[test]
    fn populated_render_round_trips_deltas_and_evidence() {
        let doc = RecDoc {
            id: 3,
            slug: "accept-req-110".to_owned(),
            title: "accept REQ-110".to_owned(),
            rec: RecMeta {
                r#move: "accept".to_owned(),
                owning_slice: Some("SL-044".to_owned()),
                decision_ref: None,
            },
            status_delta: vec![StatusDelta {
                requirement: "REQ-110".to_owned(),
                from: "pending".to_owned(),
                to: "active".to_owned(),
            }],
            evidence_ref: vec![EvidenceRef {
                slice: "SL-044".to_owned(),
                requirement: "REQ-110".to_owned(),
                contributing_change: "SL-040".to_owned(),
                mode: "VT".to_owned(),
            }],
            tags: Vec::new(),
        };
        let text = render_rec_toml_populated(&doc).unwrap();
        // A REAL (un-commented) array-of-tables header, not the template's `#  …`
        // example comment — match a line that is exactly the header.
        assert!(
            text.lines().any(|l| l == "[[status_delta]]"),
            "real delta table: {text}"
        );
        assert!(
            text.lines().any(|l| l == "[[evidence_ref]]"),
            "real evidence table: {text}"
        );
        let back: RecDoc = toml::from_str(&text).unwrap();
        assert_eq!(back.status_delta, doc.status_delta);
        assert_eq!(back.evidence_ref, doc.evidence_ref);
        assert_eq!(back.rec.r#move, "accept");
        assert_eq!(back.rec.owning_slice.as_deref(), Some("SL-044"));
    }

    /// A `redesign` REC renders with NO `[[status_delta]]` table (F7) but still
    /// parses to an empty delta list.
    #[test]
    fn populated_render_emits_no_delta_table_when_empty() {
        let doc = RecDoc {
            id: 1,
            slug: "redesign".to_owned(),
            title: "redesign".to_owned(),
            rec: RecMeta {
                r#move: "redesign".to_owned(),
                owning_slice: Some("SL-044".to_owned()),
                decision_ref: None,
            },
            status_delta: Vec::new(),
            evidence_ref: Vec::new(),
            tags: Vec::new(),
        };
        let text = render_rec_toml_populated(&doc).unwrap();
        // No REAL (un-commented) status_delta table — only the template's `#  …`
        // example comment may mention the header.
        assert!(
            !text.lines().any(|l| l == "[[status_delta]]"),
            "no real delta table (F7): {text}"
        );
        let back: RecDoc = toml::from_str(&text).unwrap();
        assert!(back.status_delta.is_empty());
    }

    /// Hostile free-text in a delta/evidence field rides `toml_string`: a `"` /
    /// newline cannot break the document or inject a key.
    #[test]
    fn populated_render_escapes_hostile_delta_values() {
        let doc = RecDoc {
            id: 1,
            slug: "s".to_owned(),
            title: "t".to_owned(),
            rec: RecMeta {
                r#move: "accept".to_owned(),
                owning_slice: None,
                decision_ref: None,
            },
            status_delta: vec![StatusDelta {
                requirement: "REQ-1\"\ninjected = \"x".to_owned(),
                from: "pending".to_owned(),
                to: "active".to_owned(),
            }],
            evidence_ref: Vec::new(),
            tags: Vec::new(),
        };
        let text = render_rec_toml_populated(&doc).unwrap();
        // The document still parses (the breaker was escaped, not spliced raw) …
        let back: RecDoc = toml::from_str(&text).unwrap();
        // … and the hostile value round-trips verbatim, no injected key.
        assert_eq!(
            back.status_delta.first().unwrap().requirement,
            "REQ-1\"\ninjected = \"x"
        );
    }

    /// The `move` field renders to the bare `move` key on disk (serde rename of the
    /// `r#move` Rust keyword), not `r#move` — a hand-editor sees `move`.
    #[test]
    fn move_field_renders_as_bare_move_key() {
        let doc = RecDoc {
            id: 1,
            slug: "s".to_owned(),
            title: "t".to_owned(),
            rec: RecMeta {
                r#move: "accept".to_owned(),
                owning_slice: None,
                decision_ref: None,
            },
            status_delta: Vec::new(),
            evidence_ref: Vec::new(),
            tags: Vec::new(),
        };
        let text = toml::to_string(&doc).unwrap();
        assert!(text.contains("move = \"accept\""), "bare move key: {text}");
        assert!(!text.contains("r#move"), "no raw-ident leak: {text}");
    }
}
