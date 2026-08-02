// SPDX-License-Identifier: GPL-3.0-only
//! Deriving a section's title from its own body (SL-233 PHASE-06, EX-13(b)).
//!
//! # Why this is a module and not a function in `run.rs` or `snapshot.rs`
//!
//! ADR-001 layers leaf ← engine ← command, and this is the leaf-most thing in
//! the tree: a total function from a section body's text to its title. It is
//! **admission-time validation**, so it cannot live in [`super::snapshot`]
//! beside the [`super::snapshot::Section`] type it feeds — that type is the
//! storage tier, strictly below admission, and putting the rule there would let
//! storage decide what is admissible. It is not [`super::run`] either:
//! `run.rs` orchestrates a whole submission across four subject kinds, and a
//! single-line grammar recogniser depends on none of that. Its own out-degree
//! is [`super::ids`] and [`super::refusal`] — the two siblings every admission
//! path already names — and it stays a sibling of `render`, so the privacy
//! precondition in this tree's module doc is untouched.
//!
//! # The rule
//!
//! `.doctrine/slice/233/sketches/marker-grammar.md` § *The derivation, stated
//! totally* is authoritative; this is its transcription. The heading is body
//! content, so the title is **derived** from it rather than declared beside it
//! — one source, and nothing for a second one to disagree with.
//!
//! Nothing here claims the derived title is stable under a Markdown formatter.
//! That claim was measured, found false for reasons no local rule can close,
//! and withdrawn (the sketch's § *The oracle this rule owes its shape to*).

use super::ids::DesignId;
use super::refusal::Refusal;

/// The delimiter set an ATX heading admits after its opening run, and the only
/// whitespace *blank* and the extraction steps recognise.
const DELIMITERS: [char; 2] = [' ', '\t'];
/// The most leading spaces an ATX heading line may carry (`CommonMark`).
///
/// `pub(crate)` because it is not the *heading's* limit, it is `CommonMark`'s:
/// four spaces make an indented code block, and every line grammar beneath it
/// binds the same three. [`super::legacy`]'s fence recogniser is the second
/// consumer, and spelling the same 3 twice would let the two drift (STD-001).
pub(crate) const MAX_INDENT: usize = 3;
/// The longest opening `#` run an ATX heading line may carry.
const MAX_HASHES: usize = 6;

/// The title of the section whose body is `body`.
///
/// The procedure, evaluated in order, where *f* is the first line that is not
/// blank (blank = nothing but spaces and tabs):
///
/// 1. no such *f* — [`Refusal::SectionBodyEmpty`];
/// 2. *f* is not an ATX heading line — [`Refusal::SectionBodyHeadingMissing`];
/// 3. *f*'s extracted text is empty — [`Refusal::SectionTitleEmpty`];
/// 4. otherwise that text is the title, and every other line — heading or not —
///    is ordinary body content.
///
/// Total and disjoint by construction: each guard is the negation of its
/// predecessors conjoined with one decidable test, and the fourth is the
/// unguarded remainder.
pub(crate) fn derive_title<'body>(id: &DesignId, body: &'body str) -> Result<&'body str, Refusal> {
    let Some(first) = body.lines().find(|line| !is_blank(line)) else {
        return Err(Refusal::SectionBodyEmpty { id: id.clone() });
    };
    let Some(region) = content_region(first) else {
        return Err(Refusal::SectionBodyHeadingMissing { id: id.clone() });
    };
    let title = extract(region);
    if title.is_empty() {
        return Err(Refusal::SectionTitleEmpty { id: id.clone() });
    }
    Ok(title)
}

/// The title an ATX heading line names, or `None` when the line is not an ATX
/// heading line, or is one that names nothing.
///
/// The **same** recogniser and the same extraction [`derive_title`] runs, exposed
/// as a line predicate for the one consumer that must decompose a document at
/// its headings ([`super::legacy`], SL-233 PHASE-11 D4). Import needs the
/// question "does this line open a section?", which is exactly arms 2 and 3 of
/// the derivation collapsed into one answer — not a new rule. A second
/// recogniser would be a parallel implementation of a grammar RV-323 spent
/// thirty rounds settling, and the two would be free to disagree about a line
/// neither author had in mind.
///
/// [`derive_title`] is **not** re-expressed over this, on purpose: it must tell
/// "no heading" from "empty title" to name the right refusal, and that
/// distinction is precisely what this collapses.
pub(crate) fn heading_title(line: &str) -> Option<&str> {
    let title = extract(content_region(line)?);
    (!title.is_empty()).then_some(title)
}

/// Whether a line holds nothing but spaces and tabs.
fn is_blank(line: &str) -> bool {
    line.chars()
        .all(|character| DELIMITERS.contains(&character))
}

/// The **content region** of an ATX heading line: what follows the leading
/// spaces and the `#` run, or `None` when the line is not an ATX heading line.
///
/// A line *L* is an ATX heading line iff, left to right: zero to three U+0020
/// spaces, then a run of one to six `#`, then **either** the line ends there
/// **or** the next character is a space or a tab. Nothing else is one — which is
/// what makes the arm-2 guard decidable on every possible line rather than on
/// the ones an enumeration happened to include.
fn content_region(line: &str) -> Option<&str> {
    let undented = line.trim_start_matches(' ');
    if line.len() - undented.len() > MAX_INDENT {
        return None;
    }
    let region = undented.trim_start_matches('#');
    let hashes = undented.len() - region.len();
    if hashes == 0 || hashes > MAX_HASHES {
        return None;
    }
    match region.chars().next() {
        None => Some(region),
        Some(character) if DELIMITERS.contains(&character) => Some(region),
        Some(_) => None,
    }
}

/// The title text of a content region: drop the trailing whitespace, then drop
/// the optional closing sequence **to exhaustion**, then trim.
///
/// Two things about that order are load-bearing, and each closed a family a
/// hand-written table of heading forms did not contain.
///
/// *The region's leading whitespace survives until after the closing-sequence
/// test.* Dropping it first leaves `## ###` as `###`, whose trailing run is
/// preceded by nothing, so the closing-sequence step cannot fire and the title
/// derives as `###`. `CommonMark` reads `## ###` as a heading with **empty**
/// content, because the closing sequence is preceded by the delimiter space.
///
/// *The strip runs to exhaustion.* One pass leaves a cascade: `## # # #`
/// derives `# #`, which re-emitted derives `#`, which derives nothing. Stripping
/// while the guard holds lands on the same fixed point from any spelling — the
/// property `section_title_derivation_is_idempotent_over_generated_bodies`
/// asserts, and the one a single pass measurably fails.
fn extract(region: &str) -> &str {
    let mut region = region.trim_end_matches(DELIMITERS);
    loop {
        let head = region.trim_end_matches('#');
        // No trailing `#` run, or one that is neither preceded by whitespace nor
        // the whole region — then it is content, not a closing sequence.
        if head.len() == region.len() || !(head.is_empty() || head.ends_with(DELIMITERS)) {
            break;
        }
        region = head.trim_end_matches(DELIMITERS);
    }
    region.trim_matches(DELIMITERS)
}
