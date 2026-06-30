// SPDX-License-Identifier: GPL-3.0-only
//! The priority subsystem (SL-047) — the cross-kind work-priority adapter.
//!
//! Sits at the engine layer above `relation_graph` (ADR-001): it consumes
//! `relation_graph`'s `pub(crate)` all-kind scan seam ([`crate::relation_graph::
//! scan_entities`]) to build a THIRD cordage `Graph` (distinct from
//! `backlog_order`'s and `inspect`'s — they share only the `Projection` TYPE),
//! carrying the dep/seq overlays, per-node attributes, a consequence pre-pass tally,
//! and an `OrderSpec`. PHASE-02 adds the pure policy core: [`partition`] (the OQ-8
//! status-class table) and [`channels`] (eligibility / blockers / actionable /
//! consequence / order-key / dep-cycle synthesis derived over a `PriorityGraph`).
//! PHASE-03 adds the operator-facing layer: [`view`] (the structured-reason render
//! source of truth), [`surface`] (the impure shell that builds the view rows from the
//! graph + channels), and [`render`] (human + `--json`). The `main.rs` command layer
//! consumes [`surface`]/[`render`] for the `survey`/`next`/`blockers`/`explain` verbs
//! and the `inspect` actionability block — the live caller that retires the
//! PHASE-01/02 self-clearing `dead_code` scopes.
pub(crate) mod channels;
pub(crate) mod config;
pub(crate) mod graph;
pub(crate) mod partition;
pub(crate) mod render;
pub(crate) mod surface;
pub(crate) mod view;

use std::io::{self, Write};
use std::path::PathBuf;

use crate::listing::{Format, RenderOpts};

/// The default `--limit` for `doctrine next` (SL-171 PHASE-02).
pub(crate) const NEXT_LIMIT_DEFAULT: usize = 20;

/// The default `--limit` for `doctrine survey` (IMP-218).
pub(crate) const SURVEY_LIMIT_DEFAULT: usize = 20;

/// Resolve the project root (default markers), shared by every priority verb.
fn root(path: Option<PathBuf>) -> anyhow::Result<std::path::PathBuf> {
    crate::root::find(path, &crate::root::default_markers())
}

/// Validate `--page`/`--limit`/`--offset` and resolve to a concrete offset.
/// Single source for both `Survey` and `Next` dispatch in `cli.rs` (IMP-218 DRY
/// extraction).
pub(crate) fn resolve_page_offset(
    page: Option<usize>,
    limit: usize,
    offset: usize,
) -> anyhow::Result<usize> {
    if page == Some(0) {
        anyhow::bail!("--page must be >= 1");
    }
    if limit == 0 && page.is_some() {
        anyhow::bail!("--page requires a positive --limit");
    }
    Ok(match page {
        Some(p) => (p - 1) * limit,
        None => offset,
    })
}

/// `doctrine survey [--all] [--hide-blocked] [--json] [--limit N] [--offset N] [--page N]`
/// (design §5.4 / IMP-218) — the importance survey with pagination. Builds the rows
/// once via [`surface::survey`] and renders per `Format` (`--json` forces JSON).
/// Pagination args are ignored under `--json`.
#[expect(
    clippy::too_many_arguments,
    reason = "CLI surface: path+all+hide_blocked+format+json+render+limit+offset = 8 — each is independently meaningful, no natural sub-struct"
)]
pub(crate) fn run_survey(
    path: Option<PathBuf>,
    all: bool,
    hide_blocked: bool,
    format: Format,
    json: bool,
    render: RenderOpts,
    limit: usize,
    offset: usize,
) -> anyhow::Result<()> {
    let root = root(path)?;
    let rows = surface::survey(&root, all, hide_blocked)?;
    let out = if json || format == Format::Json {
        render::survey_json(&rows)?
    } else {
        render::survey_human(&rows, render, limit, offset)
    };
    write!(io::stdout(), "{out}")?;
    Ok(())
}

/// `doctrine next [--json] [--columns <CSV>] [--limit N] [--offset N] [--page N]`
/// (design §5.4) — the actionable-only advisory worklist. `columns` is the
/// `--columns` projection; `limit`/`offset` are the pagination slice
/// (ignored under `--json`).
pub(crate) fn run_next(
    path: Option<PathBuf>,
    format: Format,
    json: bool,
    render: RenderOpts,
    columns: Option<&Vec<String>>,
    limit: usize,
    offset: usize,
) -> anyhow::Result<()> {
    let root = root(path)?;
    let rows = surface::next(&root)?;
    let out = if json || format == Format::Json {
        render::next_json(&rows)?
    } else {
        render::next_human(&rows, render, columns.map(Vec::as_slice), limit, offset)?
    };
    write!(io::stdout(), "{out}")?;
    Ok(())
}

/// `doctrine blockers <ID> [--transitive] [--json]` (design §5.4). An unknown prefix
/// / malformed ref surfaces a clean `anyhow` error (never a panic).
pub(crate) fn run_blockers(
    path: Option<PathBuf>,
    id: &str,
    transitive: bool,
    format: Format,
    json: bool,
    _render: RenderOpts,
) -> anyhow::Result<()> {
    let root = root(path)?;
    let view = surface::blockers(&root, id, transitive)?;
    let out = if json || format == Format::Json {
        render::blockers_json(&view)?
    } else {
        render::blockers_human(&view)
    };
    write!(io::stdout(), "{out}")?;
    Ok(())
}

/// `doctrine explain <ID> [--json]` (design §5.4 / D11) — always to root.
pub(crate) fn run_explain(
    path: Option<PathBuf>,
    id: &str,
    format: Format,
    json: bool,
    _render: RenderOpts,
) -> anyhow::Result<()> {
    let root = root(path)?;
    let ex = surface::explain(&root, id)?;
    let out = if json || format == Format::Json {
        render::explain_json(&ex)?
    } else {
        render::explain_human(&ex)
    };
    write!(io::stdout(), "{out}")?;
    Ok(())
}
