// SPDX-License-Identifier: GPL-3.0-only
//! RV read model (SL-278 PHASE-01, engine tier): the closed-vocabulary defect
//! disclosure every read surface renders (moved here from the command-tier
//! `review::read`), and — from PHASE-01 onward — the pure projection types
//! `review show`/the census build on (design sec-2). Imports only
//! `review_ledger` siblings, `serde`, and leaf modules (ADR-001): no disk, no
//! clock.

use serde::Serialize;

use super::derive::{VocabDefect, counters, finding_await, vocabulary_defects};
use super::gate::{OutstandingCounts, effective_severity};
use super::schema::{FindingRow, ReviewDoc, TurnRow, canonical_id};
use super::vocab::{Await, FindingStatus, ReviewStatus, Severity, Vocab};

// ---------------------------------------------------------------------------
// Warnings (moved from `review::read`, SL-278 PHASE-01 T4 — unchanged fields,
// serde attributes and `line()`)
// ---------------------------------------------------------------------------

/// One closed-vocabulary defect as disclosed on a structured channel (`--json`,
/// MCP): the defect plus the RV it sits on, so a multi-review surface (`list`)
/// needs no second shape. Serialises as `{effect, field, finding, raw, rv}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ReviewWarning {
    pub(crate) rv: String,
    #[serde(flatten)]
    pub(crate) defect: VocabDefect,
}

impl ReviewWarning {
    /// The text-channel line (`warning: RV-NNN F-n …`), newline-terminated.
    pub(crate) fn line(&self) -> String {
        let mut line = self.defect.warning_line(&self.rv);
        line.push('\n');
        line
    }
}

/// Every closed-vocabulary defect in one ledger, as [`ReviewWarning`]s.
pub(crate) fn warnings_of(doc: &ReviewDoc) -> Vec<ReviewWarning> {
    let rv = canonical_id(doc.id);
    vocabulary_defects(doc)
        .into_iter()
        .map(|defect| ReviewWarning {
            rv: rv.clone(),
            defect,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// The projection (SL-278 PHASE-01 T5, design sec-2)
// ---------------------------------------------------------------------------

/// How much of each shown finding a render carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Depth {
    Index,
    Full,
}

/// Which findings a view carries. Ids and filters are exclusive by
/// construction: the CLI and MCP refuse the combination before this exists
/// ([`Projection::parse`]).
#[derive(Debug)]
pub(crate) enum Selection {
    /// Every finding the filter admits (an empty filter admits all).
    Filtered(FindingFilter),
    /// Exactly these ids, normalised to `F-n`, deduped, in the order given.
    Ids(Vec<String>),
}

/// The selection plus whether the brief body rides along (design sec-2):
/// everything `review show`/the census need to pick findings out of a
/// ledger, transport-neutral.
#[derive(Debug)]
pub(crate) struct Projection {
    pub(crate) selection: Selection,
    /// `--full`: carry the brief body.
    pub(crate) with_body: bool,
}

impl Projection {
    /// `Full` iff [`Self::with_body`] or [`Selection::Ids`]; else `Index` — a
    /// selection by id always shows what it asked for in full (DEC-362).
    pub(crate) fn depth(&self) -> Depth {
        if self.with_body || matches!(self.selection, Selection::Ids(_)) {
            Depth::Full
        } else {
            Depth::Index
        }
    }

    /// The default render — what `doctrine show RV-N` gets (DEC-364): every
    /// finding, index depth, no body.
    pub(crate) fn index() -> Self {
        Self {
            selection: Selection::Filtered(FindingFilter::default()),
            with_body: false,
        }
    }

    /// Parse the transport-neutral arguments both CLI and MCP collect — the
    /// single validation point, so both refuse alike. `finding` together with
    /// any filter is refused; otherwise `finding` (normalised, deduped, order
    /// kept) becomes [`Selection::Ids`], else the filter becomes
    /// [`Selection::Filtered`].
    pub(crate) fn parse(args: &ProjectionArgs) -> anyhow::Result<Self> {
        let ids = split_commas(&args.finding);
        let filtered = args.open || !args.status.is_empty() || !args.severity.is_empty();
        if !ids.is_empty() && filtered {
            anyhow::bail!("--finding selects exact findings; drop --open/-s/-S");
        }
        let selection = if ids.is_empty() {
            Selection::Filtered(FindingFilter::parse(
                args.open,
                &args.status,
                &args.severity,
            )?)
        } else {
            let mut normalised: Vec<String> = Vec::new();
            for raw in ids {
                let id = normalise_finding_id(&raw);
                if !normalised.contains(&id) {
                    normalised.push(id);
                }
            }
            Selection::Ids(normalised)
        };
        Ok(Self {
            selection,
            with_body: args.full,
        })
    }
}

/// Raw strings as either transport delivers them; [`Projection::parse`]
/// validates. Each of `finding`/`status`/`severity` may carry pre-split
/// elements (the CLI's `value_delimiter`) or a single comma-joined string (an
/// MCP caller) — both read alike (`split_commas`).
#[derive(Default)]
pub(crate) struct ProjectionArgs {
    pub(crate) finding: Vec<String>,
    pub(crate) full: bool,
    pub(crate) open: bool,
    pub(crate) status: Vec<String>,
    pub(crate) severity: Vec<String>,
}

/// Split every element on `,`, trim, and drop empty segments — the one comma
/// domain `Projection::parse`/`FindingFilter::parse` share, so a CLI's
/// pre-split `Vec` and an MCP's single joined string read alike.
fn split_commas(raw: &[String]) -> Vec<String> {
    raw.iter()
        .flat_map(|s| s.split(','))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

/// A bare number becomes `F-n` (A3); `F-n` is kept; any other spelling (e.g.
/// `f-3`) passes through verbatim and fails later as an unknown id —
/// [`Projection::parse`] names only bare numbers as a second spelling.
fn normalise_finding_id(raw: &str) -> String {
    if !raw.is_empty() && raw.bytes().all(|b| b.is_ascii_digit()) {
        let mut id = "F-".to_owned();
        id.push_str(raw);
        id
    } else {
        raw.to_owned()
    }
}

/// Which findings a filtered selection admits (design sec-2, DEC-363): an
/// empty filter admits every finding.
#[derive(Debug, Default)]
pub(crate) struct FindingFilter {
    pub(crate) open: bool,
    pub(crate) statuses: Vec<FindingStatus>,
    pub(crate) severities: Vec<Severity>,
}

impl FindingFilter {
    /// Split commas, validate against the vocabularies, build. Called by
    /// [`Projection::parse`] and by the census, so both refuse alike — an
    /// unknown status or severity names the vocabulary (the
    /// `listing::validate_statuses` shape, via `FindingStatus::parse`/
    /// `Severity::parse`, which already name it).
    pub(crate) fn parse(
        open: bool,
        status: &[String],
        severity: &[String],
    ) -> anyhow::Result<Self> {
        let statuses = split_commas(status)
            .iter()
            .map(|s| FindingStatus::parse(s).map_err(|e| anyhow::anyhow!(e)))
            .collect::<anyhow::Result<Vec<_>>>()?;
        let severities = split_commas(severity)
            .iter()
            .map(|s| Severity::parse(s).map_err(|e| anyhow::anyhow!(e)))
            .collect::<anyhow::Result<Vec<_>>>()?;
        Ok(Self {
            open,
            statuses,
            severities,
        })
    }

    /// The filter predicate (DEC-363): `--open` keeps non-terminal findings
    /// only (an out-of-vocabulary status is non-terminal, so it is kept); an
    /// empty `statuses`/`severities` admits every value, otherwise only a
    /// named one — through [`effective_severity`], so this agrees with the
    /// close gate's classification by construction.
    pub(crate) fn admits(&self, row: &FindingRow) -> bool {
        let status = Vocab::<FindingStatus>::read(&row.status);
        (!self.open || !status.is_known_terminal())
            && (self.statuses.is_empty()
                || status.known().is_some_and(|s| self.statuses.contains(&s)))
            && (self.severities.is_empty()
                || self.severities.contains(&effective_severity(&row.severity)))
    }
}

// ---------------------------------------------------------------------------
// The view (design sec-2)
// ---------------------------------------------------------------------------

/// An owned, serialisable projection of one review (design sec-2, RFC-032 D5):
/// the header facts (over the **whole** ledger, whatever the selection) plus
/// the selected findings. Built once per read; pure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ReviewView {
    pub(crate) canonical: String,
    pub(crate) title: String,
    pub(crate) facet: String,
    pub(crate) status: ReviewStatus,
    pub(crate) awaiting: Await,
    /// The `reviews`-edge label (`SL-024` or `SL-024@PHASE-03`, via
    /// [`super::schema::Target::label`]).
    pub(crate) target: String,
    pub(crate) raiser: String,
    pub(crate) responder: String,
    pub(crate) rounds: u32,
    pub(crate) concluded: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) tags: Vec<String>,
    pub(crate) depth: Depth,
    pub(crate) finding_count: usize,
    pub(crate) outstanding: Outstanding,
    pub(crate) findings: Vec<FindingView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) body_lines: Option<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) warnings: Vec<ReviewWarning>,
}

/// One finding as projected (design sec-2): identity and status always;
/// `detail`/`response`/`turns` only at [`Depth::Full`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FindingView {
    pub(crate) id: String,
    pub(crate) severity: Vocab<Severity>,
    pub(crate) status: Vocab<FindingStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) disposition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) route: Option<String>,
    pub(crate) title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) response: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) turns: Vec<TurnRow>,
}

impl FindingView {
    /// Project one authored row (never truncated, DEC-366): `detail`/
    /// `response`/`turns` only at [`Depth::Full`] — [`Depth::Index`] carries
    /// no prose.
    fn of(row: &FindingRow, depth: Depth) -> Self {
        let full = depth == Depth::Full;
        Self {
            id: row.id.clone(),
            severity: Vocab::read(&row.severity),
            status: Vocab::read(&row.status),
            disposition: row.disposition.clone(),
            route: row.route.clone(),
            title: row.title.clone(),
            detail: full.then(|| row.detail.clone()),
            response: if full { row.response.clone() } else { None },
            turns: if full { row.turn.clone() } else { Vec::new() },
        }
    }
}

/// Non-terminal findings by whose move they await, counted by severity
/// (design sec-2): built with [`finding_await`] and [`effective_severity`],
/// the same two functions `derived_status`/the close gate use, so the three
/// cannot classify a finding differently.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub(crate) struct Outstanding {
    pub(crate) responder: OutstandingCounts,
    pub(crate) raiser: OutstandingCounts,
}

impl Outstanding {
    fn of(doc: &ReviewDoc) -> Self {
        let mut outstanding = Self::default();
        for finding in &doc.finding {
            let status = Vocab::<FindingStatus>::read(&finding.status);
            let bucket = match finding_await(&status) {
                Await::Responder => &mut outstanding.responder,
                Await::Raiser => &mut outstanding.raiser,
                Await::None => continue,
            };
            bucket.add(effective_severity(&finding.severity));
        }
        outstanding
    }
}

impl ReviewView {
    /// Build the projection from the authored doc, the brief body, and the
    /// baton's legacy `(rounds, contests)` (design sec-2) — the single
    /// raw→typed read every render/census row shares. `Err` only for
    /// [`Selection::Ids`] naming an id the ledger does not have (A4).
    pub(crate) fn build(
        doc: &ReviewDoc,
        body: &str,
        legacy_counters: (u32, u32),
        projection: &Projection,
    ) -> anyhow::Result<Self> {
        let (status, awaiting) = doc.derived();
        let depth = projection.depth();
        let findings = match &projection.selection {
            Selection::Filtered(filter) => doc
                .finding
                .iter()
                .filter(|row| filter.admits(row))
                .map(|row| FindingView::of(row, depth))
                .collect(),
            Selection::Ids(ids) => ids
                .iter()
                .map(|id| {
                    doc.finding
                        .iter()
                        .find(|row| &row.id == id)
                        .map(|row| FindingView::of(row, depth))
                        .ok_or_else(|| unknown_finding_error(doc, id))
                })
                .collect::<anyhow::Result<Vec<_>>>()?,
        };
        let (body, body_lines) = if projection.with_body {
            (Some(body.to_owned()), None)
        } else {
            (None, Some(body.lines().count()))
        };
        Ok(Self {
            canonical: canonical_id(doc.id),
            title: doc.title.clone(),
            facet: doc.review.facet.clone(),
            status,
            awaiting,
            target: doc.target.label(),
            raiser: doc.review.raiser.clone(),
            responder: doc.review.responder.clone(),
            rounds: counters(doc, legacy_counters).rounds,
            concluded: doc.review.concluded,
            tags: doc.tags.clone(),
            depth,
            finding_count: doc.finding.len(),
            outstanding: Outstanding::of(doc),
            findings,
            body,
            body_lines,
            warnings: warnings_of(doc),
        })
    }
}

/// A4: `unknown finding F-9 on RV-012 (has F-1..F-5)` — the ledger's own ids,
/// as a range when they are exactly `F-1..F-n` in order, else comma-joined;
/// `(has none)` when the ledger is empty.
fn unknown_finding_error(doc: &ReviewDoc, id: &str) -> anyhow::Error {
    let ids: Vec<&str> = doc.finding.iter().map(|f| f.id.as_str()).collect();
    let known = if ids.is_empty() {
        "none".to_owned()
    } else if ids
        .iter()
        .enumerate()
        .all(|(i, &want)| want == format!("F-{}", i + 1))
    {
        format!("F-1..F-{}", ids.len())
    } else {
        ids.join(", ")
    };
    anyhow::anyhow!(
        "unknown finding {id} on {} (has {known})",
        canonical_id(doc.id)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A multi-line `detail` over 3 KB (sec-8's fixture set) — generated, not
    /// a literal, so the byte-for-byte check (`full_projection_carries_detail_verbatim`)
    /// is not fooled by a hand-typed string that happens to round-trip.
    fn big_detail() -> String {
        (0..400)
            .map(|i| format!("detail line {i}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The sec-8 fixture: six findings spanning all four severities, every
    /// known status, one out-of-vocabulary status/severity pair (F-6), a
    /// routed finding (F-2), a journalled finding (F-3, two turns in file
    /// order), and the over-3KB `detail` (F-4) — the `derive.rs::ledger`
    /// inline-TOML pattern, copied.
    fn fixture() -> ReviewDoc {
        let text = format!(
            "id = 12\nslug = \"s\"\ntitle = \"Design review: SL-278\"\n\
             [review]\nfacet = \"design\"\nraiser = \"raiser\"\nresponder = \"responder\"\n\
             [target]\nref = \"SL-278\"\n\
             [[finding]]\nid = \"F-1\"\nstatus = \"open\"\nseverity = \"blocker\"\n\
             title = \"t1\"\ndetail = \"d1\"\n\
             [[finding]]\nid = \"F-2\"\nstatus = \"answered\"\nseverity = \"major\"\n\
             title = \"t2\"\ndetail = \"d2\"\ndisposition = \"fix\"\nroute = \"review\"\n\
             response = \"resp2\"\n\
             [[finding]]\nid = \"F-3\"\nstatus = \"contested\"\nseverity = \"minor\"\n\
             title = \"t3\"\ndetail = \"d3\"\n\
             [[finding.turn]]\nact = \"raise\"\nrole = \"raiser\"\n\
             [[finding.turn]]\nact = \"contest\"\nrole = \"raiser\"\nnote = \"n\"\n\
             [[finding]]\nid = \"F-4\"\nstatus = \"verified\"\nseverity = \"nit\"\n\
             title = \"t4\"\ndetail = \"\"\"\n{}\"\"\"\n\
             [[finding]]\nid = \"F-5\"\nstatus = \"withdrawn\"\nseverity = \"blocker\"\n\
             title = \"t5\"\ndetail = \"d5\"\n\
             [[finding]]\nid = \"F-6\"\nstatus = \"zombie\"\nseverity = \"crit\"\n\
             title = \"t6\"\ndetail = \"d6\"\n",
            big_detail()
        );
        toml::from_str(&text).unwrap()
    }

    /// A ledger with no turn and no base, for [`rounds_reads_baton_only_for_unjournalled_ledger`].
    fn unjournalled() -> ReviewDoc {
        let text = "id = 1\nslug = \"s\"\ntitle = \"t\"\n[review]\nfacet = \"design\"\n\
                     raiser = \"r\"\nresponder = \"s\"\n[target]\nref = \"SL-001\"\n";
        toml::from_str(text).unwrap()
    }

    fn ids_of(view: &ReviewView) -> Vec<&str> {
        view.findings.iter().map(|f| f.id.as_str()).collect()
    }

    fn build(doc: &ReviewDoc, body: &str, projection: &Projection) -> ReviewView {
        ReviewView::build(doc, body, (0, 0), projection).unwrap()
    }

    #[test]
    fn index_projection_carries_no_prose() {
        let doc = fixture();
        let view = build(&doc, "brief body\n", &Projection::index());
        assert_eq!(view.depth, Depth::Index);
        let json = serde_json::to_value(&view).unwrap();
        assert!(json.get("body").is_none(), "{json}");
        assert!(json["body_lines"].is_u64());
        for finding in json["findings"].as_array().unwrap() {
            assert!(finding.get("detail").is_none(), "{finding}");
            assert!(finding.get("response").is_none(), "{finding}");
            assert!(finding.get("turns").is_none(), "{finding}");
        }
    }

    #[test]
    fn full_projection_carries_detail_verbatim() {
        let doc = fixture();
        let projection = Projection {
            selection: Selection::Filtered(FindingFilter::default()),
            with_body: true,
        };
        let view = build(&doc, "brief body\n", &projection);
        assert_eq!(view.depth, Depth::Full);
        assert_eq!(view.body.as_deref(), Some("brief body\n"));
        assert!(view.body_lines.is_none());
        let f3 = view.findings.iter().find(|f| f.id == "F-3").unwrap();
        assert_eq!(
            f3.turns.iter().map(|t| t.act.as_str()).collect::<Vec<_>>(),
            vec!["raise", "contest"],
            "turns in file order"
        );
        let f4 = view.findings.iter().find(|f| f.id == "F-4").unwrap();
        assert_eq!(f4.detail.as_deref(), Some(big_detail().as_str()));
    }

    #[test]
    fn selection_by_id_keeps_given_order_and_normalises_bare_numbers() {
        let doc = fixture();
        let projection = Projection::parse(&ProjectionArgs {
            finding: vec!["3".to_owned(), "F-1".to_owned()],
            ..ProjectionArgs::default()
        })
        .unwrap();
        let view = build(&doc, "", &projection);
        assert_eq!(ids_of(&view), vec!["F-3", "F-1"]);
        assert_eq!(view.depth, Depth::Full);
    }

    #[test]
    fn unknown_finding_id_is_refused_naming_valid_ids() {
        let doc = fixture();
        let projection = Projection {
            selection: Selection::Ids(vec!["F-9".to_owned()]),
            with_body: false,
        };
        let err = ReviewView::build(&doc, "", (0, 0), &projection).unwrap_err();
        let text = err.to_string();
        assert!(text.contains("unknown finding F-9 on RV-012"), "{text}");
        assert!(text.contains("has F-1..F-6"), "{text}");
    }

    #[test]
    fn finding_with_filter_is_refused() {
        let err = Projection::parse(&ProjectionArgs {
            finding: vec!["F-1".to_owned()],
            open: true,
            ..ProjectionArgs::default()
        })
        .unwrap_err();
        let text = err.to_string();
        assert!(text.contains("--finding selects exact findings"), "{text}");
        assert!(text.contains("--open"), "{text}");
    }

    #[test]
    fn open_filter_treats_unknown_status_as_open() {
        let doc = fixture();
        let filter = FindingFilter::parse(true, &[], &[]).unwrap();
        let projection = Projection {
            selection: Selection::Filtered(filter),
            with_body: false,
        };
        let view = build(&doc, "", &projection);
        assert_eq!(ids_of(&view), vec!["F-1", "F-2", "F-3", "F-6"]);
    }

    #[test]
    fn severity_filter_treats_unknown_severity_as_blocker() {
        let doc = fixture();
        let filter = FindingFilter::parse(false, &[], &["blocker".to_owned()]).unwrap();
        let projection = Projection {
            selection: Selection::Filtered(filter),
            with_body: false,
        };
        let view = build(&doc, "", &projection);
        assert_eq!(ids_of(&view), vec!["F-1", "F-5", "F-6"]);
    }

    #[test]
    fn filters_and_together() {
        let doc = fixture();
        let filter = FindingFilter::parse(
            false,
            &["open".to_owned(), "contested".to_owned()],
            &["minor".to_owned()],
        )
        .unwrap();
        let projection = Projection {
            selection: Selection::Filtered(filter),
            with_body: false,
        };
        let view = build(&doc, "", &projection);
        assert_eq!(
            ids_of(&view),
            vec!["F-3"],
            "status AND severity both narrow"
        );
    }

    #[test]
    fn header_facts_cover_whole_ledger_under_a_filter() {
        let doc = fixture();
        let narrow = build(
            &doc,
            "",
            &Projection {
                selection: Selection::Filtered(
                    FindingFilter::parse(false, &[], &["nit".to_owned()]).unwrap(),
                ),
                with_body: false,
            },
        );
        let whole = build(&doc, "", &Projection::index());
        assert_eq!(narrow.finding_count, 6);
        assert_eq!(narrow.finding_count, whole.finding_count);
        assert_eq!(narrow.outstanding, whole.outstanding);
        assert_eq!(narrow.warnings, whole.warnings);
        assert_eq!(whole.warnings.len(), 2, "F-6's status and severity defects");
    }

    #[test]
    fn outstanding_groups_by_awaited_party() {
        let doc = fixture();
        let view = build(&doc, "", &Projection::index());
        assert_eq!(
            view.outstanding,
            Outstanding {
                responder: OutstandingCounts {
                    blocker: 2,
                    major: 0,
                    minor: 1,
                    nit: 0,
                },
                raiser: OutstandingCounts {
                    blocker: 0,
                    major: 1,
                    minor: 0,
                    nit: 0,
                },
            },
            "open/contested/unknown await the responder; answered awaits the raiser"
        );
    }

    #[test]
    fn rounds_reads_baton_only_for_unjournalled_ledger() {
        let fresh = unjournalled();
        let view = ReviewView::build(&fresh, "", (5, 2), &Projection::index()).unwrap();
        assert_eq!(view.rounds, 5, "no turn, no base: the baton stands");

        let doc = fixture();
        let journalled = ReviewView::build(&doc, "", (99, 99), &Projection::index()).unwrap();
        assert_eq!(
            journalled.rounds, 2,
            "F-3's two turns are counted; the baton is ignored once journalled"
        );
    }
}
