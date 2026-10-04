// SPDX-License-Identifier: GPL-3.0-only
//! RV ledger schema (SL-268 D4, engine tier): the authored `review-NNN.toml`
//! shape (`ReviewMeta`, `Target`, `FindingRow`, `ReviewDoc`) and its readers.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use super::derive::{FindingState, derived_status};
use super::vocab::{Await, ReviewStatus, Vocab};
use crate::entity;
use crate::kinds::{REVIEW_DIR, REVIEW_KIND};
use crate::listing;

// ---------------------------------------------------------------------------
// Render (eager — the authored ledger toml + the `## Brief` md companion)
// ---------------------------------------------------------------------------

/// The parsed `[target]` edge (design §5/§7): the subject canonical ref and an
/// optional phase scope. Validated at `new`; the edge `RV-NNN ──reviews──▶ ref`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct Target {
    #[serde(rename = "ref")]
    pub(crate) reference: String,
    #[serde(default)]
    pub(crate) phase: Option<String>,
}

impl Target {
    /// Parse a `--target` argument, accepting the `ref@PHASE-NN` spelling as
    /// equivalent to separate `--target ref --phase PHASE-NN` flags (SL-268
    /// PHASE-07 D8, design D8 "target"). `PHASE-NN`'s shape is not validated
    /// here — `--phase` does not validate it either today, so the two
    /// spellings stay equal.
    pub(crate) fn parse(raw: &str, phase: Option<&str>) -> anyhow::Result<Target> {
        match raw.split_once('@') {
            None => Ok(Target {
                reference: raw.to_owned(),
                phase: phase.map(str::to_owned),
            }),
            Some((reference, phase_part)) => {
                if reference.is_empty() || phase_part.is_empty() || phase_part.contains('@') {
                    anyhow::bail!(
                        "malformed --target `{raw}`: the `ref@PHASE-NN` spelling needs a \
                         non-empty ref and phase on either side of exactly one `@`"
                    );
                }
                if phase.is_some() {
                    anyhow::bail!(
                        "--target `{raw}` already carries a phase via `@{phase_part}`; \
                         --phase is redundant with the `@PHASE-NN` spelling"
                    );
                }
                Ok(Target {
                    reference: reference.to_owned(),
                    phase: Some(phase_part.to_owned()),
                })
            }
        }
    }

    /// The `reviews`-edge label (`SL-024` or `SL-024@PHASE-03`, ADR-004):
    /// moved here from the command-tier `review::read::edge_label` (SL-278
    /// PHASE-01 A1) so the engine-tier [`super::view::ReviewView`] can build
    /// it too, with no second copy.
    pub(crate) fn label(&self) -> String {
        match &self.phase {
            Some(phase) => {
                let mut s = self.reference.clone();
                s.push('@');
                s.push_str(phase);
                s
            }
            None => self.reference.clone(),
        }
    }
}

/// The `[review]` metadata table (design §5): the facet, the two role labels, and
/// the concluded-pass marker.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct ReviewMeta {
    pub(crate) facet: String,
    pub(crate) raiser: String,
    pub(crate) responder: String,
    /// Whether the raiser has declared this pass finished (SL-244 `sec-4`,
    /// IMP-392, SL-268 D2). Set by `run_conclude`; cleared by `raise`/`reopen` in
    /// the same write that appends their turn (RV-396 `F-4`). Never a function of
    /// the findings.
    ///
    /// **This is not the `status` the file's own header forbids.** A review's
    /// status is derived from its findings (ADR-007 D-C8) and so is never stored;
    /// a pass *concluding* is an event, and it is not derivable — a clean pass and
    /// a pass never run present the same findings, the same derived status and the
    /// same `await`. Storing what cannot be derived is the storage rule holding,
    /// not bending.
    ///
    /// **Absence is not-concluded.** There is no third state and no migration:
    /// every ledger minted before this key existed reads unconcluded, which is the
    /// right answer for a pass nobody closed and the conservative one for a pass
    /// somebody did.
    #[serde(default)]
    pub(crate) concluded: bool,
    /// The baton's `rounds` at the first journalled write (SL-268 sec-2): the
    /// pre-journal history the turn count cannot see. Absent until seeded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) rounds_base: Option<u32>,
    /// The baton's `contests` at the first journalled write; the twin of
    /// [`Self::rounds_base`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) contests_base: Option<u32>,
    /// The review-level journal (`[[review.turn]]`): acts that move no finding.
    /// Absent is an empty journal (no migration).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) turn: Vec<TurnRow>,
}

/// One journalled turn (SL-268 sec-2, D1): who acted, how, and why. Every field
/// is a raw string, read open (D15) — an unknown `act` or `role` is carried, and
/// still counts as a turn. `disposition`/`route`/`response` snapshot what a
/// `dispose` answered, so a later re-dispose cannot erase what a contest argued
/// against. Written only through `review_ledger::transition`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct TurnRow {
    pub(crate) act: String,
    pub(crate) role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) disposition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) route: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) response: Option<String>,
}

// ---------------------------------------------------------------------------
// show / list — review computes its OWN derived status (never the shared reader)
// ---------------------------------------------------------------------------

/// One authored `[[finding]]` row, read as data for `show`/`list` derived status.
/// A faithful mirror of the on-disk shape (the raiser/responder/status fields).
/// The closed-vocab strings stay raw here: each reader classifies them through
/// [`Vocab`] (SL-268 D15), so a hand-edited value is carried, never rewritten.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct FindingRow {
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) severity: String,
    pub(crate) title: String,
    pub(crate) detail: String,
    #[serde(default)]
    pub(crate) disposition: Option<String>,
    /// Where the finding's answer routes (design sec-2/sec-4, SL-268 D1/D8) —
    /// set by `dispose`/`amend`. `skip_serializing_if` keeps every legacy and
    /// hand-seeded ledger's `show --json` byte-identical: absence on read is
    /// absence on write back, never a null.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) route: Option<String>,
    #[serde(default)]
    pub(crate) response: Option<String>,
    /// This finding's journal (`[[finding.turn]]`), in file order. Absent is an
    /// empty journal (no migration).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) turn: Vec<TurnRow>,
}

/// The full `review-NNN.toml` read as data (design §5) — id/slug/title (NO stored
/// status, D-C8), the `[review]` and `[target]` tables, and the append-only
/// findings. Review's own readers parse this; the shared strict `Meta` is never
/// asked for a status review does not store (D2).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct ReviewDoc {
    pub(crate) id: u32,
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) review: ReviewMeta,
    pub(crate) target: Target,
    #[serde(default)]
    pub(crate) finding: Vec<FindingRow>,
    #[serde(default, deserialize_with = "deserialize_tags_lenient")]
    pub(crate) tags: Vec<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::estimate::deserialize_lenient"
    )]
    pub(crate) estimate: Option<crate::estimate::EstimateFacet>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::value::deserialize_lenient"
    )]
    pub(crate) value: Option<crate::value::ValueFacet>,
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

impl ReviewDoc {
    /// Read the authored finding-status strings for the derived-status summary —
    /// the one status parse every reader shares. An out-of-vocabulary status (a
    /// hand-edit) reads as [`Vocab::Unknown`] with its raw string, never as a
    /// guessed known status (SL-268 D15); `derived_status` holds it non-terminal
    /// and the read surfaces disclose it (`vocabulary_defects`).
    pub(crate) fn finding_states(&self) -> Vec<FindingState> {
        self.finding
            .iter()
            .map(|f| FindingState {
                status: Vocab::read(&f.status),
            })
            .collect()
    }

    /// The review's derived `(ReviewStatus, Await)` (design §8, SL-268 D2) —
    /// computed at read time, never stored.
    pub(crate) fn derived(&self) -> (ReviewStatus, Await) {
        derived_status(&self.finding_states(), self.review.concluded)
    }
}

/// Read one review's `review-NNN.toml` as data.
pub(crate) fn read_review(review_root: &Path, id: u32) -> anyhow::Result<ReviewDoc> {
    let name = format!("{id:03}");
    let path = review_root.join(&name).join(format!("review-{name}.toml"));
    let text = fs::read_to_string(&path)
        .with_context(|| format!("review {name} not found at {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("Failed to parse {}", path.display()))
}

/// Read every `review-NNN.toml` under the review tree as data (for `list`).
pub(crate) fn read_reviews(review_root: &Path) -> anyhow::Result<Vec<ReviewDoc>> {
    let mut docs = Vec::new();
    for id in entity::scan_ids(review_root)? {
        docs.push(read_review(review_root, id)?);
    }
    Ok(docs)
}

/// One RV id that exists on disk but did not read as a [`ReviewDoc`] — a
/// numeric dir with no `review-NNN.toml`, or one that fails to parse (SL-278
/// PHASE-01 T6, design sec-5 § Tolerant scan). Wire shape `{"rv","reason"}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct UnreadableReview {
    pub(crate) rv: String,
    pub(crate) reason: String,
}

/// Tolerant scan (design sec-5 § Tolerant scan, A6): every readable review,
/// plus every id that exists but did not read, disclosed rather than skipped
/// (STD-003) — `rv` ascending for a deterministic `unreadable` order. Only a
/// failure to *list* the tree (`entity::scan_ids`) is an `Err`; a per-id read
/// failure becomes an [`UnreadableReview`] instead. [`read_reviews`] stays
/// fail-fast unchanged (EX-5) — the close gate must still fail closed.
pub(crate) fn scan_reviews(
    review_root: &Path,
) -> anyhow::Result<(Vec<ReviewDoc>, Vec<UnreadableReview>)> {
    let mut ids = entity::scan_ids(review_root)?;
    ids.sort_unstable();
    let mut docs = Vec::new();
    let mut unreadable = Vec::new();
    for id in ids {
        match read_review(review_root, id) {
            Ok(doc) => docs.push(doc),
            Err(e) => unreadable.push(UnreadableReview {
                rv: canonical_id(id),
                reason: format!("{e:#}"),
            }),
        }
    }
    Ok((docs, unreadable))
}

/// The `RV-NNN` canonical id for a numeric review id, via the single id-form
/// authority.
pub(crate) fn canonical_id(id: u32) -> String {
    listing::canonical_id(REVIEW_KIND.prefix, id)
}

/// Parse a review reference — `RV-007`, `rv-7`, or the bare id `7` — to its id.
/// Delegates to the shared [`crate::listing::parse_ref`] (IMP-125).
pub(crate) fn parse_ref(reference: &str) -> anyhow::Result<u32> {
    crate::listing::parse_ref("RV", "a review", reference)
}

/// Read the authored ledger bytes + the parsed doc for a review id — the step-2
/// snapshot the two CAS windows compare against.
pub(crate) fn read_authored(root: &Path, id: u32) -> anyhow::Result<(String, ReviewDoc)> {
    let name = format!("{id:03}");
    let path = root
        .join(REVIEW_DIR)
        .join(&name)
        .join(format!("review-{name}.toml"));
    let text = fs::read_to_string(&path)
        .with_context(|| format!("review {name} not found at {}", path.display()))?;
    let doc: ReviewDoc =
        toml::from_str(&text).with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok((text, doc))
}

/// The authored ledger path for a review id.
pub(crate) fn authored_path(root: &Path, id: u32) -> PathBuf {
    let name = format!("{id:03}");
    root.join(REVIEW_DIR)
        .join(&name)
        .join(format!("review-{name}.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEAD: &str = "id = 1\nslug = \"s\"\ntitle = \"t\"\n[review]\nfacet = \"design\"\n\
                        raiser = \"r\"\nresponder = \"s\"\n";
    const FINDING: &str = "[target]\nref = \"SL-001\"\n[[finding]]\nid = \"F-1\"\n\
                           status = \"open\"\nseverity = \"major\"\ntitle = \"t\"\ndetail = \"d\"\n";

    /// SL-278 PHASE-01 A1: `label` renders the bare ref with no phase, and
    /// `ref@PHASE-NN` with one.
    #[test]
    fn target_label_appends_the_phase_when_set() {
        assert_eq!(
            Target {
                reference: "SL-024".to_owned(),
                phase: None,
            }
            .label(),
            "SL-024"
        );
        assert_eq!(
            Target {
                reference: "SL-024".to_owned(),
                phase: Some("PHASE-03".to_owned()),
            }
            .label(),
            "SL-024@PHASE-03"
        );
    }

    /// `Target::parse` (SL-268 PHASE-07 T3, VT-3): the `ref@PHASE-NN` spelling
    /// and the separate `ref` + `--phase` flags parse to the same [`Target`].
    #[test]
    fn at_phase_spelling() {
        assert_eq!(
            Target::parse("SL-001", None).unwrap(),
            Target {
                reference: "SL-001".to_owned(),
                phase: None,
            }
        );
        assert_eq!(
            Target::parse("SL-001", Some("PHASE-02")).unwrap(),
            Target {
                reference: "SL-001".to_owned(),
                phase: Some("PHASE-02".to_owned()),
            }
        );
        assert_eq!(
            Target::parse("SL-001@PHASE-02", None).unwrap(),
            Target {
                reference: "SL-001".to_owned(),
                phase: Some("PHASE-02".to_owned()),
            }
        );
    }

    /// Both spellings naming a phase at once is refused (D-T3-1): the raw
    /// string cannot silently pick one.
    #[test]
    fn at_phase_conflicts_with_phase_flag() {
        let err = Target::parse("SL-001@PHASE-02", Some("PHASE-01"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("SL-001@PHASE-02"), "{err}");
        assert!(err.contains("already carries a phase"), "{err}");
    }

    /// A malformed `@` spelling — an empty side, or a second `@` — is refused,
    /// never silently truncated.
    #[test]
    fn at_phase_malformed_rows_refused() {
        for raw in ["@X", "SL-001@", "SL-001@PHASE-01@X"] {
            let err = Target::parse(raw, None).unwrap_err().to_string();
            assert!(err.contains(raw), "{raw}: {err}");
        }
    }

    /// Turn rows read open (D15): an unknown act/role parses and keeps its raw
    /// strings; a ledger with no `turn` key reads empty journals.
    #[test]
    fn turn_rows_read_open_vocabulary() {
        let legacy: ReviewDoc = toml::from_str(&format!("{HEAD}{FINDING}")).unwrap();
        assert!(legacy.review.turn.is_empty());
        assert!(legacy.finding[0].turn.is_empty());
        assert_eq!(
            (legacy.review.rounds_base, legacy.review.contests_base),
            (None, None)
        );

        let text = format!(
            "{HEAD}[[review.turn]]\nact = \"ponder\"\nrole = \"oracle\"\n{FINDING}\
             [[finding.turn]]\nact = \"frobnicate\"\nrole = \"gremlin\"\nroute = \"sideways\"\n"
        );
        let doc: ReviewDoc = toml::from_str(&text).unwrap();
        assert_eq!(
            (
                doc.review.turn[0].act.as_str(),
                doc.review.turn[0].role.as_str()
            ),
            ("ponder", "oracle")
        );
        let turn = &doc.finding[0].turn[0];
        assert_eq!(
            (turn.act.as_str(), turn.role.as_str(), turn.route.as_deref()),
            ("frobnicate", "gremlin", Some("sideways"))
        );
    }

    /// VT-2 (SL-278 PHASE-01 T6, A6): a valid `001` and a garbage `002` scan to
    /// one doc plus one [`UnreadableReview`] naming `RV-002`, with a non-empty
    /// `reason` and the `{"rv","reason"}` wire shape — the garbage ledger is
    /// disclosed, never skipped (STD-003).
    #[test]
    fn scan_reviews_names_an_unparseable_ledger() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("001")).unwrap();
        fs::write(
            root.join("001").join("review-001.toml"),
            format!("{HEAD}{FINDING}"),
        )
        .unwrap();
        fs::create_dir_all(root.join("002")).unwrap();
        fs::write(root.join("002").join("review-002.toml"), "not = [valid").unwrap();

        let (docs, unreadable) = scan_reviews(root).unwrap();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0].id, 1);
        assert_eq!(unreadable.len(), 1);
        assert_eq!(unreadable[0].rv, "RV-002");
        assert!(!unreadable[0].reason.is_empty());
        assert_eq!(
            serde_json::to_value(&unreadable[0]).unwrap(),
            serde_json::json!({"rv": "RV-002", "reason": unreadable[0].reason}),
        );
    }
}
