// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine revision` — the REV change-axis kind (SL-066, ADR-013). A Revision is
//! a first-class **work-lifecycle** entity, peer to slice/REC on the change axis: it
//! carries a pending revise-intent against authored governance/spec truth plus a
//! staged delta payload (the `[[change]]` rows). PHASE-02 stood up the kind itself —
//! schema + scaffold/show/status — and the THREE corpus-walk arms a new `KINDS` row is
//! consumed by (G1 partition, G2 `dep_seq`, G3 outbound). PHASE-03 adds the typed
//! `[[change]]` `revises` payload (§4.4), the `revision change add` writer, and fills
//! the `relation_edges` accessor — each `[[change]]` row projects to one `Revises`
//! edge, surfaced outbound on `inspect REV-N` and inbound on `inspect ADR-X`/`REQ-N`.
//!
//! REV rides the REC eager-materialise seam verbatim (no parallel impl): a numbered
//! authored kind with an eager-materialised fileset (`revision-NNN.toml` +
//! `revision-NNN.md` plus the `NNN-slug` symlink), a `KINDS` row, and a status-ful
//! scan. Its fields exceed `ScaffoldCtx` (`status`, `approval`, the seeded dep/seq
//! block, and the future `[[change]]` payload), so like REC it materialises eagerly
//! rather than via `Kind.scaffold`.
//!
//! Lifecycle borrows backlog's work FSM — `proposed → started → done` (+ `abandoned`
//! from any non-terminal) — NOT slice's 9-state classifier. `approval` is a SEPARATE
//! field (`none | requested | approved | rejected`), ORTHOGONAL to status (entity-
//! model "approval is not lifecycle", ADR-009): lifecycle transitions are
//! approval-blind, so a `started` REV at `approval=none` is valid. The default `gate`
//! posture is a baked v1 default (ADR-009's slice-FSM-state-keyed `[conduct]` table
//! does not address Revision; extending it is deferred). `approval` is seeded `none`
//! and not mutated this phase — the `approve` verb is PHASE-05.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::entity::{self, Kind, LocalFs, Materialised};
use crate::listing::{self, Format};
use crate::requirement::ReqStatus;
use crate::tomlfmt::toml_string;

// ---------------------------------------------------------------------------
// Pure core — the two closed vocabularies REV owns (`status`, `approval`), each
// with a kebab serde derive (the single variant↔string source) + an `as_str`
// render mirror + a `&[&str]` known-set kept in lockstep by a drift canary test
// (the backlog.rs / rec.rs pattern).
// ---------------------------------------------------------------------------

/// A Revision's work-lifecycle status. Closed set, kebab serde (round-trips the
/// toml's `status`), `clap::ValueEnum` (the `revision status <state>` positional —
/// the slice-status precedent). Borrows backlog's work shape, NOT slice's 9-state
/// (R: backlog and slice lifecycles are independent vocabularies).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RevStatus {
    Proposed,
    Started,
    Done,
    Abandoned,
}

impl RevStatus {
    /// The kebab string for render (matches the serde rename). Lockstep-guarded
    /// against [`REV_STATUSES`] by `rev_statuses_matches_the_variants`.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            RevStatus::Proposed => "proposed",
            RevStatus::Started => "started",
            RevStatus::Done => "done",
            RevStatus::Abandoned => "abandoned",
        }
    }

    /// Whether this status is terminal (`done`/`abandoned`). A **REV-local**
    /// predicate — explicitly NOT slice's or backlog's terminal set (independent
    /// vocabularies). Drives the FSM `validate_transition` refuse-leave-terminal
    /// rule, and binds the G1 partition's `terminal` set (the VT-2 canary).
    const fn is_terminal(self) -> bool {
        matches!(self, RevStatus::Done | RevStatus::Abandoned)
    }
}

/// The REV status known-set — the four `RevStatus` variants, REV's OWN vocabulary
/// (NOT backlog's `open/triaged/started/resolved/closed`). The authority the G1
/// [`crate::priority::partition`] canary binds against; lockstep-guarded against the
/// enum by `rev_statuses_matches_the_variants`. Consumed only by the cross-module
/// G1 drift canary (`revision_partition_covers_the_real_vocabulary`) — a `cfg(test)`
/// reference — and by this module's own canary, so it is dead in the non-test lib;
/// the partition row spells its vocab literally, the canary proves they agree. The
/// PHASE-03 `revision change`/list surfaces will read it in non-test builds.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SL-066 PHASE-02: REV's status SSoT — consumed only by the G1 drift canary now; the PHASE-03 list/change surfaces read it in non-test builds"
    )
)]
pub(crate) const REV_STATUSES: &[&str] = &["proposed", "started", "done", "abandoned"];

/// A Revision's approval state — a SEPARATE field, ORTHOGONAL to `status` (hard
/// canon: entity-model "approval is not lifecycle", ADR-009). Closed set, kebab
/// serde. Seeded `none`; mutated by the `approve` verb (PHASE-05), NOT this phase.
/// Lifecycle transitions are approval-blind, so a `started` REV at `approval=none`
/// is valid (the apply-time checkpoint that consults approval is PHASE-05).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Approval {
    None,
    Requested,
    Approved,
    Rejected,
}

impl Approval {
    /// The kebab string for render (matches the serde rename). Lockstep-guarded
    /// against [`APPROVALS`] by `approvals_matches_the_variants`.
    const fn as_str(self) -> &'static str {
        match self {
            Approval::None => "none",
            Approval::Requested => "requested",
            Approval::Approved => "approved",
            Approval::Rejected => "rejected",
        }
    }
}

/// The `Approval` known-set — lockstep-guarded against the enum by
/// `approvals_matches_the_variants`. Test-only drift canary anchor this phase (the
/// `approve` verb that reads it in non-test builds is PHASE-05).
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SL-066 PHASE-02: approval vocab SSoT — consumed only by the drift canary now; the PHASE-05 approve verb reads it in non-test builds"
    )
)]
const APPROVALS: &[&str] = &["none", "requested", "approved", "rejected"];

// ---------------------------------------------------------------------------
// Pure FSM — the REV-local legal-transition guard (NOT slice's classifier).
// ---------------------------------------------------------------------------

/// The pure REV-local lifecycle gate (design §4.2; the backlog `validate_transition`
/// shape — pure, returns Ok/err): the legal graph is `proposed → started`,
/// `started → done`, and `{proposed, started} → abandoned`. Refuses leaving a
/// terminal source (`done`/`abandoned`) and refuses a skip (`proposed → done`). A
/// no-op (`from == to`) is allowed (idempotent). APPROVAL-BLIND — `approval` is never
/// consulted (lifecycle transitions are approval-blind, ADR-009). No clock/disk.
fn validate_transition(from: RevStatus, to: RevStatus) -> anyhow::Result<()> {
    if from == to {
        return Ok(()); // idempotent no-op.
    }
    if from.is_terminal() {
        anyhow::bail!(
            "revision is terminal (`{}`) — no transition out of a terminal status",
            from.as_str()
        );
    }
    let legal = matches!(
        (from, to),
        (RevStatus::Proposed, RevStatus::Started)
            | (RevStatus::Started, RevStatus::Done)
            | (
                RevStatus::Proposed | RevStatus::Started,
                RevStatus::Abandoned
            )
    );
    if legal {
        Ok(())
    } else {
        anyhow::bail!(
            "illegal revision transition `{}` → `{}` (legal: proposed→started, \
             started→done, proposed/started→abandoned)",
            from.as_str(),
            to.as_str()
        )
    }
}

// ---------------------------------------------------------------------------
// Schema — the authored `revision-NNN.toml` shape (PHASE-02 subset). The
// `[[change]]` payload is PHASE-03; this phase seeds an empty `[relationships]`
// dep/seq block so the G2 `dep_seq::read` arm is total now (D4).
// ---------------------------------------------------------------------------

/// The full `revision-NNN.toml` read/written as data (design §4.1, PHASE-02 subset).
/// `id/slug/title/status` round-trip into the shared `meta::Meta`; `approval` is the
/// orthogonal field. The `[[change]]` payload rows are PHASE-03 — NOT modelled here.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct RevDoc {
    pub(crate) id: u32,
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) status: RevStatus,
    pub(crate) approval: Approval,
}

// ---------------------------------------------------------------------------
// Kind row + eager render (design §4.1; the rec.rs eager-materialise shape).
// ---------------------------------------------------------------------------

/// Relative dir of the REV tree inside the project root — a distinct top-level
/// authored tree (design §4.1), parallel to `.doctrine/rec`.
pub(crate) const REV_DIR: &str = ".doctrine/revision";

/// The REV kind: `revision-NNN.toml` + `revision-NNN.md` + `NNN-slug` symlink,
/// riding the kind-blind engine. The scaffold is inert — REV's fields exceed
/// `ScaffoldCtx` (`status`/`approval`/seeded dep/seq/future `[[change]]`), so it
/// renders its fileset eagerly in [`run_new`] (the rec rationale); this stub exists
/// only to satisfy the `Kind` descriptor `integrity::KINDS` references.
pub(crate) const REV_KIND: Kind = Kind {
    dir: REV_DIR,
    prefix: crate::kinds::REV,
    scaffold: rev_scaffold_unused,
};

/// Inert scaffold — see [`REV_KIND`]. REV never rides `Kind.scaffold`; this is the
/// descriptor stub.
fn rev_scaffold_unused(_ctx: &entity::ScaffoldCtx<'_>) -> anyhow::Result<entity::Fileset> {
    anyhow::bail!("revision materialises eagerly, not via Kind.scaffold")
}

/// Render `revision-NNN.toml` from the embedded template (design §4.1). Every
/// user-supplied string (`slug`/`title`) is spliced through `toml_string`, so a
/// hostile value can neither break the document nor inject a key
/// (mem.pattern.render.toml-splice-escape-user-values). A fresh REV seeds
/// `status = "proposed"`, `approval = "none"`, the `updated` stamp, and an empty
/// `[relationships]` dep/seq block (so the G2 read arm is total). `date` is the
/// injected clock (the shell stamps it; the pure render stays clock-free).
fn render_revision_toml(id: u32, slug: &str, title: &str, date: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/revision.toml")?
        .replace("{{id}}", &id.to_string())
        .replace("{{slug}}", &toml_string(slug))
        .replace("{{title}}", &toml_string(title))
        .replace("{{date}}", date))
}

/// Render `revision-NNN.md` — the rationale companion (design §4.1). Plain markdown
/// token substitution (no toml-splice escaping: markdown body, not a structured
/// value).
fn render_revision_md(canonical: &str, title: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/revision.md")?
        .replace("{{ref}}", canonical)
        .replace("{{title}}", title))
}

// ---------------------------------------------------------------------------
// The `[[change]]` payload (design §4.3/§4.4) — the typed `revises` rows. TWO row
// shapes share one table (F3 — creation ops cannot key on an FK that does not exist
// yet): existing-target ops (`modify|retire|move|status`) key on a live FK; creation
// ops (`introduce|create`) carry a frozen `new_label` + a live `member_of` SPEC. The
// rows ARE the edges (members.toml precedent): each row projects to one `Revises`
// edge whose target is the FK (existing-target ops) or the destination SPEC (creation
// ops), so inbound reciprocity on `inspect ADR-X` covers every touching REV uniformly.
// ---------------------------------------------------------------------------

/// A `[[change]]` row's action verb (design §4.4). Closed set, kebab serde + a
/// `clap::ValueEnum` for the `revision change add --action` selector. Splits into two
/// shapes by [`ChangeAction::is_creation`]: creation ops (`introduce`/`create`) mint a
/// new entity (frozen `new_label` + `member_of`); existing-target ops key on a live FK
/// (`from` auto-captured for `status`). Lockstep-guarded against [`CHANGE_ACTIONS`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ChangeAction {
    /// Existing-target: edit a peer's statement / prose (surfaced-for-manual at apply).
    Modify,
    /// Existing-target: lifecycle-retire a peer (surfaced-for-manual at apply).
    Retire,
    /// Existing-target: move a requirement's spec membership (surfaced-for-manual).
    Move,
    /// Existing-target: a requirement status move — the one auto-applied row (PHASE-05),
    /// `from` auto-captured at `change add` (the §4.5 from-guard input).
    Status,
    /// Creation: introduce a new requirement into a live `member_of` SPEC.
    Introduce,
    /// Creation: create a new spec.
    Create,
}

impl ChangeAction {
    /// The kebab string for render (matches the serde rename). Lockstep-guarded against
    /// [`CHANGE_ACTIONS`] by `change_actions_matches_the_variants`.
    const fn as_str(self) -> &'static str {
        match self {
            ChangeAction::Modify => "modify",
            ChangeAction::Retire => "retire",
            ChangeAction::Move => "move",
            ChangeAction::Status => "status",
            ChangeAction::Introduce => "introduce",
            ChangeAction::Create => "create",
        }
    }

    /// Whether this is a CREATION op (`introduce`/`create`) — keys on no pre-existing
    /// FK, so it carries a frozen `new_label` (REQUIRED, E4) + a live `member_of`
    /// instead of a `target`. The complement is the existing-target ops, which key on a
    /// live FK with `from` auto-captured for `status`.
    const fn is_creation(self) -> bool {
        matches!(self, ChangeAction::Introduce | ChangeAction::Create)
    }
}

/// The `ChangeAction` known-set — lockstep-guarded against the enum by
/// `change_actions_matches_the_variants`. Consumed only by that drift canary (a
/// `cfg(test)` reference), so it reads as dead in the non-test lib; the `--action`
/// surface binds the enum directly via `clap::ValueEnum`, not this set.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SL-066 PHASE-03: ChangeAction vocab SSoT — consumed only by the drift canary; clap binds the enum directly"
    )
)]
const CHANGE_ACTIONS: &[&str] = &["modify", "retire", "move", "status", "introduce", "create"];

/// One authored `[[change]]` row (design §4.4) — the typed `revises` payload. TWO
/// shapes share the struct via optional columns: existing-target ops populate `target`
/// (the live FK); creation ops populate `new_label` (required, frozen) + `member_of`
/// (a live SPEC). `primary` is a display/headline hint only (F1 — at most one,
/// optional; nothing functional keys on it). Detail columns (`from`/`to_status`/
/// `new_statement`/`allocated`) are optional, populated only where the action warrants.
/// `#[serde(default)]` keeps a minimal hand-trimmed row parseable (the read-tolerant
/// convention).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct ChangeRow {
    /// Existing-target ops: the live peer FK (`REQ-201`, `ADR-006`). Absent on
    /// creation ops (no id exists until apply).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) target: Option<String>,
    /// The action verb (the row discriminant).
    pub(crate) action: ChangeAction,
    /// Display/headline hint only — at most one per REV (F1). NOT a dep anchor.
    #[serde(default)]
    pub(crate) primary: bool,
    /// `status` rows: the target's `ReqStatus` AT `change add` (auto-captured) — the
    /// pre-flight from-guard input (PHASE-05).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) from: Option<String>,
    /// `status` rows: the requested target status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) to_status: Option<String>,
    /// Creation ops: the FROZEN membership label (REQUIRED, E4 — keeps approved ==
    /// landed against membership churn between draft and apply).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) new_label: Option<String>,
    /// Creation ops: the destination spec (a live `SPEC-NNN`; no cross-row creation
    /// deps in v1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) member_of: Option<String>,
    /// `introduce`: the new requirement's statement line (optional detail).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) new_statement: Option<String>,
    /// Apply back-fill (PHASE-05): the id a creation op allocated. Absent until applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) allocated: Option<String>,
}

/// The `[[change]]`-row document shape — the array of payload rows. `#[serde(default)]`
/// so a REV with no rows (a fresh `revision new`) parses to an empty payload (the
/// read-tolerant convention every kind module follows). Every other key (`id`/`status`/
/// `[relationships]` …) is ignored by this targeted parse.
#[derive(Debug, Default, Deserialize)]
struct ChangeDoc {
    #[serde(default)]
    change: Vec<ChangeRow>,
}

impl ChangeRow {
    /// The `Revises` edge target of this row: the existing-target FK, or the creation
    /// op's destination `member_of` SPEC. `None` for a malformed row carrying neither —
    /// such a row contributes no edge (a dangler-free skip; `change add` refuses to
    /// author one, so this only guards a hand-broken file).
    fn edge_target(&self) -> Option<&str> {
        self.target.as_deref().or(self.member_of.as_deref())
    }
}

/// A revision's authored outbound relations (design §4.3/§4.4). Reads the `[[change]]`
/// payload and projects each row to ONE `Revises` edge (the rows ARE the edges,
/// members.toml precedent): existing-target ops emit an edge to their FK, creation ops
/// to their destination `member_of` SPEC. The engine [`crate::relation_graph`] interns
/// these as outbound `revises` (on `inspect REV-N`) and, via the indexed `in_edges`
/// reverse-adjacency, surfaces them as inbound `revises` on the touched targets
/// (`inspect ADR-X` / `inspect REQ-N`) — uniform over ALL rows, NOT keyed on `primary`.
/// A REV with no `[[change]]` rows (or only malformed rows) yields no edges. NEVER reads
/// `show` (ADR-004 §3 reserves inbound completeness to the scan-backed `inspect`).
pub(crate) fn relation_edges(
    root: &Path,
    id: u32,
) -> anyhow::Result<Vec<crate::relation::RelationEdge>> {
    let name = format!("{id:03}");
    let path = root
        .join(REV_DIR)
        .join(&name)
        .join(format!("revision-{name}.toml"));
    let text = fs::read_to_string(&path)
        .with_context(|| format!("revision {name} not found at {}", path.display()))?;
    let doc: ChangeDoc =
        toml::from_str(&text).with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(doc
        .change
        .iter()
        .filter_map(|row| {
            row.edge_target().map(|t| {
                crate::relation::RelationEdge::new(
                    crate::relation::RelationLabel::Revises,
                    t.to_owned(),
                )
            })
        })
        .collect())
}

// ---------------------------------------------------------------------------
// CLI: `revision new`
// ---------------------------------------------------------------------------

/// `doctrine revision new "<title>" [--slug S]` — allocate a fresh REV and write its
/// skeleton (`status = proposed`, `approval = none`, empty dep/seq + no `[[change]]`
/// rows) plus the rationale md. The `change add` verb (PHASE-03) populates the
/// payload. Mirrors `rec::run_new`'s claim-retry materialise. Prints the canonical
/// `REV-NNN` id.
pub(crate) fn run_new(
    path: Option<PathBuf>,
    title: Option<String>,
    slug: Option<String>,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let title = crate::input::resolve_title(title)?;
    let slug = crate::input::resolve_slug(&title, slug)?;
    let date = crate::clock::today();

    let trunk_ids = crate::git::trunk_entity_ids(&root, REV_DIR)?;
    let out: Materialised = entity::materialise_fresh_prebuilt(
        &LocalFs,
        &root,
        REV_DIR,
        REV_KIND.prefix,
        &trunk_ids,
        |id, canonical| {
            let name = format!("{id:03}");
            Ok(vec![
                entity::Artifact::File {
                    rel_path: PathBuf::from(format!("{name}/revision-{name}.toml")),
                    body: render_revision_toml(id, &slug, &title, &date)?,
                },
                entity::Artifact::File {
                    rel_path: PathBuf::from(format!("{name}/revision-{name}.md")),
                    body: render_revision_md(canonical, &title)?,
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
        .context("revision kind must yield a numeric id")?;
    writeln!(
        io::stdout(),
        "Created revision {id:03}: {}",
        out.dir.display()
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// show — REV is status-ful; the reader surfaces its own local state (no derived
// status, no cross-corpus scan — the show reader is pure over the entity's own
// file, ADR-004 §3).
// ---------------------------------------------------------------------------

/// The `REV-NNN` canonical id for a numeric rev id, via the single id-form authority.
fn canonical_id(id: u32) -> String {
    listing::canonical_id(REV_KIND.prefix, id)
}

/// Parse a revision reference — `REV-007`, `revision-7`, or the bare id `7` — to its
/// id.
fn parse_ref(reference: &str) -> anyhow::Result<u32> {
    let digits = reference
        .strip_prefix("REV-")
        .or_else(|| reference.strip_prefix("revision-"))
        .unwrap_or(reference);
    digits.parse::<u32>().with_context(|| {
        format!("not a revision reference: `{reference}` (expected `REV-007` or `7`)")
    })
}

/// Read one REV's `revision-NNN.toml` as data.
fn read_revision(rev_root: &Path, id: u32) -> anyhow::Result<RevDoc> {
    let name = format!("{id:03}");
    let path = rev_root.join(&name).join(format!("revision-{name}.toml"));
    let text = fs::read_to_string(&path)
        .with_context(|| format!("revision {name} not found at {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("Failed to parse {}", path.display()))
}

/// Read the `revision-NNN.md` rationale body (the prose companion).
fn read_rationale(rev_root: &Path, id: u32) -> anyhow::Result<String> {
    let name = format!("{id:03}");
    let path = rev_root.join(&name).join(format!("revision-{name}.md"));
    fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))
}

/// `doctrine revision show <REV-NNN>` — read the REV as data and render the readable
/// whole (`Table`) or the faithful toml-as-data + rationale (`Json`).
pub(crate) fn run_show(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let rev_root = root.join(REV_DIR);
    let id = parse_ref(reference)?;
    let doc = read_revision(&rev_root, id)?;
    let body = read_rationale(&rev_root, id)?;
    let out = match format {
        Format::Table => format_show(&doc, &body),
        Format::Json => show_json(&doc, &body)?,
    };
    write!(io::stdout(), "{out}")?;
    Ok(())
}

/// Render the `Table` show: identity header, the status + approval, then the
/// rationale body. House style — `Vec<String>` joined by `concat`.
fn format_show(doc: &RevDoc, body: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!("{} — {}\n", canonical_id(doc.id), doc.title));
    parts.push(format!(
        "status={} · approval={}\n",
        doc.status.as_str(),
        doc.approval.as_str()
    ));
    parts.push(format!("\n{body}"));
    parts.concat()
}

/// Render the `Json` show under the shared `{kind, …}` envelope. The closed enums
/// render via `as_str` (a hand-projected row, not a derive that would leak Rust
/// idents).
fn show_json(doc: &RevDoc, body: &str) -> anyhow::Result<String> {
    let value = serde_json::json!({
        "kind": "revision",
        "revision": {
            "id": canonical_id(doc.id),
            "slug": doc.slug,
            "title": doc.title,
            "status": doc.status.as_str(),
            "approval": doc.approval.as_str(),
        },
        "body": body,
    });
    serde_json::to_string_pretty(&value).context("failed to serialize revision show JSON")
}

// ---------------------------------------------------------------------------
// status — the positional `<REV-N> <state>` FSM transition verb (slice precedent),
// gated by the REV-local `validate_transition`, written edit-preservingly via the
// shared `dep_seq::set_authored_status` seam (approval untouched).
// ---------------------------------------------------------------------------

/// `doctrine revision status <REV-N> <state>` — classify and write a REV lifecycle
/// transition (design §4.2; the slice-status precedent). Reads the current authored
/// status, gates the move via [`validate_transition`] (approval-blind), writes it
/// edit-preservingly via the shared `dep_seq::set_authored_status` seam, and prints
/// the move. `approval` is NEVER touched (orthogonal — the `approve` verb is
/// PHASE-05). A missing item hard-errors (read fails); never an implicit create.
pub(crate) fn run_status(
    path: Option<PathBuf>,
    reference: &str,
    state: RevStatus,
    color: bool,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let rev_root = root.join(REV_DIR);
    let id = parse_ref(reference)?;
    let from = read_revision(&rev_root, id)?.status;
    // Gate in the shell, BEFORE the write: the REV-local legal-transition guard.
    validate_transition(from, state)?;
    set_revision_status(&rev_root, id, state, &crate::clock::today())?;
    writeln!(
        io::stdout(),
        "{} → {}",
        crate::listing::status_colored(from.as_str(), color),
        crate::listing::status_colored(state.as_str(), color)
    )?;
    Ok(())
}

/// Edit-preserving status transition on one authored `revision-NNN.toml` — the
/// backlog `set_backlog_status` precedent: delegates the write-core (no-op guard +
/// F-1 refuse + edit-preserving insert) to the shared `dep_seq::set_authored_status`
/// seam, so the inert `[relationships]` table, hand-added comments, the `approval`
/// field, and unknown keys all survive (the file is never reserialised). Stamps
/// `updated`; leaves `approval` untouched (orthogonal axis). The gate is the
/// caller's; this owns only the write.
fn set_revision_status(
    rev_root: &Path,
    id: u32,
    state: RevStatus,
    today: &str,
) -> anyhow::Result<()> {
    let name = format!("{id:03}");
    let path = rev_root.join(&name).join(format!("revision-{name}.toml"));
    let hint = format!(
        "malformed revision {name}: missing seeded `status`/`updated` (regenerate via `revision new`)"
    );
    crate::dep_seq::set_authored_status(
        &path,
        &[("status", state.as_str()), ("updated", today)],
        &hint,
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// CLI: `revision change add` — author one `[[change]]` payload row (design §4.4).
// The ONLY writer of a `Revises` edge (the rule row is `TypedVerbOnly`; `doctrine
// link … revises …` is refused at `validate_link`). Validates the row against the
// rule (target kind + existence), auto-captures `from` for a status row, freezes
// `new_label` for a creation row, enforces at-most-one `primary` and the OQ-1 dedup,
// then appends edit-preservingly. NEVER mutates the target — that is `apply` (PHASE-05).
// ---------------------------------------------------------------------------

/// The parsed `revision change add` arguments (the thin CLI carrier). `clap` parses
/// `--action` to the closed [`ChangeAction`]; the optional columns are validated in
/// [`run_change_add`] against the row shape (so a missing `--new-label` on a creation
/// op or a missing `--target` on an existing-target op is a clean, testable refusal —
/// NOT a clap arity error that couples the two shapes into one required-arg set).
pub(crate) struct ChangeAddArgs {
    pub(crate) action: ChangeAction,
    pub(crate) target: Option<String>,
    pub(crate) to_status: Option<String>,
    pub(crate) new_label: Option<String>,
    pub(crate) member_of: Option<String>,
    pub(crate) new_statement: Option<String>,
    pub(crate) primary: bool,
}

/// `doctrine revision change add REV-N --action <a> …` — author one `[[change]]` row.
///
/// Shape-routed (F3): a CREATION op (`introduce`/`create`) carries a frozen
/// `--new-label` (REQUIRED, E4) + a live `--member-of` SPEC, NO `--target`; an
/// existing-target op (`modify`/`retire`/`move`/`status`) keys on a live `--target` FK
/// validated against the `revises` rule (`{SPEC,PRD,REQ,ADR,POL,STD}` — off-target
/// refused), with `from` AUTO-captured from the target's current `ReqStatus` for a
/// `status` row. `primary` is at-most-one (F1). OQ-1: a second row of the same
/// `(action, target)` for an existing-target op is refused (the change is named once).
pub(crate) fn run_change_add(
    path: Option<PathBuf>,
    reference: &str,
    args: &ChangeAddArgs,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let rev_root = root.join(REV_DIR);
    let id = parse_ref(reference)?;
    // The REV must exist (never an implicit create); also reads the current payload
    // for the primary + dedup guards.
    read_revision(&rev_root, id)?;
    let existing = read_change_rows(&rev_root, id)?;

    let row = build_row(&root, args, &existing)?;
    append_change_row(&rev_root, id, &row)?;

    let subject = row
        .target
        .clone()
        .or_else(|| row.member_of.clone())
        .unwrap_or_default();
    writeln!(
        io::stdout(),
        "{}: change {} {}",
        canonical_id(id),
        row.action.as_str(),
        subject
    )?;
    Ok(())
}

/// Build + validate the row from the args and the REV's existing rows — PURE over the
/// inputs save the from-capture / existence reads it performs against `root`. Splits on
/// [`ChangeAction::is_creation`]; both arms enforce the at-most-one `primary` guard.
fn build_row(
    root: &Path,
    args: &ChangeAddArgs,
    existing: &[ChangeRow],
) -> anyhow::Result<ChangeRow> {
    // F1 — at most one primary per REV. Refuse a second.
    if args.primary && existing.iter().any(|r| r.primary) {
        anyhow::bail!(
            "this revision already has a primary change row — `primary` is at most one (F1)"
        );
    }

    if args.action.is_creation() {
        build_creation_row(root, args)
    } else {
        build_existing_target_row(root, args, existing)
    }
}

/// A creation op (`introduce`/`create`): `new_label` REQUIRED + frozen (E4), `member_of`
/// a live `SPEC-NNN` (no cross-row creation deps in v1). `target`/`from`/`to_status`
/// MUST be absent (a creation op keys on no FK).
fn build_creation_row(root: &Path, args: &ChangeAddArgs) -> anyhow::Result<ChangeRow> {
    anyhow::ensure!(
        args.target.is_none(),
        "`--target` is not valid for a creation op (`{}`) — it keys on no pre-existing FK; use `--member-of`",
        args.action.as_str()
    );
    anyhow::ensure!(
        args.to_status.is_none(),
        "`--to-status` is only valid for a `status` row"
    );
    // E4 — `new_label` is REQUIRED + frozen at change add (membership churn between
    // draft and apply would otherwise silently change what lands).
    let new_label = args.new_label.clone().with_context(|| {
        format!(
            "a creation op (`{}`) requires `--new-label` (frozen at change add, E4)",
            args.action.as_str()
        )
    })?;
    // `member_of` must name a live SPEC (the destination spec; not a PRD/other kind).
    let member_of = args.member_of.clone().with_context(|| {
        format!(
            "a creation op (`{}`) requires `--member-of` naming a live SPEC-NNN",
            args.action.as_str()
        )
    })?;
    let (kref, _) = crate::integrity::parse_canonical_ref(&member_of)?;
    anyhow::ensure!(
        kref.kind.prefix == "SPEC",
        "`--member-of` must name a SPEC, got a {}",
        kref.kind.prefix
    );
    crate::integrity::ensure_ref_resolves(root, &member_of)?;

    Ok(ChangeRow {
        target: None,
        action: args.action,
        primary: args.primary,
        from: None,
        to_status: None,
        new_label: Some(new_label),
        member_of: Some(member_of),
        new_statement: args.new_statement.clone(),
        allocated: None,
    })
}

/// An existing-target op (`modify`/`retire`/`move`/`status`): keys on a live `--target`
/// FK, validated against the `revises` rule (target kind ∈ {SPEC,PRD,REQ,ADR,POL,STD},
/// off-target refused) + existence. For `status`: `--to-status` REQUIRED and `from`
/// AUTO-captured from the target's CURRENT `ReqStatus` (the §4.5 from-guard input).
/// OQ-1 dedup: a second row of the same `(action, target)` is refused.
fn build_existing_target_row(
    root: &Path,
    args: &ChangeAddArgs,
    existing: &[ChangeRow],
) -> anyhow::Result<ChangeRow> {
    anyhow::ensure!(
        args.new_label.is_none() && args.member_of.is_none(),
        "`--new-label` / `--member-of` are only valid for a creation op (`introduce`/`create`)"
    );
    let target = args.target.clone().with_context(|| {
        format!(
            "an existing-target op (`{}`) requires `--target` naming a live peer FK",
            args.action.as_str()
        )
    })?;

    // Validate the target against the `revises` rule: kind ∈ the six authored-truth
    // kinds (off-target — e.g. `revises SL-001` — refused), then existence.
    let (kref, _) = crate::integrity::parse_canonical_ref(&target)?;
    let rule = crate::relation::lookup(&REV_KIND, crate::relation::RelationLabel::Revises)
        .context("internal: missing `revises` rule row")?;
    crate::relation::check_target_kind(rule, &REV_KIND, kref.kind.prefix)?;
    crate::integrity::ensure_ref_resolves(root, &target)?;

    // OQ-1 — the change is named once per target: refuse a second row of the same
    // (action, target). (Against the table shape: existing-target ops key on the FK, so
    // two `status` rows for one REQ are contradictory; the rule generalises to every
    // existing-target action.)
    anyhow::ensure!(
        !existing
            .iter()
            .any(|r| r.action == args.action && r.target.as_deref() == Some(target.as_str())),
        "this revision already carries a `{}` change for {target} (OQ-1: a change is named once)",
        args.action.as_str()
    );

    // `status` rows: auto-capture `from` from the target's current ReqStatus; require
    // `--to-status`. Only REQ carries a ReqStatus, so a `status` row targets a REQ.
    let (from, to_status) = if args.action == ChangeAction::Status {
        anyhow::ensure!(
            kref.kind.prefix == "REQ",
            "a `status` change targets a requirement (REQ), got a {}",
            kref.kind.prefix
        );
        let to = args
            .to_status
            .clone()
            .context("a `status` change requires `--to-status`")?;
        let current = crate::requirement::load(root, &target)?.status;
        (Some(current.as_str().to_owned()), Some(to))
    } else {
        anyhow::ensure!(
            args.to_status.is_none(),
            "`--to-status` is only valid for a `status` row"
        );
        (None, None)
    };

    Ok(ChangeRow {
        target: Some(target),
        action: args.action,
        primary: args.primary,
        from,
        to_status,
        new_label: None,
        member_of: None,
        new_statement: args.new_statement.clone(),
        allocated: None,
    })
}

/// Read the REV's existing `[[change]]` rows (for the primary + dedup guards). An
/// absent payload (a fresh REV) reads as empty (the read-tolerant convention).
fn read_change_rows(rev_root: &Path, id: u32) -> anyhow::Result<Vec<ChangeRow>> {
    let name = format!("{id:03}");
    let path = rev_root.join(&name).join(format!("revision-{name}.toml"));
    let text = fs::read_to_string(&path)
        .with_context(|| format!("revision {name} not found at {}", path.display()))?;
    let doc: ChangeDoc =
        toml::from_str(&text).with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(doc.change)
}

/// Append one `[[change]]` row to `revision-NNN.toml` edit-preservingly (the
/// `toml_edit::DocumentMut` in-place idiom — the `[relationships]` block, comments,
/// `approval`, and unknown keys all survive; never a parse→serialise round-trip). Every
/// user free-text value (`new_statement`) rides `toml_string`; structured fields are
/// closed vocab / validated refs, spliced as bare literals. The `[[change]]` array is
/// CREATED if absent (unlike `[relationships]`, it is not scaffold-seeded — a fresh REV
/// carries none).
fn append_change_row(rev_root: &Path, id: u32, row: &ChangeRow) -> anyhow::Result<()> {
    let name = format!("{id:03}");
    let path = rev_root.join(&name).join(format!("revision-{name}.toml"));
    let text = fs::read_to_string(&path)
        .with_context(|| format!("revision {name} not found at {}", path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", path.display()))?;

    let array = doc
        .entry("change")
        .or_insert_with(|| toml_edit::Item::ArrayOfTables(toml_edit::ArrayOfTables::new()))
        .as_array_of_tables_mut()
        .context("malformed revision: `change` is present but is not an array-of-tables")?;

    let mut table = toml_edit::Table::new();
    if let Some(t) = &row.target {
        table.insert("target", toml_edit::value(t.as_str()));
    }
    table.insert("action", toml_edit::value(row.action.as_str()));
    table.insert("primary", toml_edit::value(row.primary));
    if let Some(f) = &row.from {
        table.insert("from", toml_edit::value(f.as_str()));
    }
    if let Some(to) = &row.to_status {
        table.insert("to_status", toml_edit::value(to.as_str()));
    }
    if let Some(l) = &row.new_label {
        table.insert("new_label", toml_edit::value(l.as_str()));
    }
    if let Some(m) = &row.member_of {
        table.insert("member_of", toml_edit::value(m.as_str()));
    }
    if let Some(s) = &row.new_statement {
        table.insert("new_statement", toml_edit::value(s.as_str()));
    }
    array.push(table);

    fs::write(&path, doc.to_string()).with_context(|| format!("Failed to write {}", path.display()))
}

// ---------------------------------------------------------------------------
// CLI: `revision approve` — the orthogonal approval flip (design §4.2). Sets
// `approval = approved` edit-preservingly (the orthogonal field; `status` untouched).
// This is the apply-time forcing-function checkpoint's enabling act — NOT
// actor-attributed authz (ADR-009 §invoker-blind: a solo dev self-approves). A
// missing REV hard-errors (read fails); never an implicit create.
// ---------------------------------------------------------------------------

/// `doctrine revision approve <REV-N>` — record an explicit approval on the orthogonal
/// `approval` axis (sets `approval = approved`), edit-preservingly. `status` is NEVER
/// touched (orthogonal — approval is not lifecycle, ADR-009). This is the enabling act
/// for the apply checkpoint (§4.2): `revision apply` REFUSES unless `approval =
/// approved`. Invoker-blind — it records THAT an approval happened, not WHO.
pub(crate) fn run_approve(path: Option<PathBuf>, reference: &str) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let rev_root = root.join(REV_DIR);
    let id = parse_ref(reference)?;
    // The REV must exist (read fails otherwise — never an implicit create).
    let doc = read_revision(&rev_root, id)?;
    set_revision_approval(&rev_root, id, Approval::Approved, &crate::clock::today())?;
    writeln!(
        io::stdout(),
        "{} approval: {} → approved",
        canonical_id(id),
        doc.approval.as_str()
    )?;
    Ok(())
}

/// Edit-preserving approval transition on one authored `revision-NNN.toml` — the
/// `set_revision_status` precedent, but over the orthogonal `approval` key (plus the
/// `updated` stamp). Delegates to the shared `dep_seq::set_authored_status` seam, so
/// the `[relationships]` table, the `[[change]]` payload, comments, `status`, and
/// unknown keys all survive (the file is never reserialised). Leaves `status`
/// untouched (orthogonal axis).
fn set_revision_approval(
    rev_root: &Path,
    id: u32,
    approval: Approval,
    today: &str,
) -> anyhow::Result<()> {
    let name = format!("{id:03}");
    let path = rev_root.join(&name).join(format!("revision-{name}.toml"));
    let hint = format!(
        "malformed revision {name}: missing seeded `approval`/`updated` (regenerate via `revision new`)"
    );
    crate::dep_seq::set_authored_status(
        &path,
        &[("approval", approval.as_str()), ("updated", today)],
        &hint,
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// CLI: `revision apply` — auto-land `status` rows, surface the rest for manual
// (design §4.5). v1 auto-lands ONLY rows whose action is `status`: each rides the
// engine-callable `requirement::set_status` + composes one `RecDoc` (REC untouched,
// `owning_slice = None`). introduce/create/modify/move + ADR/POL/STD prose rows are
// surfaced-for-manual (listed, never auto-applied — only `status` has an engine seam).
// Apply REFUSES unless `approval = approved` (the §4.2 checkpoint). A pre-flight
// all-or-nothing from-guard sweep reads the CURRENT ReqStatus for every status row and
// aborts the WHOLE apply if any `current != row.from` (or the target is missing) —
// writing nothing (never silently clobbers an intervening reconcile move). Terminal
// disposition (§4.2): status-only → `done` (dependents unblock); a REV also carrying
// surfaced-for-manual rows stays `started` (status landed, manual list printed) until
// the operator completes them and marks `done` by hand (done never lies).
// ---------------------------------------------------------------------------

/// One status row resolved for apply — the parsed `to_status` plus the requirement's
/// numeric id and FK. PURE-derived in the pre-flight, consumed by the writer.
struct PlannedStatus {
    fk: String,
    req_id: u32,
    to: ReqStatus,
}

/// A from-guard staleness finding (pure): the status row whose stored `from` no longer
/// matches the target's current `ReqStatus` (the target moved since the change was
/// drafted), or whose target cannot be resolved. Surfaced as the stale set; ANY entry
/// aborts the whole apply before the first write.
struct StaleFinding {
    fk: String,
    expected_from: String,
    actual: String,
}

/// Parse a `ReqStatus` from its kebab string (the inverse of `as_str`), via serde over
/// the closed enum — no hand-maintained match that could drift from the variants. Used
/// to turn a stored `to_status` / a current-status string into the typed status the
/// setter takes. Pure.
fn parse_req_status(s: &str) -> anyhow::Result<ReqStatus> {
    serde_json::from_value::<ReqStatus>(serde_json::Value::String(s.to_owned()))
        .with_context(|| format!("not a known requirement status: `{s}`"))
}

/// Partition the REV's change rows into the auto-landable `status` set and the
/// surfaced-for-manual remainder (PURE over the rows). Only `status` rows have a v1
/// engine seam; everything else (introduce/create/modify/move + ADR/POL/STD prose) is
/// surfaced-for-manual.
fn partition_change_rows(rows: &[ChangeRow]) -> (Vec<&ChangeRow>, Vec<&ChangeRow>) {
    rows.iter().partition(|r| r.action == ChangeAction::Status)
}

/// `doctrine revision apply <REV-N>` — land the approved change rows (design §4.5).
///
/// The shell: resolve root, read the REV, enforce the approval checkpoint, partition
/// the rows, run the pre-flight all-or-nothing from-guard sweep over the `status` rows
/// (existence + `from` vs current `ReqStatus`), then — only if the sweep is clean —
/// write each status row (`set_status` + one `RecDoc`), print the surfaced-for-manual
/// list, and settle the terminal disposition. All git/disk live here; the from-guard
/// classification + `RecDoc` composition are pure over the resolved inputs (mirrors
/// reconcile.rs).
pub(crate) fn run_apply(path: Option<PathBuf>, reference: &str) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let rev_root = root.join(REV_DIR);
    let id = parse_ref(reference)?;
    let doc = read_revision(&rev_root, id)?;

    // The apply-time forcing-function checkpoint (§4.2 / E3): refuse unless an explicit
    // approval act has been recorded. Invoker-blind — not actor-attributed authz.
    anyhow::ensure!(
        doc.approval == Approval::Approved,
        "{} is not approved (approval = `{}`) — run `doctrine revision approve {}` before apply",
        canonical_id(id),
        doc.approval.as_str(),
        canonical_id(id)
    );

    let rows = read_change_rows(&rev_root, id)?;
    let (status_rows, manual_rows) = partition_change_rows(&rows);

    // Pre-flight all-or-nothing from-guard sweep (§4.5, F7→E1): read CURRENT ReqStatus
    // for every status row (existence + from-guard), THEN write. Any refusal here aborts
    // the WHOLE apply before the first write — never a partial land, never a silent
    // clobber of an intervening reconcile move. Builds the resolved write plan as it
    // sweeps; collects every stale/missing finding so the operator sees the full set.
    let mut planned: Vec<PlannedStatus> = Vec::new();
    let mut stale: Vec<StaleFinding> = Vec::new();
    for row in &status_rows {
        let fk = row
            .target
            .clone()
            .context("internal: a `status` change row carries no target")?;
        let expected_from = row
            .from
            .clone()
            .context("internal: a `status` change row carries no captured `from`")?;
        let to_str = row
            .to_status
            .clone()
            .context("internal: a `status` change row carries no `to_status`")?;
        // Parse `to_status` up front — an unknown target status is a hard refusal BEFORE
        // any write (still all-or-nothing).
        let to = parse_req_status(&to_str)?;
        // Existence + current-status read (the staleness compare). A missing target is a
        // stale finding too (the target vanished since draft).
        match crate::requirement::load(&root, &fk) {
            Ok(req) => {
                let actual = req.status.as_str().to_owned();
                if actual == expected_from {
                    let req_id = crate::requirement::id_from_fk(&fk)?;
                    planned.push(PlannedStatus { fk, req_id, to });
                } else {
                    stale.push(StaleFinding {
                        fk: fk.clone(),
                        expected_from,
                        actual,
                    });
                }
            }
            Err(_) => stale.push(StaleFinding {
                fk: fk.clone(),
                expected_from,
                actual: "missing".to_owned(),
            }),
        }
    }

    // Abort the WHOLE apply on ANY stale finding — surface the full set, write nothing.
    if !stale.is_empty() {
        use std::fmt::Write as _;
        let mut msg = String::from(
            "apply aborted — the requirement status drifted since the change was drafted (all-or-nothing; nothing written):",
        );
        for f in &stale {
            write!(
                msg,
                "\n  {} expected from `{}`, found `{}` — re-draft the status change",
                f.fk, f.expected_from, f.actual
            )?;
        }
        anyhow::bail!(msg);
    }

    let mut out = io::stdout();

    // Land the status rows: each a `set_status` write + one self-describing RecDoc
    // (owning_slice = None — a standalone, non-slice-close status change). One commit
    // carries the N status edits + N RecDocs (the operator commits; this writes the
    // worktree like reconcile.rs). REC schema untouched.
    for p in &planned {
        let prior = crate::requirement::load(&root, &p.fk)?.status;
        let rec = compose_apply_rec(&p.fk, prior, p.to);
        let rec_id = crate::rec::materialise_populated(&root, &rec)?; // WAL-first (NF-003)
        crate::requirement::set_status(&root, p.req_id, p.to)?;
        writeln!(
            out,
            "{}: status {} → {} (rec {rec_id:03})",
            p.fk,
            prior.as_str(),
            p.to.as_str()
        )?;
    }

    // Surface-for-manual: list the rows with no v1 engine seam (NOT auto-applied).
    if !manual_rows.is_empty() {
        writeln!(
            out,
            "\nsurfaced for manual handling ({} row(s) — land by operator hand-edit):",
            manual_rows.len()
        )?;
        for row in &manual_rows {
            let subject = row
                .target
                .clone()
                .or_else(|| row.member_of.clone())
                .unwrap_or_default();
            writeln!(out, "  {} {}", row.action.as_str(), subject)?;
        }
    }

    // Terminal disposition (§4.2 / E5 / M1): a status-only REV → `done` (dependents
    // unblock). A REV ALSO carrying surfaced-for-manual rows stays `started` (status
    // landed, manual list printed) — `done` never lies (every row landed). Drive the
    // REV FSM through legal steps toward the target.
    let target = if manual_rows.is_empty() {
        RevStatus::Done
    } else {
        RevStatus::Started
    };
    settle_disposition(&rev_root, id, doc.status, target)?;
    writeln!(out, "{} → {}", canonical_id(id), target.as_str())?;
    Ok(())
}

/// Compose the standalone-apply `RecDoc` for one landed status row (PURE) — the
/// reconcile.rs `compose_status_rec` shape, but `owning_slice = None` (a standalone,
/// non-slice-close status change, §4.6) and NO evidence (apply rests on the approved
/// Revision, not a coverage scan — REC's `evidence_ref` is empty for this subset). The
/// `id` is a placeholder; the engine assigns the reserved id at materialise. REC schema
/// is untouched — one `[[status_delta]]`, an empty `[[evidence_ref]]`.
fn compose_apply_rec(req: &str, prior: ReqStatus, written: ReqStatus) -> crate::rec::RecDoc {
    crate::rec::RecDoc {
        id: 0,
        slug: format!("apply-{}", req.to_lowercase()),
        title: format!("apply {req}"),
        rec: crate::rec::RecMeta {
            r#move: crate::rec::RecMove::Revise.as_str().to_owned(),
            owning_slice: None,
            decision_ref: None,
        },
        status_delta: vec![crate::rec::StatusDelta {
            requirement: req.to_owned(),
            from: prior.as_str().to_owned(),
            to: written.as_str().to_owned(),
        }],
        evidence_ref: Vec::new(),
    }
}

/// Drive the REV lifecycle from `current` toward `target` (`done` or `started`) through
/// LEGAL FSM steps (`proposed → started → done`), each gated by `validate_transition`.
/// Idempotent: a REV already at/past the target writes nothing (the no-op guard in the
/// setter holds content + mtime). Used by apply's terminal disposition (§4.2).
fn settle_disposition(
    rev_root: &Path,
    id: u32,
    current: RevStatus,
    target: RevStatus,
) -> anyhow::Result<()> {
    let today = crate::clock::today();
    // The path proposed → started → done; stop at `target`. Skip any step already at or
    // beyond it. (A terminal current is left untouched — apply on a terminal REV is a
    // no-op disposition; the FSM would refuse leaving it anyway.)
    if current.is_terminal() {
        return Ok(());
    }
    let steps: &[RevStatus] = match target {
        RevStatus::Started => &[RevStatus::Started],
        RevStatus::Done => &[RevStatus::Started, RevStatus::Done],
        _ => &[],
    };
    let mut from = current;
    for &step in steps {
        if from == step {
            continue;
        }
        validate_transition(from, step)?;
        set_revision_status(rev_root, id, step, &today)?;
        from = step;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Drift canary (the backlog.rs `*_matches_the_variants` pattern): every
    /// `RevStatus` variant's `as_str` is in the known-set and vice versa, in order.
    #[test]
    fn rev_statuses_matches_the_variants() {
        let variants = [
            RevStatus::Proposed,
            RevStatus::Started,
            RevStatus::Done,
            RevStatus::Abandoned,
        ];
        let from_variants: Vec<&str> = variants.iter().map(|s| s.as_str()).collect();
        assert_eq!(
            from_variants, REV_STATUSES,
            "REV_STATUSES drifted from the RevStatus variants"
        );
    }

    /// Drift canary: every `ChangeAction` variant's `as_str` is in the known-set and
    /// vice versa, in order (the `*_matches_the_variants` pattern).
    #[test]
    fn change_actions_matches_the_variants() {
        let variants = [
            ChangeAction::Modify,
            ChangeAction::Retire,
            ChangeAction::Move,
            ChangeAction::Status,
            ChangeAction::Introduce,
            ChangeAction::Create,
        ];
        let from_variants: Vec<&str> = variants.iter().map(|a| a.as_str()).collect();
        assert_eq!(
            from_variants, CHANGE_ACTIONS,
            "CHANGE_ACTIONS drifted from the ChangeAction variants"
        );
    }

    /// `is_creation` partitions the vocabulary: exactly `introduce`/`create` are
    /// creation ops; the existing-target ops key on a live FK.
    #[test]
    fn is_creation_partitions_the_actions() {
        assert!(ChangeAction::Introduce.is_creation());
        assert!(ChangeAction::Create.is_creation());
        for a in [
            ChangeAction::Modify,
            ChangeAction::Retire,
            ChangeAction::Move,
            ChangeAction::Status,
        ] {
            assert!(!a.is_creation(), "{a:?} is an existing-target op");
        }
    }

    /// A `[[change]]` row's `edge_target` is its FK (existing-target) or its
    /// destination `member_of` SPEC (creation) — the projection `relation_edges` reads.
    #[test]
    fn change_row_edge_target_picks_fk_then_member_of() {
        let existing = ChangeRow {
            target: Some("ADR-006".to_owned()),
            action: ChangeAction::Modify,
            primary: false,
            from: None,
            to_status: None,
            new_label: None,
            member_of: None,
            new_statement: None,
            allocated: None,
        };
        assert_eq!(existing.edge_target(), Some("ADR-006"));
        let creation = ChangeRow {
            target: None,
            action: ChangeAction::Introduce,
            primary: false,
            from: None,
            to_status: None,
            new_label: Some("FR-007".to_owned()),
            member_of: Some("SPEC-018".to_owned()),
            new_statement: None,
            allocated: None,
        };
        assert_eq!(creation.edge_target(), Some("SPEC-018"));
    }

    /// The `[[change]]` table parses both row shapes from one array (the read-tolerant
    /// targeted parse — every other key in the file is ignored).
    #[test]
    fn change_doc_parses_both_row_shapes() {
        let text = r#"
id = 1
status = "started"
[[change]]
target = "REQ-201"
action = "status"
primary = false
from = "active"
to_status = "retired"
[[change]]
action = "introduce"
member_of = "SPEC-018"
new_label = "FR-007"
primary = true
"#;
        let doc: ChangeDoc = toml::from_str(text).unwrap();
        assert_eq!(doc.change.len(), 2);
        assert_eq!(doc.change[0].action, ChangeAction::Status);
        assert_eq!(doc.change[0].from.as_deref(), Some("active"));
        assert_eq!(doc.change[1].action, ChangeAction::Introduce);
        assert!(doc.change[1].primary);
        assert_eq!(doc.change[1].new_label.as_deref(), Some("FR-007"));
    }

    /// Drift canary for the orthogonal approval axis.
    #[test]
    fn approvals_matches_the_variants() {
        let variants = [
            Approval::None,
            Approval::Requested,
            Approval::Approved,
            Approval::Rejected,
        ];
        let from_variants: Vec<&str> = variants.iter().map(|a| a.as_str()).collect();
        assert_eq!(
            from_variants, APPROVALS,
            "APPROVALS drifted from the Approval variants"
        );
    }

    /// The schema round-trips through serde: id/slug/title/status/approval survive
    /// toml → struct → toml, and the closed enums render kebab.
    #[test]
    fn schema_round_trips_through_serde() {
        let doc = RevDoc {
            id: 7,
            slug: "revise-adr-006".to_owned(),
            title: "revise ADR-006".to_owned(),
            status: RevStatus::Started,
            approval: Approval::None,
        };
        let text = toml::to_string(&doc).unwrap();
        assert!(
            text.contains("status = \"started\""),
            "kebab status: {text}"
        );
        assert!(
            text.contains("approval = \"none\""),
            "kebab approval: {text}"
        );
        let back: RevDoc = toml::from_str(&text).unwrap();
        assert_eq!(back, doc);
    }

    // -- FSM: the REV-local legal-transition guard --------------------------

    #[test]
    fn fsm_advances_proposed_started_done() {
        assert!(validate_transition(RevStatus::Proposed, RevStatus::Started).is_ok());
        assert!(validate_transition(RevStatus::Started, RevStatus::Done).is_ok());
    }

    #[test]
    fn fsm_abandons_from_any_non_terminal() {
        assert!(validate_transition(RevStatus::Proposed, RevStatus::Abandoned).is_ok());
        assert!(validate_transition(RevStatus::Started, RevStatus::Abandoned).is_ok());
    }

    #[test]
    fn fsm_refuses_leaving_a_terminal_source() {
        assert!(validate_transition(RevStatus::Done, RevStatus::Started).is_err());
        assert!(validate_transition(RevStatus::Abandoned, RevStatus::Started).is_err());
        // even terminal → terminal is refused (leaving a terminal source).
        assert!(validate_transition(RevStatus::Done, RevStatus::Abandoned).is_err());
    }

    #[test]
    fn fsm_refuses_a_skip() {
        // proposed → done skips `started` — refused.
        let err =
            validate_transition(RevStatus::Proposed, RevStatus::Done).expect_err("skip refused");
        assert!(
            format!("{err}").contains("illegal"),
            "names the illegal move: {err}"
        );
    }

    #[test]
    fn fsm_allows_idempotent_no_op() {
        // from == to is allowed (idempotent), even on a terminal status — the
        // value does not change, so no terminal-leave occurs.
        assert!(validate_transition(RevStatus::Started, RevStatus::Started).is_ok());
        assert!(validate_transition(RevStatus::Done, RevStatus::Done).is_ok());
    }

    #[test]
    fn is_terminal_marks_done_and_abandoned() {
        assert!(RevStatus::Done.is_terminal());
        assert!(RevStatus::Abandoned.is_terminal());
        assert!(!RevStatus::Proposed.is_terminal());
        assert!(!RevStatus::Started.is_terminal());
    }

    /// Hostile free-text in a title rides `toml_string`: a `"` / newline cannot
    /// break the rendered document or inject a key (the rec.rs precedent).
    #[test]
    fn render_escapes_a_hostile_title() {
        let text = render_revision_toml(1, "s", "T\"\ninjected = \"x", "2026-06-14").unwrap();
        // The document still parses (the breaker was escaped, not spliced raw) …
        let back: RevDoc = toml::from_str(&text).unwrap();
        // … and the hostile value round-trips verbatim, no injected key.
        assert_eq!(back.title, "T\"\ninjected = \"x");
        assert_eq!(back.status, RevStatus::Proposed, "seeded proposed");
        assert_eq!(back.approval, Approval::None, "seeded none");
    }

    // -- PHASE-05 apply pure helpers ----------------------------------------

    /// `parse_req_status` is the exact inverse of `ReqStatus::as_str` over every
    /// variant (round-trips the kebab strings), and rejects an unknown token — so a
    /// stored `to_status` / current-status string maps back to the typed status the
    /// setter takes, with no hand-maintained match to drift.
    #[test]
    fn parse_req_status_round_trips_and_rejects_unknown() {
        for s in crate::requirement::REQ_STATUSES {
            let parsed = parse_req_status(s).expect("known status parses");
            assert_eq!(parsed.as_str(), *s, "round-trips `{s}`");
        }
        assert!(
            parse_req_status("not-a-status").is_err(),
            "an unknown status token is refused"
        );
    }

    /// `partition_change_rows` auto-lands ONLY `status` rows; every other action
    /// (creation/modify/move) is surfaced-for-manual (the v1 engine-seam boundary).
    #[test]
    fn partition_splits_status_from_surfaced_for_manual() {
        let row = |action: ChangeAction, target: &str| ChangeRow {
            target: Some(target.to_owned()),
            action,
            primary: false,
            from: None,
            to_status: None,
            new_label: None,
            member_of: None,
            new_statement: None,
            allocated: None,
        };
        let rows = vec![
            row(ChangeAction::Status, "REQ-201"),
            row(ChangeAction::Modify, "ADR-006"),
            row(ChangeAction::Move, "REQ-202"),
            row(ChangeAction::Status, "REQ-203"),
        ];
        let (status, manual) = partition_change_rows(&rows);
        assert_eq!(status.len(), 2, "only the two status rows auto-land");
        assert!(status.iter().all(|r| r.action == ChangeAction::Status));
        assert_eq!(manual.len(), 2, "modify + move are surfaced-for-manual");
        assert!(manual.iter().all(|r| r.action != ChangeAction::Status));
    }

    /// `compose_apply_rec` mirrors the reconcile status-REC shape but is STANDALONE:
    /// `owning_slice = None` (a non-slice-close status change, §4.6), one
    /// `[[status_delta]]` carrying the from→to move, and NO evidence (apply rests on the
    /// approved Revision, not a coverage scan). REC schema untouched.
    #[test]
    fn compose_apply_rec_is_standalone_with_one_delta_no_evidence() {
        let rec = compose_apply_rec("REQ-201", ReqStatus::Active, ReqStatus::Retired);
        assert_eq!(
            rec.rec.owning_slice, None,
            "standalone: owning_slice = None"
        );
        assert_eq!(rec.status_delta.len(), 1, "one status_delta");
        let d = &rec.status_delta[0];
        assert_eq!(
            (d.requirement.as_str(), d.from.as_str(), d.to.as_str()),
            ("REQ-201", "active", "retired")
        );
        assert!(
            rec.evidence_ref.is_empty(),
            "apply rests on the approved REV, not coverage — no evidence"
        );
    }
}
