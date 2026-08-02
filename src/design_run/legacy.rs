// SPDX-License-Identifier: GPL-3.0-only
//! Reading an **unmarked** legacy `design.md` into ordered regions (SL-233
//! PHASE-11, DEC-084/DEC-085).
//!
//! # Why this is not [`super::document::parse`]
//!
//! `parse` is the exact inverse of `render`, and it refuses a marker-free
//! document by construction: its row 2 ([`Refusal::MarkerFreeAddition`]) fires
//! when non-blank bytes stand before the first marker, and in a legacy document
//! *every* byte does. Import is the complementary reader — it rides the
//! fingerprint/[`super::section::derive_title`] half of the marker machinery and
//! none of the marker half. Both halves stay single-sourced: the ATX recogniser
//! is [`super::section::heading_title`]'s, not a second one written here (D4).
//!
//! # Why this is a module and not a function in `commands/design.rs`
//!
//! ADR-001 layers leaf ← engine ← command, and this is a total function from
//! bytes to regions: no clock, disk, digest or snapshot. Its out-degree is
//! [`super::refusal`] and [`super::section`], both siblings, so the privacy
//! precondition in this tree's module doc is untouched.
//!
//! # The rule
//!
//! Decompose at **every** ATX heading line, any level, that names a title —
//! never at a heading-shaped line inside a fenced code block. The head (bytes
//! before the first such line) must be blank.
//!
//! Splitting at every level rather than at `##` is not taste. The template every
//! design in this repo came from puts an HTML comment and a status blockquote
//! *between* the `#` title and the first `##`, so a `##`-only reader orphans
//! that front matter into an unheaded head and refuses essentially the whole
//! corpus. Inferring nesting from heading depth is the other tempting move, and
//! it would **invent structure** — precisely what DEC-084 principle 6 forbids.
//! Flat is honest; an agent can restructure after import.
//!
//! Three of the four rules below were written against a survey of the 228 real
//! designs in this repo (the slice's `notes.md` § *Learned*), not against
//! reasoning about arbitrary prose, and two of them contradict what that
//! reasoning had produced. The corpus is still the oracle: `tests` runs this
//! reader over all of it.

use super::refusal::Refusal;
use super::section;

/// The least backticks or tildes a fenced code block's opening run carries
/// (`CommonMark` §4.5).
const MIN_FENCE: usize = 3;

/// The two characters a code block may be fenced with (`CommonMark` §4.5).
const FENCE_CHARACTERS: [char; 2] = ['`', '~'];

/// One region of a legacy document: its own heading line, and every byte from
/// there to the next region's heading line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Region<'doc> {
    /// The 1-based line its heading stands on — the source location `EX-2`
    /// requires an imported section to carry.
    pub(crate) line: usize,
    /// The region's bytes, verbatim. Nothing is trimmed anywhere on this path,
    /// the same discipline [`super::document::parse`] holds to.
    pub(crate) body: &'doc str,
}

/// A fenced code block that is open: what it is fenced with, how long its
/// opening run is, and where that run stands.
struct Fence {
    character: char,
    length: usize,
    line: usize,
}

/// Decompose an unmarked authored document into its regions, in document order.
///
/// Every byte of the document lands in exactly one part — a blank head, or one
/// region — so classifying the head and each region totally is sufficient, and
/// concatenating the regions returns the document minus its blank head.
///
/// # The four refusals, in one fixed order
///
/// 1. [`Refusal::CarriageReturnInDocument`] — any `\r`, whole document. The same
///    refusal the marker path raises, at a door it does not watch: `str::lines`
///    drops `\r` silently, so a CRLF document would import with a `\r` inside
///    every derived title and be unmaterialisable ever after.
/// 2. [`Refusal::UnclosedFence`] — the fence state is still open at EOF (D6).
/// 3. [`Refusal::UnheadedPreamble`] — non-blank bytes before the first heading
///    that names a title (D2).
///
/// 2 outranks 3 because an unclosed fence *causes* an unheaded head whenever it
/// opens before the first heading: reporting the head would name the symptom and
/// leave the reader looking in the wrong place.
///
/// A heading line whose derived title is empty (`###`, or `## ###`, which
/// `CommonMark` reads as an empty heading) does **not** split — it stays
/// ordinary body content (D1a). A line that cannot name a section cannot open
/// one, and the alternative — refusing the whole document over one degenerate
/// line — makes import non-total over arbitrary prose for no gain. The side
/// effect is worth stating: every region returned therefore begins with a
/// title-bearing heading *by construction*, so all three of `derive_title`'s
/// refusal arms are unreachable on the import path.
pub(crate) fn read(text: &str) -> Result<Vec<Region<'_>>, Refusal> {
    if text.contains('\r') {
        return Err(Refusal::CarriageReturnInDocument);
    }

    // Where each region starts: its byte offset, and its 1-based line.
    let mut heads: Vec<(usize, usize)> = Vec::new();
    let mut open: Option<Fence> = None;
    let mut at = 0_usize;
    for (index, chunk) in text.split_inclusive('\n').enumerate() {
        let line = chunk.strip_suffix('\n').unwrap_or(chunk);
        let number = index.saturating_add(1);
        if let Some(fence) = &open {
            if closes(fence, line) {
                open = None;
            }
        } else if let Some((character, length, _)) = fence(line) {
            open = Some(Fence {
                character,
                length,
                line: number,
            });
        } else if section::heading_title(line).is_some() {
            heads.push((at, number));
        }
        at = at.saturating_add(chunk.len());
    }

    if let Some(fence) = open {
        return Err(Refusal::UnclosedFence { line: fence.line });
    }

    // The head. Whitespace-only is not a preamble — a leading blank line is what
    // a formatter produces, the same carve-out `parse`'s row 2 makes.
    let head_end = heads.first().map_or(text.len(), |&(start, _)| start);
    let head = text.get(..head_end).unwrap_or_default();
    if let Some(line) = first_content_line(head) {
        return Err(Refusal::UnheadedPreamble { line });
    }

    Ok(heads
        .iter()
        .enumerate()
        .map(|(index, &(start, line))| {
            let end = heads
                .get(index.saturating_add(1))
                .map_or(text.len(), |&(next, _)| next);
            Region {
                line,
                body: text.get(start..end).unwrap_or_default(),
            }
        })
        .collect())
}

// ── The Open Questions section (EX-3/EX-4) ────────────────────────────────

/// The title an explicit Open Questions section carries, lowercased. Matched by
/// containment, because the corpus spells it `6. Open Questions & Unknowns`,
/// `17. Open Questions` and `Remaining open questions` — three spellings of one
/// section, and a whole-title equality test recognises none of them.
const OPEN_QUESTIONS: &str = "open questions";

/// The label a conventional entry opens with.
const ENTRY_PREFIX: &str = "OQ-";

/// The canonical prefix an explicit citation names a question record by.
///
/// Spelled here rather than read from `crate::knowledge`: this module is a leaf
/// with zero crate out-degree (ADR-001), and what it recognises is a **lexical
/// convention of authored prose**, not the kind registry. The two agreeing is a
/// property of the corpus, not an invariant this module can enforce.
const CITATION_PREFIX: &str = "QUE-";

/// The list markers an entry may be introduced by.
const LIST_MARKERS: [char; 3] = ['-', '*', '+'];

/// The emphasis characters that may wrap an entry's label.
const EMPHASIS: [char; 2] = ['*', '_'];

/// What may stand between an entry's label and its question text.
const LABEL_DELIMITERS: [char; 8] = [' ', '\t', ':', '.', '-', '\u{2014}', '\u{2013}', '*'];

/// The closing emphasis run that ends a bolded entry headline.
const EMPHASIS_CLOSE: &str = "**";

/// What separates an entry's label from its title inside the emphasised span.
/// The plain hyphen is deliberately **not** one: it stands inside every
/// canonical id a co-label carries (`RSK-229`), and cutting there would leave
/// the number as the question.
const HEADLINE_SEPARATORS: [char; 3] = [':', '\u{2014}', '\u{2013}'];

/// One conventional `OQ-*` entry of an Open Questions section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OpenQuestion<'doc> {
    /// The 1-based line of the document the entry stands on.
    pub(crate) line: usize,
    /// The entry's own label, verbatim — `OQ-1`. DEC-085's first term, and the
    /// only thing that distinguishes two entries with identical text.
    pub(crate) label: &'doc str,
    /// Its headline — see [`open_questions`] for what that is and is not.
    pub(crate) question: &'doc str,
    /// The canonical question record the entry cites, if it cites one. The ONLY
    /// thing that may associate an imported entry with an existing record
    /// (`EX-4`, DEC-085).
    pub(crate) citation: Option<&'doc str>,
}

/// Is this the **explicit** Open Questions section (`EX-3`)?
pub(crate) fn is_open_questions(title: &str) -> bool {
    title.to_lowercase().contains(OPEN_QUESTIONS)
}

/// The conventional `OQ-*` entries `region` carries, in document order.
///
/// An entry is a **list item** whose text, past an optional emphasis run, opens
/// with `OQ-` and an id. Both halves of that are evidence, not taste: every
/// Open Questions section in the corpus is a list, and requiring the marker is
/// what keeps prose *about* an entry — `Body prose mentions OQ-9 here` — from
/// seeding a node. Missing an unconventionally-spelled entry is the conservative
/// failure; inventing one from a sentence is not (DEC-084 principle 6).
///
/// The question is the entry's **headline**: its first line, past the label and
/// its delimiters, ending at the closing `**` when the label was bolded and
/// something stands before it. That reads both corpus conventions correctly —
/// `**OQ-1:** whether the reader tracks fences` yields the whole remainder, and
/// `**OQ-1 — discovery of applicable knowledge.** V1 accepts an…` yields the
/// title rather than the title welded to the first line of the body. The
/// continuation lines are deliberately **not** joined in: the node is a
/// proposal, the whole entry stays in the section body verbatim, and the node's
/// source line points straight at it.
pub(crate) fn open_questions<'doc>(region: &Region<'doc>) -> Vec<OpenQuestion<'doc>> {
    region
        .body
        .lines()
        .enumerate()
        .filter_map(|(offset, line)| {
            let (label, rest) = entry_label_and_text(line)?;
            Some(OpenQuestion {
                line: region.line.saturating_add(offset),
                label,
                question: headline(rest),
                citation: citation(rest),
            })
        })
        .collect()
}

/// A conventional entry's **label** and the bytes after it, or `None` when
/// `line` is not one.
///
/// The label is returned rather than discarded because DEC-085 requires an
/// imported node to carry it: the parse strips it to find the headline, so
/// dropping it here drops it from the provenance too, and two entries whose
/// text is identical then have nothing left to tell them apart (`EX-9`).
fn entry_label_and_text(line: &str) -> Option<(&str, &str)> {
    let marked = line.trim_start();
    let listed = marked.strip_prefix(LIST_MARKERS)?;
    if !listed.starts_with([' ', '\t']) {
        return None;
    }
    let labelled = listed.trim_start().trim_start_matches(EMPHASIS);
    let after = labelled.strip_prefix(ENTRY_PREFIX)?;
    let id_end = after
        .find(|character: char| !continues_id(character))
        .unwrap_or(after.len());
    if id_end == 0 {
        return None;
    }
    let label = labelled.get(..ENTRY_PREFIX.len().saturating_add(id_end))?;
    Some((label, after.get(id_end..)?))
}

/// An entry's headline — the corpus writes it two ways and this reads both.
///
/// | form | where the title is |
/// |---|---|
/// | `**OQ-1:** whether the reader tracks fences.` | after the closing `**` |
/// | `**OQ-2 / RSK-229 — managed authority.** Boot establishes…` | inside it |
///
/// The first is told from the second by what stands between the label and the
/// closing run: nothing but delimiters means the emphasis wrapped the label
/// alone. Inside the emphasised span, the title begins past the first `:`/`—`,
/// which is what carries a co-label (`/ RSK-229`) out of the question text. The
/// separator is looked for **only** within that span — a colon in the body
/// prose after the emphasis is not a label boundary.
fn headline(rest: &str) -> &str {
    match rest.split_once(EMPHASIS_CLOSE) {
        Some((label, title)) if label.trim_matches(LABEL_DELIMITERS).is_empty() => trimmed(title),
        Some((titled, _)) => past_separator(titled),
        None => past_separator(rest),
    }
}

/// The text past a label's first separator, or the whole of it when the label
/// carries none.
fn past_separator(span: &str) -> &str {
    let title = span
        .split_once(HEADLINE_SEPARATORS)
        .map_or(span, |(_, title)| title);
    trimmed(title)
}

/// Leading delimiters and emphasis off; trailing **whitespace only**, so a
/// title's own closing full stop survives.
fn trimmed(title: &str) -> &str {
    title.trim_start_matches(LABEL_DELIMITERS).trim_end()
}

/// Does `character` continue a canonical id rather than end it?
///
/// The single source of that boundary for both readers here. [`entry_label_and_text`] gets
/// the property for free — its run is maximal over exactly these characters, so
/// the character it stops on is a boundary by construction. [`citation`] does not:
/// its run is digits only, so it must test the boundary itself.
fn continues_id(character: char) -> bool {
    character.is_ascii_alphanumeric()
}

/// The canonical question record an entry cites, if it cites one.
///
/// Every occurrence of the prefix is a candidate, not just the first: a bare
/// prefix standing in the prose (`compare QUE- with QUE-177`) must not mask a
/// canonical citation later on the line, because a citation missed here seeds a
/// duplicate record where DEC-085 requires a merge.
///
/// A candidate qualifies on three counts — an id boundary before the prefix, a
/// non-empty digit run, and an id boundary closing that run. All three are one
/// rule: a citation is a **whole** canonical token. `QUE-177abc` is a longer token
/// rather than a citation to `QUE-177`, `XQUE-177` is a different token entirely,
/// and `QUE-` alone is prose rather than a citation to nothing.
///
/// [`crate::integrity::line_cites`] holds exactly this line for `SL-031` inside
/// `ASL-031`. The rule is duplicated here rather than shared because this module
/// is a leaf of crate out-degree zero — the e2e suites compile it standalone, so
/// nothing in it may name `crate::`. Do not "fix" that by reaching for the
/// sibling.
fn citation(rest: &str) -> Option<&str> {
    rest.match_indices(CITATION_PREFIX).find_map(|(at, _)| {
        let preceded = rest
            .get(..at)
            .and_then(|before| before.chars().next_back())
            .is_some_and(continues_id);
        if preceded {
            return None;
        }
        let cited = rest.get(at..)?;
        let body = cited.get(CITATION_PREFIX.len()..)?;
        let end = body
            .find(|character: char| !character.is_ascii_digit())
            .unwrap_or(body.len());
        let followed = body
            .get(end..)
            .is_some_and(|tail| tail.starts_with(continues_id));
        if end == 0 || followed {
            return None;
        }
        cited.get(..CITATION_PREFIX.len().saturating_add(end))
    })
}

/// The 1-based line of `head`'s first non-blank line, or `None` when every line
/// is blank.
fn first_content_line(head: &str) -> Option<usize> {
    head.lines()
        .position(|line| !line.trim().is_empty())
        .map(|index| index.saturating_add(1))
}

/// The fence `line` opens or closes — its character, its run length, and the
/// bytes after the run — or `None` when the line is not a fence line.
///
/// [`section::MAX_INDENT`] is shared rather than re-spelled: four spaces make an
/// indented code block, so three is the same limit the heading grammar binds.
fn fence(line: &str) -> Option<(char, usize, &str)> {
    let undented = line.trim_start_matches(' ');
    if line.len().saturating_sub(undented.len()) > section::MAX_INDENT {
        return None;
    }
    let character = *FENCE_CHARACTERS
        .iter()
        .find(|candidate| undented.starts_with(**candidate))?;
    let info = undented.trim_start_matches(character);
    let length = undented.len().saturating_sub(info.len());
    if length < MIN_FENCE {
        return None;
    }
    // A backtick fence's info string may not itself hold a backtick, or an
    // inline code span would open a block (`CommonMark` §4.5). A tilde fence's
    // may, which is why the guard is not symmetric.
    if character == '`' && info.contains('`') {
        return None;
    }
    Some((character, length, info))
}

/// Whether `line` closes `open`: the same character, a run **at least as long**,
/// and nothing but whitespace after it — a closing fence carries no info string
/// (`CommonMark` §4.5).
///
/// The obvious implementation is a left-to-right `is_fence → flip` toggle, and
/// it is measurably wrong: on 2 of the 228 real designs surveyed it ends the
/// document in the wrong state and classifies genuine `##` headings as code,
/// including the very `## Open Questions` heading `EX-3` depends on locating.
fn closes(open: &Fence, line: &str) -> bool {
    match fence(line) {
        Some((character, length, info)) => {
            character == open.character && length >= open.length && info.trim().is_empty()
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::super::ids::DesignId;
    use super::*;

    /// Every region's **derived** title — the derivation `seat_section` runs, so
    /// a test that names titles also asserts D1(a)'s side effect: every seated
    /// region is title-bearing by construction.
    fn titles(text: &str) -> Vec<String> {
        let id = DesignId::parse("sec-1").expect("a well-formed section id");
        read(text)
            .expect("the fixture imports")
            .iter()
            .map(|region| {
                section::derive_title(&id, region.body)
                    .expect("every imported region begins with a title-bearing heading")
                    .to_owned()
            })
            .collect()
    }

    /// The citation each conventional entry of a one-region `body` carries, in
    /// document order — `None` where the entry cites no canonical record.
    fn citations(body: &str) -> Vec<Option<&str>> {
        let region = Region { line: 1, body };
        open_questions(&region)
            .iter()
            .map(|entry| entry.citation)
            .collect()
    }

    /// The regions' bytes, concatenated — the document minus its blank head.
    fn joined(text: &str) -> String {
        read(text)
            .expect("the fixture imports")
            .iter()
            .map(|region| region.body)
            .collect()
    }

    #[test]
    fn splits_at_every_heading_level_and_keeps_every_byte() {
        let text = "# Title\n\nfront matter\n\n## One\n\nbody\n\n### One point one\n\nmore\n";
        assert_eq!(titles(text), ["Title", "One", "One point one"]);
        assert_eq!(joined(text), text, "every byte lands in exactly one region");

        let lines: Vec<usize> = read(text)
            .expect("imports")
            .iter()
            .map(|region| region.line)
            .collect();
        assert_eq!(lines, vec![1, 5, 9], "each region carries its source line");
    }

    #[test]
    fn a_degenerate_heading_stays_body_content() {
        // `###` alone, and `## ###` — which CommonMark reads as an EMPTY heading
        // — name nothing, so they do not open a section. D1(a): under the first
        // wording one such line anywhere refused the whole import.
        let text = "# Title\n\n###\n\n## ###\n\n## Real\n";
        assert_eq!(titles(text), ["Title", "Real"]);
        assert_eq!(joined(text), text);
    }

    #[test]
    fn a_heading_inside_a_fence_does_not_split() {
        // 33 of 228 real designs (14.5%) carry one of these; a fence-blind
        // splitter shreds the block and seats sections with unbalanced fences.
        let text = "# Title\n\n```text\n## Not a heading\n```\n\n## Real\n";
        assert_eq!(titles(text), ["Title", "Real"]);
        assert_eq!(joined(text), text);
    }

    #[test]
    fn only_a_run_at_least_as_long_closes_a_fence() {
        // A parity toggle reads the inner ``` as the close, ends the block
        // early, and promotes `## Inside` to a section.
        let nested = "# Title\n\n````md\n```\n## Inside\n```\n````\n\n## Real\n";
        assert_eq!(titles(nested), ["Title", "Real"]);

        // Tildes fence too, and do not close a backtick fence.
        let tilde = "# Title\n\n~~~\n## Inside\n~~~\n\n## Real\n";
        assert_eq!(titles(tilde), ["Title", "Real"]);
        let crossed = "# Title\n\n```\n~~~\n## Inside\n```\n\n## Real\n";
        assert_eq!(titles(crossed), ["Title", "Real"]);

        // A closing fence carries no info string: the second ```rust opens
        // nothing and closes nothing, so `## Inside` stays inside the block.
        let info = "# Title\n\n```rust\n```rust\n## Inside\n```\n\n## Real\n";
        assert_eq!(titles(info), ["Title", "Real"]);
    }

    #[test]
    fn an_open_fence_at_end_of_document_refuses() {
        // D6. CommonMark would close it at EOF; import refuses, because closing
        // it drops every section after line 3 without saying so.
        let text = "# Title\n\n```text\n## Swallowed\n";
        assert_eq!(read(text), Err(Refusal::UnclosedFence { line: 3 }));
    }

    #[test]
    fn a_contentless_section_is_legal_and_lossless() {
        // 87% of the corpus has at least one: a `##` immediately followed by its
        // first `###`. The norm, not an edge case.
        let text = "# Title\n\n## Parent\n\n### Child\n\nbody\n";
        assert_eq!(titles(text), ["Title", "Parent", "Child"]);
        assert_eq!(
            read(text)
                .expect("imports")
                .get(1)
                .map(|region| region.body),
            Some("## Parent\n\n")
        );
        assert_eq!(joined(text), text);
    }

    #[test]
    fn text_before_the_first_heading_refuses_but_blank_does_not() {
        assert_eq!(
            read("a preamble\n\n# Title\n"),
            Err(Refusal::UnheadedPreamble { line: 1 })
        );
        // No heading at all: the head is the whole document.
        assert_eq!(
            read("just prose\n"),
            Err(Refusal::UnheadedPreamble { line: 1 })
        );
        // A degenerate heading is body content (D1a), so it is a preamble too —
        // reported at its own line, not at the document's first.
        assert_eq!(
            read("\n###\n\n# Title\n"),
            Err(Refusal::UnheadedPreamble { line: 2 })
        );

        assert_eq!(titles("\n\n# Title\n"), ["Title"]);
        assert!(read("").expect("a blank document imports").is_empty());
        assert!(read("\n\n").expect("a blank document imports").is_empty());
    }

    #[test]
    fn a_carriage_return_is_refused_at_the_import_door_too() {
        assert_eq!(
            read("# Title\r\n\n## One\r\n"),
            Err(Refusal::CarriageReturnInDocument)
        );
    }

    #[test]
    fn an_open_questions_section_is_recognised_by_any_of_its_corpus_spellings() {
        for title in [
            "6. Open Questions & Unknowns",
            "17. Open Questions",
            "Remaining open questions",
        ] {
            assert!(is_open_questions(title), "{title}");
        }
        for title in ["2. Proposed Design", "Questions answered", "Open issues"] {
            assert!(!is_open_questions(title), "{title}");
        }
    }

    #[test]
    fn an_entry_is_a_list_item_and_its_question_is_its_headline() {
        // Both conventions the corpus actually uses. The first is the fixture's:
        // label bolded, question outside the emphasis. The second is this
        // slice's own design: label AND title bolded, body prose after the
        // closing `**` — the headline stops at the emphasis rather than welding
        // the title to the first line of the body.
        let region = Region {
            line: 10,
            body: "## 6. Open Questions\n\
                   \n\
                   - **OQ-1:** whether the reader tracks fences.\n\
                   - **OQ-2 / RSK-229 — managed-instruction authority.** Boot\n\
                     establishes strong routing obligations but not the general\n\
                   - **Multi-line DOT labels**: no OQ label, so not an entry.\n\
                   Body prose mentions OQ-9 here, mid-paragraph.\n",
        };
        assert_eq!(
            open_questions(&region),
            vec![
                OpenQuestion {
                    line: 12,
                    label: "OQ-1",
                    question: "whether the reader tracks fences.",
                    citation: None,
                },
                // The label stops at the id's boundary, so a co-label
                // (`/ RSK-229`) stays in the headline's span and out of it.
                OpenQuestion {
                    line: 13,
                    label: "OQ-2",
                    question: "managed-instruction authority.",
                    citation: None,
                },
            ]
        );
    }

    #[test]
    fn only_an_explicit_canonical_citation_is_read_as_one() {
        assert_eq!(
            citations(
                "## Open Questions\n\
                 - **OQ-1 / QUE-177 — bootstrapping.** cites a record.\n\
                 - **OQ-2:** mentions QUE- with no number, and QUExyz.\n"
            ),
            vec![Some("QUE-177"), None],
            "a citation is the prefix AND an id; a bare prefix is prose"
        );
    }

    #[test]
    fn a_numeric_prefix_followed_by_identifier_characters_is_not_a_citation() {
        assert_eq!(
            citations(
                "## Open Questions\n\
                 - **OQ-1:** cites QUE-177abc, a longer token entirely.\n"
            ),
            vec![None],
            "an id ends at a boundary; a token merely opening with one is not a citation"
        );
    }

    #[test]
    fn a_prefix_glued_to_preceding_text_is_not_a_citation() {
        assert_eq!(
            citations(
                "## Open Questions\n\
                 - **OQ-1:** the token XQUE-177 is not a citation.\n"
            ),
            vec![None],
            "a canonical ref is a WHOLE token — `integrity::line_cites` holds the \
             same line for `SL-031` inside `ASL-031`"
        );
    }

    #[test]
    fn a_bare_prefix_does_not_mask_a_later_canonical_citation() {
        assert_eq!(
            citations(
                "## Open Questions\n\
                 - **OQ-1:** compare QUE- with QUE-177.\n"
            ),
            vec![Some("QUE-177")],
            "the scan continues past a prefix carrying no id — a missed citation \
             seeds a duplicate where DEC-085 requires a merge"
        );
    }
}
