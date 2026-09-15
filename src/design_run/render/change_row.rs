// SPDX-License-Identifier: GPL-3.0-only
//! The **rendered** change row — a descendant of [`super`], so it can see the
//! private `ENVELOPE_*` bounds its arithmetic is written against.
//!
//! The stored row ([`super::super::change_log::ChangeRow`]) and this artefact are
//! deliberately different things: the store keeps full fidelity, this keeps a
//! budget. Nothing here writes back.

use super::super::bounds::DESIGN_EVENT_NAME_BYTES;
use super::super::change_log::{ChangeRow, RawRow};
use super::super::ids::DesignId;
use super::render_value;

/// How a run-wide event's absent subject renders. One byte, so it can only make
/// the row narrower than the worst case the budget is derived from.
const RUN_WIDE_SUBJECT: &str = "-";

/// What stands in the event column of a row this binary could not read.
const UNREADABLE_EVENT: &str = "unreadable";

/// The payload key the reason is disclosed under, so a degraded row reads in the
/// same `key=value` idiom as every other payload rather than in a second one.
const UNREADABLE_REASON_KEY: &str = "why";

/// The event column's budget is owed by the marker too, **proved** in the idiom
/// [`super::super::change_log`] uses for the vocabulary it bounds: a wider marker
/// would stop the build rather than quietly overrun a rendered row.
const _: () = assert!(UNREADABLE_EVENT.len() <= DESIGN_EVENT_NAME_BYTES);

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

/// A row this binary could not read, disclosed as unreadable **and named**
/// (`STD-003`, EX-5).
///
/// It keeps the five-column shape, so a degraded row reads in the delta rather
/// than interrupting it, and it carries the cause [`RawRow`] classified at the
/// point of failure — never re-derived here, which is the whole reason the
/// reason is stored (`RV-365` `F-4`).
///
/// One rendering, not two: there is no `--full` form of a row whose payload this
/// binary cannot interpret, so there is nothing for detail to abbreviate.
pub(crate) fn render_unreadable(row: &RawRow) -> String {
    [
        row.revision.to_string(),
        row.index.to_string(),
        UNREADABLE_EVENT.to_owned(),
        RUN_WIDE_SUBJECT.to_owned(),
        [
            UNREADABLE_REASON_KEY,
            super::PAYLOAD_KEY_SEPARATOR,
            row.why.as_str(),
        ]
        .concat(),
    ]
    .join(super::FIELD_SEPARATOR)
}
