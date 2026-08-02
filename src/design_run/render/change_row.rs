// SPDX-License-Identifier: GPL-3.0-only
//! The **rendered** change row — a descendant of [`super`], so it can see the
//! private `ENVELOPE_*` bounds its arithmetic is written against.
//!
//! The stored row ([`super::super::change_log::ChangeRow`]) and this artefact are
//! deliberately different things: the store keeps full fidelity, this keeps a
//! budget. Nothing here writes back.

use super::super::change_log::ChangeRow;
use super::super::ids::DesignId;
use super::render_value;

/// How a run-wide event's absent subject renders. One byte, so it can only make
/// the row narrower than the worst case the budget is derived from.
const RUN_WIDE_SUBJECT: &str = "-";

/// The subject column: the id whole (identity is never truncated), or the
/// run-wide marker.
fn subject(row: &ChangeRow) -> String {
    row.subject
        .as_ref()
        .map_or_else(|| RUN_WIDE_SUBJECT.to_owned(), DesignId::to_string)
}

/// The rendered payload: space-separated `key=value`, each term rendered by its
/// value kind. Fits [`super::ENVELOPE_PAYLOAD_BYTES`] by the derivation in
/// [`super`]'s module doc — identity terms are already within their admission
/// bounds, and prose is the one term that elides.
pub(crate) fn render_payload(row: &ChangeRow) -> String {
    row.terms
        .iter()
        .map(|term| {
            [
                term.key().as_str(),
                super::PAYLOAD_KEY_SEPARATOR,
                &render_value(term.kind(), term.value()),
            ]
            .concat()
        })
        .collect::<Vec<String>>()
        .join(super::FIELD_SEPARATOR)
}

/// The whole rendered row: `revision index event subject payload`.
pub(crate) fn render(row: &ChangeRow) -> String {
    [
        row.revision.to_string(),
        row.index.to_string(),
        row.event.as_str().to_owned(),
        subject(row),
        render_payload(row),
    ]
    .join(super::FIELD_SEPARATOR)
}

/// The row at full stored fidelity — no abbreviation, no elision, and therefore
/// no elision marker. This is what `show --full` reads.
pub(crate) fn render_full(row: &ChangeRow) -> String {
    let payload = row
        .terms
        .iter()
        .map(|term| {
            [
                term.key().as_str(),
                super::PAYLOAD_KEY_SEPARATOR,
                term.value(),
            ]
            .concat()
        })
        .collect::<Vec<String>>()
        .join(super::FIELD_SEPARATOR);
    [
        row.revision.to_string(),
        row.index.to_string(),
        row.event.as_str().to_owned(),
        subject(row),
        payload,
    ]
    .join(super::FIELD_SEPARATOR)
}
