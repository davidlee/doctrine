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

/// The `[review]` metadata table (design §5): the facet, the two role labels, and
/// the concluded-pass marker.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub(crate) struct ReviewMeta {
    pub(crate) facet: String,
    pub(crate) raiser: String,
    pub(crate) responder: String,
    /// Whether the raiser has declared this pass finished (SL-244 `sec-4`,
    /// IMP-392). Written only by [`run_conclude`]; never a function of the
    /// findings.
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
    #[serde(default)]
    pub(crate) response: Option<String>,
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

    /// The review's derived `(ReviewStatus, Await)` (design §8) — computed at read
    /// time, never stored.
    pub(crate) fn derived(&self) -> (ReviewStatus, Await) {
        derived_status(&self.finding_states())
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
