// SPDX-License-Identifier: GPL-3.0-only
//! The budgeted rendering — **emission**, and nothing else (SL-233 PHASE-03).
//!
//! # Why the constants below carry no `pub`
//!
//! EX-16(c): the layer boundary is enforced by **Rust privacy — a compile error,
//! not a test and not a grep**. `ENVELOPE_*` bounds the budgeted rendering; a
//! storage or admission path cannot reference what it cannot name, so the
//! violation stops compiling. That is closed under aliasing and indirection,
//! because you cannot alias a name that is not in scope and a re-export would be
//! a visible, reviewable line *inside this module*.
//!
//! The guarantee holds only while this module stays a **sibling** of
//! [`super::snapshot`], [`super::change_log`], [`super::ids`] and
//! [`super::submission`] — Rust privacy is module-scoped and descendants see an
//! ancestor's private items, so nesting storage beneath `render` would make the
//! compile error silently stop firing while every test still passed. See the
//! privacy precondition in [`super`]'s module doc.
//!
//! The converse direction is intended: [`super::bounds`]'s `DESIGN_*` admission
//! constants are `pub(crate)` and this module names them for its row arithmetic.
//! The rule forbids emission bounds reaching storage, not the reverse.
//!
//! **What privacy does not catch, stated plainly:** a *copied literal* — someone
//! typing `96` into storage code. Nothing here catches that; it is STD-001's
//! no-magic-numbers rule and it is caught by review, not by construction.
//!
//! # The derivation (sketch § *The rendered payload, derived rather than asserted*)
//!
//! A rendered payload is space-separated `key=value`. The widest member of the
//! closed vocabulary is `stage_moved` carrying a regression reason:
//! `from=`(5) + 16 + `to=`(3) + 16 + `reason=`(7) + 96 + two separators (2) =
//! **145 B**, so [`ENVELOPE_PAYLOAD_BYTES`] = 160 holds every member with 15 B
//! spare. The whole row is revision (20) + index (10) + event name (32) +
//! subject id (32) + payload (160) + framing (10) = **264 B**.
//!
//! Both stage-label and id terms are *admission* bounds, so a value arriving
//! here is already within them by construction: the worst case holds because no
//! value that large can exist, not because this module would truncate it to fit.

use super::bounds::{DESIGN_EVENT_NAME_BYTES, DESIGN_ID_BYTES, DESIGN_STAGE_LABEL_BYTES};
use super::change_log::{ChangeEvent, ChangeRow, PayloadKey, ValueKind};

pub(crate) mod change_row;
pub(crate) mod envelope;

// ── cardinality caps (sketch §(a)) ────────────────────────────────────────
//
// Every one bounds an entry COUNT, and every one is flat in run size: a run with
// eight inquiry nodes and one with eight hundred project the same ceiling. That
// constancy — not the constants — is what R1 rests on.

/// Nearby-frontier entries.
///
/// Derivation: the frontier is the decision surface for *one* turn, not a
/// listing of the map. Seven carries the cursor's children and its siblings
/// without either crowding out the other.
const ENVELOPE_FRONTIER_NODES: usize = 7;

/// Active-path entries, retained from the **cursor** end.
///
/// Derivation: six levels of decomposition is deeper than a readable design
/// tree; above that, the top of the path is context the stage and the next
/// obligation already carry.
const ENVELOPE_ACTIVE_PATH_DEPTH: usize = 6;

/// Blocker entries.
///
/// Derivation: more than five simultaneous blockers is a state the agent needs
/// to look at whole, which is why the global blocker total is in the no-drop set
/// — the envelope says "there are more" rather than implying five is all.
const ENVELOPE_BLOCKERS: usize = 5;

/// Material-change delta rows, retained from the **newest** end.
///
/// Derivation: sized to what is absorbable in one turn. Deliberately a different
/// constant from [`super::bounds::CHANGE_LOG_REVISIONS`], which bounds *storage*.
const ENVELOPE_CHANGE_ROWS: usize = 10;

/// Linked durable DEC/QUE/ASM references.
///
/// Derivation: sized to one turn, matching the corpus's existing `FETCH_LIMIT`
/// precedent for "records worth showing beside the thing you asked about".
const ENVELOPE_DURABLE_RECORDS: usize = 8;

/// Section / review-state rows.
///
/// Derivation: sections scale with the design *document*, not with the run, and
/// no `design.md` in this corpus reaches sixteen.
const ENVELOPE_SECTION_ROWS: usize = 16;

// ── emission caps (sketch §(a)) ───────────────────────────────────────────
//
// Bytes of the budgeted rendering, and nothing else. Every one of these bounds
// gracefully degrading prose, which is why every one of them may elide —
// identity and closed vocabularies are bounded at admission instead.

/// An inquiry node's question, as rendered.
///
/// Derivation: design §5.3 says the question is *concise* — one line, measured
/// in the units the budget is actually spent in.
const ENVELOPE_QUESTION_BYTES: usize = 160;

/// Any title as rendered: record, section, fragment name.
///
/// Derivation: a title is a headline, not a body; 120 B is a wide terminal line
/// less the row's own labelling.
const ENVELOPE_LABEL_BYTES: usize = 120;

/// The next obligation, a blocker's reason, and a *live* regression reason.
///
/// Derivation: these are the three prose fields an agent acts on directly, so
/// they get the widest prose allowance the per-entry worst cases in sketch §(e)
/// can carry.
const ENVELOPE_REASON_BYTES: usize = 240;

/// The worked next-mutation example.
///
/// Derivation: sketch §(e) budgets the mutation contract plus its worked example
/// at 1344 B together. **This cap never truncates** — half a contract is worse
/// than none, so it is in the no-drop set and an oversized example is an
/// authoring defect, refused rather than clipped. The assertion below is that
/// refusal, moved to compile time.
const ENVELOPE_DECLARATION_EXAMPLE_BYTES: usize = 1024;

/// The entire budgeted rendering.
///
/// Derivation: sketch §(e)'s saturated table totals 15,576 B, and 24 KiB sits
/// 9000 B (57.8%) above it. **The headroom is not what makes the ceiling true**
/// — [`envelope::project`]'s eviction ladder is. The headroom exists so that a
/// ceiling this far above the saturated table cannot become the reason a design
/// alternative is rejected, which is the failure mode a 16 KiB ceiling produced
/// twice.
const ENVELOPE_NORMAL_BUDGET_BYTES: usize = 24576;

/// Bytes of a regression reason as *rendered* on a change row.
///
/// Derivation: the residual of the payload budget after the two stage labels and
/// their keys — 160 − (5 + 16 + 3 + 16 + 2) = 118, rounded down to 96 so the
/// widest payload lands 15 B inside the budget rather than flush against it.
/// Prose is the only term that degrades gracefully, so it is the only one that
/// absorbs the rounding.
const ENVELOPE_CHANGE_REASON_BYTES: usize = 96;

/// Bytes of one *rendered* change row's event payload. Derivation above.
const ENVELOPE_PAYLOAD_BYTES: usize = 160;

/// Bytes of a fingerprint as rendered on a change row.
///
/// Derivation: a fingerprint is a uniformly distributed digest, so a 12-hex
/// prefix is ~48 bits — collision-resistant enough to identify a section in a
/// rendering the reader can widen with one command, while the stored row keeps
/// the whole digest. An abbreviation with a stated collision budget, not a
/// truncation that loses recoverable information.
const ENVELOPE_FINGERPRINT_SHORT_BYTES: usize = 12;

/// The explicit marker an elided value carries, so a reader can tell a short
/// reason from a shortened one.
const ELISION_MARKER: &str = "…";

/// A `u64` decimal's maximum rendered width — a constant per wire type, which is
/// what closes the "counts and revisions grow in width" gap.
const U64_DECIMAL_BYTES: usize = 20;
/// A `u32` decimal's maximum rendered width.
const U32_DECIMAL_BYTES: usize = 10;
/// Row framing and separators, fixed by the encoding: four separating spaces,
/// inside the 10 B the sketch's derivation reserves.
///
/// **Deliberately unchanged at 10** (RV-321 F-2, framing half — *rejected*).
/// [`change_row::render`] joins five fields with one space, so the encoder emits
/// exactly four separator bytes. The surplus six is a **disclosed conservative
/// reserve**, and it runs in the safe direction: the emitted row is narrower
/// than the bound, so containment holds a fortiori. Sketch §(e) costs the change
/// delta at 10 × [`WIDEST_ROW_BYTES`], so "correcting" this to 4 would falsify
/// the sketch's own 15,576 B saturated total.
const ROW_FRAMING_BYTES: usize = 10;

/// What separates a payload term's key from its value.
const PAYLOAD_KEY_SEPARATOR: &str = "=";
/// What separates two payload terms, and two row fields.
const FIELD_SEPARATOR: &str = " ";

/// One `key=` prefix's rendered width, **derived from the key vocabulary**
/// rather than retyped (RV-321 F-2, literal half): a payload-key respelling
/// moves this arithmetic instead of leaving it silently stale.
const fn key_prefix_bytes(key: PayloadKey) -> usize {
    key.as_str().len() + PAYLOAD_KEY_SEPARATOR.len()
}

/// The separators between the widest payload's terms — one fewer than the term
/// count of the event that produces it, taken from the event's own declared
/// shape rather than counted by hand.
const WIDEST_PAYLOAD_SEPARATORS: usize =
    (WIDEST_PAYLOAD_EVENT.payload_terms().len() - 1) * FIELD_SEPARATOR.len();

/// The member of the closed vocabulary that produces the widest payload:
/// `stage_moved` carrying a regression reason (sketch § *The rendered payload*).
const WIDEST_PAYLOAD_EVENT: ChangeEvent = ChangeEvent::StageMoved;

/// The widest payload the closed vocabulary can produce: `stage_moved` carrying
/// a regression reason. Every key width derives from [`PayloadKey::as_str`].
const WIDEST_PAYLOAD_BYTES: usize = key_prefix_bytes(PayloadKey::From)
    + DESIGN_STAGE_LABEL_BYTES
    + key_prefix_bytes(PayloadKey::To)
    + DESIGN_STAGE_LABEL_BYTES
    + key_prefix_bytes(PayloadKey::Reason)
    + ENVELOPE_CHANGE_REASON_BYTES
    + WIDEST_PAYLOAD_SEPARATORS;

/// The **reserved budget** for one whole rendered row — not the width of any row
/// the encoder actually emits.
///
/// Read it as a ceiling, not as a measurement: every term is its named bound's
/// worst case, and [`ROW_FRAMING_BYTES`] carries a disclosed reserve above the
/// four bytes the encoder emits. A real row is narrower; sketch §(e)'s
/// change-delta line is `ENVELOPE_CHANGE_ROWS × this`, which is what makes the
/// saturated table a ceiling rather than a prediction.
const WIDEST_ROW_BYTES: usize = U64_DECIMAL_BYTES
    + U32_DECIMAL_BYTES
    + DESIGN_EVENT_NAME_BYTES
    + DESIGN_ID_BYTES
    + ENVELOPE_PAYLOAD_BYTES
    + ROW_FRAMING_BYTES;

/// The change-delta row figure the projection-bounds sketch §(e) costs its
/// saturated table with (`10 × 264 B = 2640 B`, and through it the 15,576 B
/// total).
///
/// It is named rather than repeated as a bare literal (STD-001) and it exists to
/// be *compared*: the assertion below pins this module's own derivation to the
/// sketch's, so a term of the derivation moving without the sketch moving stops
/// the build instead of quietly restating the sketch's arithmetic.
const SKETCH_WIDEST_ROW_BYTES: usize = 264;

/// The containment check, at **compile time**: every term of the widest payload
/// has a named bound, and their sum fits the budget with room to spare. This is
/// the arithmetic half of the check F-1's defect class escaped three times; the
/// behavioural half — every member of the vocabulary rendered with every scalar
/// saturated — is a test, because a sum cannot prove the renderer applies the
/// bounds it is written against.
const _: () = assert!(WIDEST_PAYLOAD_BYTES <= ENVELOPE_PAYLOAD_BYTES);
const _: () = assert!(WIDEST_ROW_BYTES == SKETCH_WIDEST_ROW_BYTES);

/// Test-only visibility for the containment assertions VA-7/VA-9 require in
/// `tests/e2e_design_state.rs`.
///
/// `#[cfg(test)]`, so no production path can name it — a storage-path reference
/// fails `cargo build` exactly as a reference to the private constant does. This
/// is the "visible, reviewable line inside the rendering module" EX-16(c) names
/// as the only lawful widening, and it is scoped so narrowly that it cannot
/// widen anything shipped.
#[cfg(test)]
pub(crate) const ENVELOPE_PAYLOAD_BYTES_UNDER_TEST: usize = ENVELOPE_PAYLOAD_BYTES;

/// Test-only visibility for the elision marker (see above).
#[cfg(test)]
pub(crate) const ELISION_MARKER_UNDER_TEST: &str = ELISION_MARKER;

/// Test-only visibility for the cardinality caps and the whole-envelope ceiling,
/// so `tests/e2e_design_projection.rs` asserts against the constants the binary
/// compiles rather than against numbers re-typed beside the assertion (EX-4).
///
/// Same `#[cfg(test)]` regime as the two above, and for the same reason: a
/// production reference to any of these fails `cargo build` exactly as a
/// reference to the private constant does, so the layer boundary is unchanged.
#[cfg(test)]
#[allow(
    dead_code,
    reason = "consumed by tests/e2e_design_projection.rs — a separate compilation unit, so \
              `expect` could not be fulfilled in both and would itself become the warning"
)]
pub(crate) mod under_test {
    pub(crate) const FRONTIER_NODES: usize = super::ENVELOPE_FRONTIER_NODES;
    pub(crate) const ACTIVE_PATH_DEPTH: usize = super::ENVELOPE_ACTIVE_PATH_DEPTH;
    pub(crate) const BLOCKERS: usize = super::ENVELOPE_BLOCKERS;
    pub(crate) const CHANGE_ROWS: usize = super::ENVELOPE_CHANGE_ROWS;
    pub(crate) const DURABLE_RECORDS: usize = super::ENVELOPE_DURABLE_RECORDS;
    pub(crate) const SECTION_ROWS: usize = super::ENVELOPE_SECTION_ROWS;
    pub(crate) const QUESTION_BYTES: usize = super::ENVELOPE_QUESTION_BYTES;
    pub(crate) const NORMAL_BUDGET_BYTES: usize = super::ENVELOPE_NORMAL_BUDGET_BYTES;
}

/// The largest prefix of `text` that is at most `cap` bytes and ends on a UTF-8
/// character boundary.
fn clip(text: &str, cap: usize) -> &str {
    if text.len() <= cap {
        return text;
    }
    let end = text
        .char_indices()
        .map(|(at, _)| at)
        .take_while(|at| *at <= cap)
        .last()
        .unwrap_or(0);
    text.get(..end).unwrap_or("")
}

/// Elide prose to `cap` bytes *including* the marker, so the rendered term is
/// never wider than its bound and a reader can always tell that it was cut.
fn elide(text: &str, cap: usize) -> String {
    if text.len() <= cap {
        return text.to_owned();
    }
    [
        clip(text, cap.saturating_sub(ELISION_MARKER.len())),
        ELISION_MARKER,
    ]
    .concat()
}

/// How one stored term renders — the layer rule, applied by *value kind*.
///
/// Identity and closed vocabulary render **whole**: they are bounded at
/// admission, so truncating them here could only make two distinct subjects
/// render identically. Digests abbreviate against a stated collision budget, and
/// only prose elides.
fn render_value(kind: ValueKind, value: &str) -> String {
    match kind {
        ValueKind::Token | ValueKind::Label => value.to_owned(),
        ValueKind::Digest => abbreviate_digest(value),
        ValueKind::Prose => elide(value, ENVELOPE_CHANGE_REASON_BYTES),
    }
}

/// A fingerprint as rendered on a change or section row.
///
/// An **abbreviation with a stated collision budget**, not a truncation that
/// loses recoverable information: the stored row keeps the whole digest, and a
/// reader widens it with one command.
fn abbreviate_digest(digest: &str) -> String {
    clip(digest, ENVELOPE_FINGERPRINT_SHORT_BYTES).to_owned()
}

// The three delta headers, single-sourced (STD-001) so the budgeted projection
// and the `--full` read cannot drift apart in the one place where a difference
// would be read as a difference in the RUN rather than in the rendering.

/// "I cannot tell you what changed" — names the floor, because a reader who
/// cannot see the range needs to know where the log does start.
fn delta_unavailable_line(floor: u64, known_revision: u64) -> String {
    format!(
        "changes: UNAVAILABLE — the change log covers revisions from {floor} onward, \
         and revision {known_revision} is below that floor; see `design show --full`"
    )
}

/// "Nothing changed" — a different fact from the one above (design R2).
fn delta_none_line(known_revision: u64) -> String {
    format!("changes: none since revision {known_revision}")
}

/// The header above a non-empty delta.
fn delta_since_line(known_revision: u64) -> String {
    format!("changes since revision {known_revision}:")
}

/// One whole rendered row.
pub(crate) fn render_row(row: &ChangeRow) -> String {
    change_row::render(row)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test code — the repo's panic-avoidance denials target production paths"
)]
mod tests {
    use super::{
        ELISION_MARKER_UNDER_TEST, ENVELOPE_CHANGE_REASON_BYTES, ENVELOPE_FINGERPRINT_SHORT_BYTES,
        ENVELOPE_PAYLOAD_BYTES_UNDER_TEST, elide, render_value,
    };
    use crate::design_run::change_log::ValueKind;

    /// Prose over the cap is elided *and marked*, and the result never exceeds
    /// the cap it was elided to — the marker is inside the budget, not beside it.
    #[test]
    fn elided_prose_carries_the_marker_inside_its_bound() {
        let long = "x".repeat(ENVELOPE_PAYLOAD_BYTES_UNDER_TEST * 4);
        let elided = elide(&long, ENVELOPE_CHANGE_REASON_BYTES);
        assert!(
            elided.len() <= ENVELOPE_CHANGE_REASON_BYTES,
            "{} bytes",
            elided.len()
        );
        assert!(elided.ends_with(ELISION_MARKER_UNDER_TEST));
        assert_eq!(
            elide("short", ENVELOPE_CHANGE_REASON_BYTES),
            "short",
            "a short value is untouched"
        );
    }

    /// Identity and closed vocabulary render WHOLE however long they are: a
    /// truncated identity is a *wrong* identity, and the bound that keeps them
    /// short is at admission, not here.
    #[test]
    fn identity_and_vocabulary_render_whole() {
        let long = "a".repeat(200);
        assert_eq!(render_value(ValueKind::Token, &long), long);
        assert_eq!(render_value(ValueKind::Label, &long), long);
        assert_eq!(
            render_value(ValueKind::Digest, &long).len(),
            ENVELOPE_FINGERPRINT_SHORT_BYTES
        );
    }

    /// A multi-byte character must not be split by elision.
    #[test]
    fn elision_lands_on_a_character_boundary() {
        let text = "é".repeat(100);
        let elided = elide(&text, 21);
        assert!(elided.len() <= 21);
        assert!(elided.ends_with(ELISION_MARKER_UNDER_TEST));
    }
}
