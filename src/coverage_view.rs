// SPDX-License-Identifier: GPL-3.0-only
//! `coverage_view` — the pure compute + render for `doctrine coverage <ref>`
//! (SL-045 PHASE-03, the read's view layer).
//!
//! This is the read half of the coverage surface: given a `REQ-`/`PRD-`/`SPEC-`
//! ref, it materialises one [`CoverageRow`] per requirement and renders the row
//! set as a text table or a JSON envelope. The orchestrating [`rows`] is the only
//! disk seam — it dispatches the ref, fans a spec into its members
//! ([`crate::spec::member_reqs`]), batches the coverage scan
//! ([`crate::coverage_scan::scan_coverage_batch`]), and resolves each member's
//! authored requirement ([`crate::requirement::load`]).
//!
//! **Two stores, two reads, one wall (NF-001 / ADR-009 §3, F1).** A row's authored
//! [`ReqStatus`] comes ONLY from `requirement::load`; the coverage fold feeds the
//! observed/verdict columns and NEVER the `status` column. There is no data edge
//! from the fold into authored status — the wall sits at the row constructor.
//!
//! **Layering (ADR-001).** This module imports `spec`; `spec` must never import
//! `coverage_view` — no back-edge, no cycle.
//!
//! **Independent observed partition (E6).** [`ObservedState`] is a TOTAL, lossy
//! partition over the five [`CoverageStatus`](crate::requirement::CoverageStatus)
//! states via [`Composite`]'s four predicates — it is NOT derived from `drift`, and
//! is deliberately not asserted 1:1 against the verdict (the two answer different
//! questions: "what was observed" vs "does authored cohere with observed").

// The whole view layer is a leaf built ahead of its consumer: this phase lands the
// compute + render; main.rs wires `doctrine coverage <ref>` in a later phase. Until
// then every item is dead in the bins/lib build, so the module carries a
// self-clearing `not(test)` dead_code expect (the `dead-code-self-clearing-leaf`
// precedent, mirroring coverage.rs). Under `cfg(test)` the VTs exercise every item,
// so `dead_code` would not fire and an unconditional `expect` would be unfulfilled;
// it scopes to `not(test)` where the gate's plain `cargo clippy` (bins/lib, no test
// cfg) sees the items as genuinely dead. It retires itself when main.rs wires the
// `coverage` verb.
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "coverage view layer (SL-045 PHASE-03) is a leaf built ahead of \
                  its main.rs `coverage` verb consumer — every item is dead in the \
                  bins/lib build until that verb is wired"
    )
)]

use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::coverage::{self, Composite, Verdict};
use crate::coverage_scan::scan_coverage_batch;
use crate::listing;
use crate::requirement::{self, ReqKind, ReqStatus};
use crate::spec::member_reqs;

// ---------------------------------------------------------------------------
// Ref dispatch (VT-3)
// ---------------------------------------------------------------------------

/// The resolved subject of a `coverage <ref>` read: a single requirement, or a
/// spec whose members fan into a requirement set. The `String` is the raw ref —
/// canonicalisation happens downstream ([`requirement::canonicalize_fk`] /
/// [`member_reqs`]), so dispatch stays a pure prefix classification.
enum Target {
    Req(String),
    Spec(String),
}

/// Classify a `coverage <ref>` argument by its leading `PREFIX-` token: `REQ-` is
/// a single requirement; `PRD-`/`SPEC-` is a spec fan; anything else errors. The
/// three prefixes are derived from the ref's own leading token — not hardcoded
/// beyond this closed set.
fn dispatch(reference: &str) -> anyhow::Result<Target> {
    let prefix = reference.split('-').next().unwrap_or("");
    match prefix {
        "REQ" => Ok(Target::Req(reference.to_owned())),
        "PRD" | "SPEC" => Ok(Target::Spec(reference.to_owned())),
        _ => anyhow::bail!("`{reference}` is not a coverage ref (expected REQ-/PRD-/SPEC-NNN)"),
    }
}

// ---------------------------------------------------------------------------
// ObservedState — the independent lossy partition (EX-2, E6, VT-1)
// ---------------------------------------------------------------------------

/// The observed-evidence summary for one requirement — a TOTAL, lossy partition
/// over the five [`CoverageStatus`](crate::requirement::CoverageStatus) states,
/// read straight off [`Composite`]'s predicates (NOT derived from `drift`, E6).
/// "What was observed", distinct from the verdict's "does authored cohere".
pub(crate) enum ObservedState {
    /// No contributing cells at all.
    None,
    /// Some cell contradicts (`Failed`) or is `Blocked`.
    Contradicted,
    /// Some cell is fresh-`Verified` — live confirming evidence.
    Verified,
    /// Every cell is still forward-intent (`Planned`/`InProgress`).
    Forward,
    /// Cells exist but none is fresh-verified / contradicting / wholly forward
    /// (the only-stale / mixed remainder).
    Stale,
}

impl ObservedState {
    /// The terse observed-state cell text (the table/JSON token). Pure.
    fn label(&self) -> &'static str {
        match self {
            ObservedState::None => "none",
            ObservedState::Contradicted => "contradicted",
            ObservedState::Verified => "verified",
            ObservedState::Forward => "forward",
            ObservedState::Stale => "stale",
        }
    }
}

/// Partition a [`Composite`] into its [`ObservedState`] — total over the five
/// coverage states via the four public predicates, in precedence order
/// (contradiction outranks confirmation outranks forward-intent; the remainder is
/// stale/mixed). Pure: the composite already carries resolved staleness.
pub(crate) fn observed_state(c: &Composite) -> ObservedState {
    if c.is_empty() {
        ObservedState::None
    } else if c.any_failed_or_blocked() {
        ObservedState::Contradicted
    } else if c.any_fresh_verified() {
        ObservedState::Verified
    } else if c.only_forward() {
        ObservedState::Forward
    } else {
        ObservedState::Stale
    }
}

// ---------------------------------------------------------------------------
// CoverageRow — an enum, not a struct (EX-1, E1)
// ---------------------------------------------------------------------------

/// One requirement's coverage row. An ENUM, not a struct (E1): a dangling member
/// FK has no kind/status/verdict to hold, so the `Dangling` arm carries only the
/// id + label + load error — never a fabricated [`ReqStatus`].
pub(crate) enum CoverageRow {
    /// A requirement that loaded: its authored kind/status (from
    /// [`requirement::load`] ONLY — the F1 wall) plus the observed partition and
    /// the drift verdict (from the coverage fold).
    Healthy {
        id: String,
        label: Option<String>,
        kind: ReqKind,
        status: ReqStatus,
        observed: ObservedState,
        verdict: Verdict,
    },
    /// A spec member whose requirement entity could not be loaded (a dangling FK).
    /// The fan continues past it (INV-4 / A2 / E5); it renders its `load_error`
    /// in the data cells rather than a fabricated status.
    Dangling {
        id: String,
        label: Option<String>,
        load_error: String,
    },
}

/// Materialise the coverage rows for `reference` — the orchestrating read (the
/// sole disk seam). Dispatches the ref, fans a spec into its ordered members (a
/// bare REQ is a one-member set with no label), batches the coverage scan over
/// the whole requirement set, then per member resolves the authored requirement
/// and builds a row.
///
/// **F1 wall:** a `Healthy` row's `status` comes ONLY from [`requirement::load`] —
/// never from the coverage fold. **Fan tolerance (INV-4):** a dangling member FK
/// (label present) becomes a [`CoverageRow::Dangling`] and the fan CONTINUES; a
/// bare single REQ (no label) propagates the load error (no fan to protect).
pub(crate) fn rows(root: &Path, reference: &str) -> anyhow::Result<Vec<CoverageRow>> {
    // (label, requirement) in surface order: a bare REQ is one unlabelled member;
    // a spec fans into its ordered, already-canonical members.
    let members: Vec<(Option<String>, String)> = match dispatch(reference)? {
        Target::Req(r) => vec![(None, requirement::canonicalize_fk(&r))],
        Target::Spec(s) => member_reqs(root, &s)?
            .into_iter()
            .map(|m| (Some(m.label), m.requirement))
            .collect(),
    };

    // ONE corpus walk + ONE git anchor over the whole requirement fan (INV-2).
    let wanted: BTreeSet<String> = members.iter().map(|(_, req)| req.clone()).collect();
    let scanned = scan_coverage_batch(root, &wanted);

    let mut out = Vec::with_capacity(members.len());
    for (label, req) in members {
        let cells = scanned.get(&req).map(Vec::as_slice).unwrap_or_default();
        let comp = coverage::composite(cells);
        match requirement::load(root, &req) {
            Ok(r) => out.push(CoverageRow::Healthy {
                id: req,
                label,
                kind: r.kind,
                // F1 WALL: authored status is read here and ONLY here — no edge
                // from `comp`/`drift` into this field.
                status: r.status,
                observed: observed_state(&comp),
                verdict: coverage::drift(r.status, &comp),
            }),
            // A spec-fan member that won't load is dangling — render the error,
            // keep fanning (INV-4 / A2 / E5).
            Err(e) if label.is_some() => out.push(CoverageRow::Dangling {
                id: req,
                label,
                load_error: e.to_string(),
            }),
            // A bare single REQ has no fan to protect — the load failure is fatal.
            Err(e) => return Err(e),
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Column model + render (mirrors spec.rs's SPEC_COLUMNS)
// ---------------------------------------------------------------------------

impl CoverageRow {
    /// The row's requirement id — present on both arms.
    fn id(&self) -> &str {
        match self {
            CoverageRow::Healthy { id, .. } | CoverageRow::Dangling { id, .. } => id,
        }
    }

    /// The row's sticky membership label, if any (a spec fan member) — present on
    /// both arms.
    fn label(&self) -> Option<&str> {
        match self {
            CoverageRow::Healthy { label, .. } | CoverageRow::Dangling { label, .. } => {
                label.as_deref()
            }
        }
    }

    /// The `kind` cell: the authored kind for a healthy row, the inline load-error
    /// note for a dangling one (NOT a fabricated kind).
    fn kind_cell(&self) -> String {
        match self {
            CoverageRow::Healthy { kind, .. } => kind.as_str().to_owned(),
            CoverageRow::Dangling { load_error, .. } => load_error.clone(),
        }
    }

    /// The `status` cell — authored status for a healthy row, else the load-error
    /// note. A dangling row never invents a `ReqStatus`.
    fn status_cell(&self) -> String {
        match self {
            CoverageRow::Healthy { status, .. } => status.as_str().to_owned(),
            CoverageRow::Dangling { load_error, .. } => load_error.clone(),
        }
    }

    /// The `observed` cell — the observed-state token, else the load-error note.
    fn observed_cell(&self) -> String {
        match self {
            CoverageRow::Healthy { observed, .. } => observed.label().to_owned(),
            CoverageRow::Dangling { load_error, .. } => load_error.clone(),
        }
    }

    /// The `verdict` cell — the verdict label, else the load-error note.
    fn verdict_cell(&self) -> String {
        match self {
            CoverageRow::Healthy { verdict, .. } => verdict.label(),
            CoverageRow::Dangling { load_error, .. } => load_error.clone(),
        }
    }

    /// The `label` cell — the sticky label, or `-` when this row has none (a bare
    /// REQ read).
    fn label_cell(&self) -> String {
        self.label().unwrap_or("-").to_owned()
    }

    /// The status-column hue from the ROW's raw authored status (SL-053 PHASE-02,
    /// F-4) — a `Dangling` row (whose `status` cell is the inline load-error note)
    /// is never coloured.
    fn status_hue(&self) -> Option<owo_colors::AnsiColors> {
        match self {
            CoverageRow::Healthy { status, .. } => listing::status_hue(status.as_str()),
            CoverageRow::Dangling { .. } => None,
        }
    }
}

/// The table columns a coverage row can show (`--columns` tokens over
/// `R = CoverageRow`). Non-capturing extractors, mirroring spec's `SPEC_COLUMNS`
/// (SL-037 D5). A `Dangling` row renders its data cells as the inline load-error
/// note (never a fabricated status). Declaration order is what the unknown-column
/// error lists.
const COVERAGE_COLUMNS: [listing::Column<CoverageRow>; 6] = [
    listing::Column {
        name: "id",
        header: "id",
        cell: |r| r.id().to_owned(),
        paint: listing::ColumnPaint::Fixed(owo_colors::AnsiColors::Cyan),
    },
    listing::Column {
        name: "label",
        header: "label",
        cell: CoverageRow::label_cell,
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "kind",
        header: "kind",
        cell: CoverageRow::kind_cell,
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "status",
        header: "status",
        cell: CoverageRow::status_cell,
        paint: listing::ColumnPaint::ByValue(CoverageRow::status_hue),
    },
    listing::Column {
        name: "observed",
        header: "observed",
        cell: CoverageRow::observed_cell,
        paint: listing::ColumnPaint::None,
    },
    listing::Column {
        name: "verdict",
        header: "verdict",
        cell: CoverageRow::verdict_cell,
        paint: listing::ColumnPaint::None,
    },
];

/// The default visible set: `id status observed verdict`. `label` is prepended
/// when the row set carries any membership label (a spec fan); `--columns` reveals
/// `label`/`kind` on demand otherwise.
const COVERAGE_DEFAULT: &[&str] = &["id", "status", "observed", "verdict"];

/// Render the coverage rows as a left-aligned text table (the `Table` format).
/// Default columns are `id status observed verdict`, with `label` prepended when
/// the row set carries any label (a spec fan). An explicit `columns` selection
/// wins verbatim. Rides `listing::select_columns` + `render_columns` UNCHANGED.
pub(crate) fn render_table(
    rows: &[CoverageRow],
    columns: Option<&[String]>,
    color: bool,
) -> anyhow::Result<String> {
    // Prepend `label` to the default set only when some row carries one — a spec
    // fan shows the membership label; a bare REQ read omits the column entirely.
    let has_label = rows.iter().any(|r| r.label().is_some());
    let default: Vec<&str> = if has_label && columns.is_none() {
        std::iter::once("label")
            .chain(COVERAGE_DEFAULT.iter().copied())
            .collect()
    } else {
        COVERAGE_DEFAULT.to_vec()
    };
    let sel = listing::select_columns(&COVERAGE_COLUMNS, &default, columns)?;
    Ok(listing::render_columns(rows, &sel, color))
}

/// One coverage row's faithful JSON shape (design §5 D5 contract; PHASE-06
/// goldens pin it). A healthy row carries the authored kind/status plus the
/// observed/verdict tokens (and the `divergent_reason` token when the verdict is
/// `Divergent`); a dangling row carries `dangling: true` + `load_error` and NO
/// status/observed/verdict/kind keys. `label` is omitted when absent.
#[derive(Serialize)]
#[serde(untagged)]
enum CoverageJsonRow {
    Healthy {
        requirement: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        kind: &'static str,
        status: &'static str,
        observed: &'static str,
        verdict: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        divergent_reason: Option<&'static str>,
    },
    Dangling {
        requirement: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        dangling: bool,
        load_error: String,
    },
}

/// Project one [`CoverageRow`] into its JSON shape — the verdict's reason is
/// surfaced as its own token only on the `Divergent` arm.
fn json_row(row: &CoverageRow) -> CoverageJsonRow {
    match row {
        CoverageRow::Healthy {
            id,
            label,
            kind,
            status,
            observed,
            verdict,
        } => {
            let divergent_reason = match verdict {
                Verdict::Divergent(r) => Some(r.label()),
                Verdict::Coherent | Verdict::Indeterminate => None,
            };
            CoverageJsonRow::Healthy {
                requirement: id.clone(),
                label: label.clone(),
                kind: kind.as_str(),
                status: status.as_str(),
                observed: observed.label(),
                verdict: verdict.label(),
                divergent_reason,
            }
        }
        CoverageRow::Dangling {
            id,
            label,
            load_error,
        } => CoverageJsonRow::Dangling {
            requirement: id.clone(),
            label: label.clone(),
            dangling: true,
            load_error: load_error.clone(),
        },
    }
}

/// Render the coverage rows as the shared `{kind:"coverage", rows:[…]}` JSON
/// envelope (via [`listing::json_envelope`]). Each row's shape is [`json_row`].
pub(crate) fn render_json(rows: &[CoverageRow]) -> anyhow::Result<String> {
    let json_rows: Vec<CoverageJsonRow> = rows.iter().map(json_row).collect();
    listing::json_envelope("coverage", &json_rows)
}

/// The thin shell behind `doctrine coverage <reference>` — resolve the root,
/// materialise the rows (the sole disk seam), then render. Mirrors
/// [`crate::spec::run_req_list`]: a top-level leaf with its own
/// `{ reference, columns, format, json }` (NOT a `CommonListArgs` surface — design
/// Q3). A read only — never writes, never derives authored status (the F1 wall
/// lives in [`rows`]).
pub(crate) fn run(
    path: Option<PathBuf>,
    reference: &str,
    columns: Option<&[String]>,
    format: listing::Format,
    json: bool,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let rows = rows(&root, reference)?;
    // `--json` forces Json over any `--format` (A-9; mirror listing.rs:171).
    let resolved = if json { listing::Format::Json } else { format };
    let out = match resolved {
        listing::Format::Json => render_json(&rows)?,
        // Resolve colour capability ONCE in the impure shell (SL-053 D3), inject as
        // a bool — JSON stays plain.
        listing::Format::Table => render_table(&rows, columns, crate::tty::stdout_color_enabled())?,
    };
    write!(std::io::stdout(), "{out}")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "tests: fail-fast unwrap on round-trip/parse is idiomatic"
)]
mod tests {
    use super::*;
    use crate::coverage::{CoverageEntry, CoverageKey, IsStale};
    use crate::requirement::CoverageStatus;
    use std::fs;

    // --- shared composite fixtures (rebuilt — coverage.rs's `composites()` is
    //     private to its test mod; we construct equivalents via the public
    //     `composite()` + `CoverageEntry`/`IsStale`). -------------------------

    fn cell(
        slice: &str,
        change: &str,
        status: CoverageStatus,
        stale: IsStale,
    ) -> (CoverageEntry, IsStale) {
        (
            CoverageEntry {
                key: CoverageKey {
                    slice: slice.to_owned(),
                    requirement: "REQ-111".to_owned(),
                    contributing_change: change.to_owned(),
                    mode: "VT".to_owned(),
                },
                status,
                git_anchor: "anchor-abc123".to_owned(),
                attested_date: None,
                touched_paths: Vec::new(),
            },
            stale,
        )
    }

    // --- VT-1: observed_state is TOTAL + non-overlapping over the 5 states -----

    #[test]
    fn observed_state_partitions_the_five_canonical_composites() {
        let empty = coverage::composite(&[]);
        assert!(matches!(observed_state(&empty), ObservedState::None));

        let fresh_verified = coverage::composite(&[cell(
            "SL-042",
            "SL-042",
            CoverageStatus::Verified,
            IsStale::Fresh,
        )]);
        assert!(matches!(
            observed_state(&fresh_verified),
            ObservedState::Verified
        ));

        let stale_verified = coverage::composite(&[cell(
            "SL-042",
            "SL-042",
            CoverageStatus::Verified,
            IsStale::Stale,
        )]);
        assert!(matches!(
            observed_state(&stale_verified),
            ObservedState::Stale
        ));

        let failed = coverage::composite(&[cell(
            "SL-042",
            "SL-042",
            CoverageStatus::Failed,
            IsStale::Fresh,
        )]);
        assert!(matches!(
            observed_state(&failed),
            ObservedState::Contradicted
        ));

        let forward = coverage::composite(&[
            cell(
                "SL-042",
                "SL-042",
                CoverageStatus::Planned,
                IsStale::Unknown,
            ),
            cell(
                "SL-043",
                "SL-043",
                CoverageStatus::InProgress,
                IsStale::Stale,
            ),
        ]);
        assert!(matches!(observed_state(&forward), ObservedState::Forward));
    }

    #[test]
    fn observed_state_does_not_track_drift_one_to_one() {
        // E6: a fresh-verified composite is `Verified` observed, but `drift`
        // against a `Pending` author is `Divergent` — the two partitions answer
        // different questions, so they intentionally diverge here.
        let fresh_verified = coverage::composite(&[cell(
            "SL-042",
            "SL-042",
            CoverageStatus::Verified,
            IsStale::Fresh,
        )]);
        assert!(matches!(
            observed_state(&fresh_verified),
            ObservedState::Verified
        ));
        let verdict = coverage::drift(ReqStatus::Pending, &fresh_verified);
        assert!(matches!(verdict, Verdict::Divergent(_)));
    }

    // --- VT-3: dispatch ------------------------------------------------------

    #[test]
    fn dispatch_classifies_req_prd_spec_and_rejects_garbage() {
        assert!(matches!(dispatch("REQ-007").unwrap(), Target::Req(r) if r == "REQ-007"));
        assert!(matches!(dispatch("PRD-001").unwrap(), Target::Spec(s) if s == "PRD-001"));
        assert!(matches!(dispatch("SPEC-012").unwrap(), Target::Spec(s) if s == "SPEC-012"));
        assert!(dispatch("SL-045").is_err());
        assert!(dispatch("garbage").is_err());
        assert!(dispatch("").is_err());
    }

    // --- VT-2: row materialisation over a temp corpus ------------------------

    /// Reserve a requirement and overwrite its authored kind/status.
    fn make_req(root: &Path, slug: &str, kind: ReqKind, status: ReqStatus) -> String {
        let reserved = requirement::reserve(root, slug, slug, "2026-06-12").unwrap();
        let id = reserved.eid.numeric_id().unwrap();
        requirement::set_kind(root, id, kind).unwrap();
        requirement::set_status(root, id, status).unwrap();
        requirement::canonical_id(id)
    }

    #[test]
    fn bare_single_req_row_has_no_label() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let fk = make_req(root, "auth", ReqKind::Functional, ReqStatus::Active);

        let rows = rows(root, &fk).unwrap();
        assert_eq!(rows.len(), 1);
        match rows.first().unwrap() {
            CoverageRow::Healthy { id, label, .. } => {
                assert_eq!(*id, fk);
                assert!(
                    label.is_none(),
                    "a bare REQ read carries no membership label"
                );
            }
            CoverageRow::Dangling { .. } => panic!("expected a healthy row"),
        }
    }

    #[test]
    fn bare_single_req_load_failure_is_fatal() {
        let dir = tempfile::tempdir().unwrap();
        // No requirement reserved — the FK dangles; a bare REQ has no fan to
        // protect, so the load error propagates.
        let err = rows(dir.path(), "REQ-404");
        assert!(err.is_err());
    }

    #[test]
    fn spec_fan_preserves_member_order_and_continues_past_a_dangling_member() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // A product spec with three members: two real, one dangling FK. Members
        // are appended in order; member_reqs preserves advisory `order`.
        crate::spec::run_new(
            Some(root.to_path_buf()),
            crate::spec::SpecSubtype::Product,
            Some("Login".to_owned()),
            Some("login".to_owned()),
        )
        .unwrap();

        let first = make_req(root, "first", ReqKind::Functional, ReqStatus::Active);
        let second = make_req(root, "second", ReqKind::Quality, ReqStatus::Pending);
        let members_path = root.join(".doctrine/spec/product/001/members.toml");
        // Append the two real members, then a dangling FK, preserving order.
        for (idx, fk) in [first.as_str(), second.as_str(), "REQ-999"]
            .iter()
            .enumerate()
        {
            let mut text = fs::read_to_string(&members_path).unwrap();
            let order = idx + 1;
            text.push_str(&format!(
                "\n[[member]]\nrequirement = \"{fk}\"\nlabel = \"FR-{order:03}\"\norder = {order}\n"
            ));
            fs::write(&members_path, text).unwrap();
        }

        let rows = rows(root, "PRD-001").unwrap();
        assert_eq!(rows.len(), 3, "the fan continues past the dangling member");

        // Order preserved: first, second, then the dangling REQ-999.
        assert_eq!(rows.first().unwrap().id(), first);
        assert_eq!(rows.get(1).unwrap().id(), second);
        match rows.get(2).unwrap() {
            CoverageRow::Dangling {
                id,
                label,
                load_error,
            } => {
                assert_eq!(id, "REQ-999");
                assert_eq!(label.as_deref(), Some("FR-003"));
                assert!(!load_error.is_empty());
            }
            CoverageRow::Healthy { .. } => panic!("REQ-999 should dangle"),
        }
    }

    #[test]
    fn dangling_json_row_has_dangling_flag_and_no_status_keys() {
        let row = CoverageRow::Dangling {
            id: "REQ-999".to_owned(),
            label: Some("FR-003".to_owned()),
            load_error: "requirement REQ-999 not found".to_owned(),
        };
        let json = render_json(std::slice::from_ref(&row)).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = value
            .get("rows")
            .and_then(|r| r.get(0))
            .and_then(serde_json::Value::as_object)
            .unwrap();

        assert_eq!(
            obj.get("dangling").and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert!(obj.contains_key("load_error"));
        assert_eq!(
            obj.get("requirement").and_then(serde_json::Value::as_str),
            Some("REQ-999")
        );
        assert_eq!(
            obj.get("label").and_then(serde_json::Value::as_str),
            Some("FR-003")
        );
        // No fabricated authored cells on a dangling row.
        assert!(!obj.contains_key("status"));
        assert!(!obj.contains_key("observed"));
        assert!(!obj.contains_key("verdict"));
        assert!(!obj.contains_key("kind"));
    }

    #[test]
    fn healthy_json_row_omits_label_and_divergent_reason_when_none() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let fk = make_req(root, "auth", ReqKind::Functional, ReqStatus::Active);
        // No coverage cells → observed=none, drift(Active, empty)=Indeterminate
        // (not Divergent), so divergent_reason is omitted.
        let rows = rows(root, &fk).unwrap();
        let json = render_json(&rows).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = value
            .get("rows")
            .and_then(|r| r.get(0))
            .and_then(serde_json::Value::as_object)
            .unwrap();

        assert_eq!(
            obj.get("status").and_then(serde_json::Value::as_str),
            Some("active")
        );
        assert_eq!(
            obj.get("observed").and_then(serde_json::Value::as_str),
            Some("none")
        );
        assert!(!obj.contains_key("label"), "a bare REQ omits the label key");
        assert!(
            !obj.contains_key("divergent_reason"),
            "a non-divergent verdict omits the reason key"
        );
    }

    // --- table render: default columns + label prepend ------------------------

    #[test]
    fn table_prepends_label_only_for_a_labelled_row_set() {
        let labelled = CoverageRow::Healthy {
            id: "REQ-001".to_owned(),
            label: Some("FR-001".to_owned()),
            kind: ReqKind::Functional,
            status: ReqStatus::Active,
            observed: ObservedState::None,
            verdict: Verdict::Indeterminate,
        };
        let out = render_table(std::slice::from_ref(&labelled), None, false).unwrap();
        let header = out.lines().next().unwrap();
        assert!(
            header.starts_with("label"),
            "labelled set leads with label: {header}"
        );

        let bare = CoverageRow::Healthy {
            id: "REQ-001".to_owned(),
            label: None,
            kind: ReqKind::Functional,
            status: ReqStatus::Active,
            observed: ObservedState::None,
            verdict: Verdict::Indeterminate,
        };
        let out = render_table(std::slice::from_ref(&bare), None, false).unwrap();
        let header = out.lines().next().unwrap();
        assert!(
            header.starts_with("id"),
            "bare set omits label, leads with id: {header}"
        );
    }
}
