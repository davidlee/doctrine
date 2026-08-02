// SPDX-License-Identifier: GPL-3.0-only
//! The authored document's grammar: markers, framing, escaping, and the seven
//! refusals a hand edit can produce (SL-233 PHASE-13 EX-3, PHASE-14 EX-3).
//!
//! # Why this is a module and not two functions in `commands/design.rs`
//!
//! ADR-001 layers leaf ← engine ← command, and this is leaf-most. [`render`],
//! [`parse`], the marker recogniser, the shape test and the escape/unescape pair
//! are not five things that happen to sit near each other — they are ONE total,
//! mutually-inverse pair, and `parse(render(S)) == S` is a property only the
//! pair can have. Splitting them across layers is precisely what lets the two
//! halves drift, which is the defect class this phase exists to close: the
//! incumbent emitted `section.body` raw from the shell and trimmed it back in a
//! different function twenty lines away, and nothing could see the disagreement.
//!
//! Nothing here reads a clock, a disk, a digest or a snapshot — it is a function
//! from ids and bodies to bytes, and back. Its out-degree is [`super::ids`] and
//! [`super::refusal`], the two siblings every admission path already names, and
//! it is a SIBLING of `render`, so the privacy precondition in this tree's
//! module doc is untouched. [`super::section`]'s home was argued the same way.
//!
//! # The rule
//!
//! `.doctrine/slice/233/sketches/marker-grammar.md` answers (a), (c) and
//! § *Row 10's resolution* are authoritative; this is their transcription.
//!
//! **Framing is uniform.** Each block is the marker line, a newline, the escaped
//! body verbatim, then ONE newline, and the blocks are concatenated with **no
//! separator**. The incumbent built the same block and then called
//! `blocks.join("\n")`, so an interior block carried two framing newlines and the
//! last carried one — a position dependence that put the end-of-document case
//! beyond any single parse rule. Remove-exactly-one inverts append-exactly-one
//! only when every block carries the same one.
//!
//! **Escaping is lexical.** A marker-shaped body line has its colon run
//! incremented on write and decremented on read. It does **not** consult
//! Markdown block structure: a parser that skipped fenced regions would have to
//! track fence state across a document a human may have left unbalanced, and an
//! unbalanced fence would then silently swallow every subsequent marker. Lexical
//! beats structural precisely because the input is untrusted.

use std::collections::BTreeSet;

use super::ids::{DesignId, IdKind};
use super::refusal::Refusal;

// ── The grammar's bytes (STD-001) ─────────────────────────────────────────

/// The marker's opening delimiter, exactly as [`render`] writes it. Adopted
/// unchanged from the incumbent: it is already written into every document
/// Doctrine has materialised, and re-spelling it would be a migration bought for
/// nothing.
pub(crate) const MARKER_OPEN: &str = "<!-- doctrine:section ";
/// The marker's closing delimiter.
pub(crate) const MARKER_CLOSE: &str = " -->";
/// The colon inside [`MARKER_OPEN`] whose repetition count carries the escaping.
/// Named because both the shape test and the transformation address it, and
/// [`marker_open_parts`] derives the split rather than re-spelling either side.
const MARKER_COLON: char = ':';
/// The most leading spaces the SHAPE test tolerates. `CommonMark`'s limit before
/// a line becomes an indented code block, so it is the widest promotion a
/// conforming formatter could perform. **Defensive**, and labelled so: prettier
/// was measured preserving 1-, 2- and 3-space indentation on a top-level HTML
/// comment, so it cannot promote an indented lookalike today.
const MAX_SHAPE_INDENT: usize = 3;

/// [`MARKER_OPEN`] split at [`MARKER_COLON`]: the bytes before it, and the bytes
/// after it. Derived from the one literal rather than written twice, so the
/// shape test cannot disagree with what [`render`] emits (STD-001).
fn marker_open_parts() -> (&'static str, &'static str) {
    MARKER_OPEN
        .split_once(MARKER_COLON)
        .unwrap_or((MARKER_OPEN, ""))
}

// ── Recognition ───────────────────────────────────────────────────────────

/// A line matching the marker SHAPE, and where its colon run sits **in the
/// original line**.
struct Shaped {
    /// How many colons stand between `doctrine` and `section`. At least one.
    colons: usize,
    /// The byte offset just past the colon run, in the ORIGINAL line — so a
    /// transformation splices one colon and leaves every other byte alone.
    colons_end: usize,
    /// The section id the line names.
    id: DesignId,
    /// Whether this is the EXACT form [`render`] writes: column 0, one colon,
    /// and the line ending at `-->`.
    exact: bool,
}

/// Is `line` **marker-shaped** (answer (c))?
///
/// After right-trimming whitespace and removing up to [`MAX_SHAPE_INDENT`]
/// leading spaces, the line must match
/// `"<!-- doctrine:" ":"* "section " <section-id> " -->"`, with the id legal
/// under answer (a). Let *k* ≥ 1 be the colon count.
///
/// **The right trim is required, not defensive.** Measured: prettier strips
/// trailing whitespace, so a body line `<!-- doctrine:section sec-6 -->` followed
/// by three spaces — not a marker as written, because recognition demands the
/// line end at `-->` — is *normalised into a marker* by a formatter run. Without
/// the right trim here that line escapes escaping, and formatting promotes body
/// text into a section boundary. It is the sharpest edge in the grammar and
/// nothing about the syntax suggests it.
///
/// **The id grammar is part of the shape test in both directions.**
/// `<!-- doctrine:section not a valid id -->` is not shaped, so it is neither
/// escaped on write nor recognised on read — consistent, and the round trip
/// holds. If the two directions disagreed about what counts as shaped, escaping
/// would stop being invertible.
///
/// Both trims **classify only**. Every offset returned is into the original
/// line, and nothing this function removes is ever removed from a byte that is
/// stored or emitted.
fn shape(line: &str) -> Option<Shaped> {
    let trimmed = line.trim_end();
    let undented = trimmed.trim_start_matches(' ');
    let indent = trimmed.len() - undented.len();
    if indent > MAX_SHAPE_INDENT {
        return None;
    }
    let (head, tail) = marker_open_parts();
    let after_head = undented.strip_prefix(head)?;
    let after_colons = after_head.trim_start_matches(MARKER_COLON);
    let colons = after_head.len() - after_colons.len();
    if colons == 0 {
        return None;
    }
    let named = after_colons
        .strip_prefix(tail)?
        .strip_suffix(MARKER_CLOSE)?;
    let id = DesignId::parse(named).ok()?;
    if id.kind() != IdKind::Section {
        return None;
    }
    Some(Shaped {
        colons,
        colons_end: indent + head.len() + colons,
        id,
        exact: indent == 0 && colons == 1 && trimmed.len() == line.len(),
    })
}

/// The section a line **marks**, or `None` when the line is body text.
///
/// Recognition is EXACT: column 0, [`MARKER_OPEN`] then a legal section id then
/// [`MARKER_CLOSE`], nothing else on the line. Because those two constants carry
/// the separators, that is exactly one space at each of the three separator
/// positions — not "one or more". A normalising formatter that collapsed runs of
/// spaces would otherwise be able to turn a two-space lookalike into a marker,
/// which is the same promotion hazard [`shape`]'s right trim exists to close.
///
/// **This tightens the incumbent, deliberately.** `authored_sections` used to
/// `.trim()` whatever sat between the delimiters, so
/// `<!-- doctrine:section  sec-1 -->` was accepted as `sec-1`. Two spellings of
/// one marker is what "recognition is exact" denies, and with answer (a)'s
/// charset excluding whitespace from ids the trim protected nothing.
pub(crate) fn marker(line: &str) -> Option<DesignId> {
    shape(line)
        .filter(|shaped| shaped.exact)
        .map(|shaped| shaped.id)
}

// ── Escaping (answer (c)) ─────────────────────────────────────────────────

/// Write side: a marker-shaped line with *k* colons is emitted with *k+1*.
fn escape_line(line: &str) -> Option<String> {
    let shaped = shape(line)?;
    let head = line.get(..shaped.colons_end)?;
    let tail = line.get(shaped.colons_end..)?;
    Some(format!("{head}{MARKER_COLON}{tail}"))
}

/// Read side: a marker-shaped line with *k* ≥ 2 colons is body, restored to
/// *k−1*. A shaped line with *k* = 1 is left alone — at column 0 [`parse`] has
/// already consumed it as a marker, and anywhere else it is body no escaping
/// ever produced.
///
/// Write maps *k ↦ k+1* on {*k* ≥ 1} and read maps *k ↦ k−1* on {*k* ≥ 2}, so
/// the two are mutually inverse on shaped lines and the identity on every other
/// line: the composition is the identity on all bodies.
fn unescape_line(line: &str) -> Option<String> {
    let shaped = shape(line)?;
    if shaped.colons < 2 {
        return None;
    }
    let head = line.get(..shaped.colons_end.saturating_sub(1))?;
    let tail = line.get(shaped.colons_end..)?;
    Some(format!("{head}{tail}"))
}

/// Apply a per-line transformation, preserving every line terminator and every
/// byte the transformation declines to claim.
fn per_line(text: &str, transform: impl Fn(&str) -> Option<String>) -> String {
    let mut out = String::with_capacity(text.len());
    for chunk in text.split_inclusive('\n') {
        match chunk.strip_suffix('\n') {
            Some(line) => {
                out.push_str(transform(line).as_deref().unwrap_or(line));
                out.push('\n');
            }
            None => out.push_str(transform(chunk).as_deref().unwrap_or(chunk)),
        }
    }
    out
}

// ── The pair ──────────────────────────────────────────────────────────────

/// One section, as the authored document carries it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DocumentSection {
    pub(crate) id: DesignId,
    pub(crate) body: String,
}

/// The authored document for `sections`, **in the order given** — which the
/// caller supplies as document order, never as id order.
///
/// For each section: the marker line, a newline, the escaped body verbatim, then
/// ONE newline. Blocks are concatenated with no separator.
pub(crate) fn render<'a>(sections: impl IntoIterator<Item = (&'a DesignId, &'a str)>) -> String {
    let mut document = String::new();
    for (id, body) in sections {
        document.push_str(MARKER_OPEN);
        document.push_str(id.as_str());
        document.push_str(MARKER_CLOSE);
        document.push('\n');
        document.push_str(&per_line(body, escape_line));
        document.push('\n');
    }
    document
}

/// Decompose an authored document into its marker-addressed sections — the exact
/// inverse of [`render`] — refusing every departure of the document from the
/// run by its own name.
///
/// The **marker lines** cut the document into a head (everything before the
/// first, possibly empty) and one **region** per marker: the bytes from just
/// after that marker line's terminating newline up to the first byte of the next
/// marker line, or to the end of the document for the last. That decomposition
/// is defined for every byte string — including one with no marker lines at all
/// (head = the whole document, zero regions) — and every byte lands in exactly
/// one part, so classifying the head and each region totally is sufficient.
///
/// | region | classification |
/// |---|---|
/// | ends with `"\n"` | body = region minus **exactly one** trailing `"\n"` |
/// | empty | [`Refusal::StructuralDeletion`] — marker present, region not |
/// | non-empty, no trailing `"\n"` | [`Refusal::UnterminatedDocument`] |
///
/// The third row is reachable **only at the end of the document**, and that is
/// what makes the rule well-defined rather than merely enumerated: a marker line
/// begins at column 0, so either it starts at offset 0 or the byte immediately
/// before it is a newline — the last byte of the preceding region. Hence every
/// region except the last either is empty or ends in a newline.
///
/// No other byte is removed. There is no trimming anywhere on this path.
///
/// # The seven checks, in one fixed order (marker-grammar sketch §(e))
///
/// The refusal set is a **partition**: total because the decomposition above is,
/// disjoint because this order makes at most one fire. The order is the whole of
/// the disjointness argument, so it lives in ONE function — bolting the run-aware
/// rows on after a run-blind parse would silently report `StructuralDeletion`
/// where the sketch mandates `UnknownMarker`.
///
/// 1. [`Refusal::CarriageReturnInDocument`] — any `\r` byte, whole document.
/// 2. [`Refusal::MarkerFreeAddition`] — non-blank bytes in the head.
/// 3. [`Refusal::DuplicateMarker`] — one id marking two regions.
/// 4. [`Refusal::UnknownMarker`] — a marker the run does not hold.
/// 5. [`Refusal::MissingMarker`] — a held section with no marker.
/// 6. [`Refusal::StructuralDeletion`] and 7. [`Refusal::UnterminatedDocument`],
///    per region in document order — and 6 before 7 by construction, since row 7
///    is reachable only for the final region.
///
/// `held` is `None` where there is no run to compare against, which skips rows 4
/// and 5 — they are **undecidable** without one, and skipping is not the same as
/// passing them.
pub(crate) fn parse(
    text: &str,
    held: Option<&BTreeSet<DesignId>>,
) -> Result<Vec<DocumentSection>, Refusal> {
    // 1 — a whole-document byte test. `str::lines` silently drops `\r`, so a
    // CRLF save would otherwise be adopted as LF and the next materialise would
    // rewrite the user's line endings without saying so.
    if text.contains('\r') {
        return Err(Refusal::CarriageReturnInDocument);
    }
    let markers = marker_lines(text);

    // 2 — the head. Whitespace-only is NOT an addition: a leading blank line is
    // what a formatter produces, and refusing it would make the document
    // unformattable. Non-blank bytes are the trigger.
    let head_end = markers.first().map_or(text.len(), |first| first.0);
    if !text.get(..head_end).unwrap_or_default().trim().is_empty() {
        return Err(Refusal::MarkerFreeAddition);
    }

    // 3 — duplicates, reported in document order.
    let mut marked: BTreeSet<DesignId> = BTreeSet::new();
    for (_, _, id) in &markers {
        if !marked.insert(id.clone()) {
            return Err(Refusal::DuplicateMarker { id: id.clone() });
        }
    }

    // 4 then 5 — the document's marker set against the run's held sections, in
    // both directions. This is a DIFFERENT comparison from the adoption
    // completeness check, which reads the CALLER's declared map: a legal but
    // unheld marker is invisible to that one.
    if let Some(held) = held {
        if let Some(id) = markers
            .iter()
            .map(|(_, _, id)| id)
            .find(|id| !held.contains(id))
        {
            return Err(Refusal::UnknownMarker { id: id.clone() });
        }
        if let Some(id) = held.iter().find(|id| !marked.contains(id)) {
            return Err(Refusal::MissingMarker { id: id.clone() });
        }
    }

    // 6 and 7 — the two non-body rows of the region table.
    let mut sections = Vec::with_capacity(markers.len());
    for (index, (_, region_at, id)) in markers.iter().enumerate() {
        let end = markers.get(index + 1).map_or(text.len(), |next| next.0);
        let region = text.get(*region_at..end).unwrap_or_default();
        let Some(body) = region.strip_suffix('\n') else {
            return Err(if region.is_empty() {
                Refusal::StructuralDeletion { id: id.clone() }
            } else {
                Refusal::UnterminatedDocument
            });
        };
        sections.push(DocumentSection {
            id: id.clone(),
            body: per_line(body, unescape_line),
        });
    }
    Ok(sections)
}

/// Every marker line: where it starts, where its region starts, and what it
/// names.
fn marker_lines(text: &str) -> Vec<(usize, usize, DesignId)> {
    let mut found = Vec::new();
    let mut at = 0_usize;
    for chunk in text.split_inclusive('\n') {
        let line = chunk.strip_suffix('\n').unwrap_or(chunk);
        if let Some(id) = marker(line) {
            found.push((at, at.saturating_add(chunk.len()), id));
        }
        at = at.saturating_add(chunk.len());
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section(raw: &str) -> DesignId {
        DesignId::parse(raw).expect("a well-formed section id")
    }

    /// The run-blind reading — rows 4 and 5 are undecidable without a run.
    fn read(text: &str) -> Result<Vec<DocumentSection>, Refusal> {
        parse(text, None)
    }

    /// The set of sections a run holds, for the run-aware rows.
    fn holding(ids: &[&str]) -> BTreeSet<DesignId> {
        ids.iter().map(|raw| section(raw)).collect()
    }

    /// A marker line and its terminating newline.
    fn block(id: &str, body: &str) -> String {
        format!("{MARKER_OPEN}{id}{MARKER_CLOSE}\n{body}")
    }

    // --- recognition is exact ---

    #[test]
    fn a_marker_is_the_exact_line_render_writes_and_nothing_else() {
        assert_eq!(
            marker("<!-- doctrine:section sec-1 -->"),
            Some(section("sec-1"))
        );
        // Slack spacing is a second spelling of one marker, so it is not one.
        assert_eq!(marker("<!-- doctrine:section  sec-1 -->"), None);
        assert_eq!(marker("<!-- doctrine:section sec-1  -->"), None);
        // Not at column 0.
        assert_eq!(marker(" <!-- doctrine:section sec-1 -->"), None);
        // Trailing whitespace: the line does not end at `-->`.
        assert_eq!(marker("<!-- doctrine:section sec-1 --> "), None);
        // An escaped lookalike is body, never a boundary.
        assert_eq!(marker("<!-- doctrine::section sec-1 -->"), None);
        // Ids illegal under answer (a), and ids of another kind.
        assert_eq!(marker("<!-- doctrine:section not a valid id -->"), None);
        assert_eq!(marker("<!-- doctrine:section sec-a.b -->"), None);
        assert_eq!(marker("<!-- doctrine:section inq-1 -->"), None);
    }

    // --- the shape test's two trims ---

    #[test]
    fn the_shape_test_right_trims_and_left_trims_to_three_spaces() {
        // Right trim: a formatter would strip those spaces and promote the line,
        // so it is escaped while it still cannot be promoted.
        assert_eq!(
            escape_line("<!-- doctrine:section sec-1 -->   ").as_deref(),
            Some("<!-- doctrine::section sec-1 -->   ")
        );
        // Left trim, up to three spaces — and not four.
        for indent in ["", " ", "  ", "   "] {
            assert!(
                escape_line(&format!("{indent}<!-- doctrine:section sec-1 -->")).is_some(),
                "{indent:?} is within CommonMark's indent limit"
            );
        }
        assert_eq!(
            escape_line("    <!-- doctrine:section sec-1 -->"),
            None,
            "four spaces is an indented code block, not a promotable comment"
        );
    }

    #[test]
    fn escaping_and_unescaping_are_mutually_inverse_on_shaped_lines() {
        for line in [
            "<!-- doctrine:section sec-1 -->",
            "   <!-- doctrine::section sec-1 -->  ",
            "<!-- doctrine:::::section sec-99 -->",
        ] {
            let escaped = escape_line(line).expect("shaped");
            assert_eq!(unescape_line(&escaped).as_deref(), Some(line));
        }
        // Identity off the shaped set, in both directions.
        for line in ["", "ordinary prose", "    <!-- doctrine:section sec-1 -->"] {
            assert_eq!(escape_line(line), None);
            assert_eq!(unescape_line(line), None);
        }
    }

    // --- the region table's three rows ---

    #[test]
    fn every_region_is_classified_by_the_three_row_table() {
        let id = section("sec-1");
        let marker_line = format!("{MARKER_OPEN}{}{MARKER_CLOSE}\n", id.as_str());

        // Row 1 — ends with a newline: exactly one is removed, nothing else.
        assert_eq!(
            read(&format!("{marker_line}prose  \n\n")),
            Ok(vec![DocumentSection {
                id: id.clone(),
                body: "prose  \n".to_owned(),
            }])
        );
        // Row 2 — empty region, at the end of the document and in the middle.
        assert_eq!(
            read(&marker_line),
            Err(Refusal::StructuralDeletion { id: id.clone() })
        );
        assert_eq!(
            read(&format!(
                "{marker_line}{MARKER_OPEN}sec-2{MARKER_CLOSE}\nx\n"
            )),
            Err(Refusal::StructuralDeletion { id })
        );
        // Row 3 — non-empty and unterminated, reachable only at end of document.
        assert_eq!(
            read(&format!("{marker_line}prose")),
            Err(Refusal::UnterminatedDocument)
        );
    }

    #[test]
    fn the_marker_open_split_is_derived_from_the_one_literal() {
        let (head, tail) = marker_open_parts();
        assert_eq!(format!("{head}{MARKER_COLON}{tail}"), MARKER_OPEN);
    }

    #[test]
    fn render_frames_every_block_identically_and_parse_inverts_it() {
        let one = section("sec-1");
        let two = section("sec-2");
        let bodies = [(&one, "interior\n\n"), (&two, "last  ")];
        let document = render(bodies);
        assert_eq!(
            document,
            format!(
                "{MARKER_OPEN}sec-1{MARKER_CLOSE}\ninterior\n\n\n\
                 {MARKER_OPEN}sec-2{MARKER_CLOSE}\nlast  \n"
            ),
            "the affix is uniform and the blocks carry no separator"
        );
        assert_eq!(
            read(&document),
            Ok(vec![
                DocumentSection {
                    id: one,
                    body: "interior\n\n".to_owned()
                },
                DocumentSection {
                    id: two,
                    body: "last  ".to_owned()
                },
            ])
        );
    }

    // --- §(e): each of the seven, by name ---

    #[test]
    fn every_departure_from_the_run_has_its_own_refusal() {
        let held = holding(&["sec-1", "sec-2"]);
        let one = block("sec-1", "one\n");
        let two = block("sec-2", "two\n");
        let whole = format!("{one}{two}");

        // The document Doctrine wrote is adopted, and a formatter's leading
        // blank line does not make it an addition.
        assert!(parse(&whole, Some(&held)).is_ok());
        assert!(parse(&format!("\n  \n{whole}"), Some(&held)).is_ok());

        // 1 — a CRLF save.
        assert_eq!(
            parse(&whole.replace('\n', "\r\n"), Some(&held)),
            Err(Refusal::CarriageReturnInDocument)
        );
        // 2 — a preamble nobody declared.
        assert_eq!(
            parse(&format!("preamble\n{whole}"), Some(&held)),
            Err(Refusal::MarkerFreeAddition)
        );
        // 3 — one id marking two regions.
        assert_eq!(
            parse(&format!("{whole}{one}"), Some(&held)),
            Err(Refusal::DuplicateMarker {
                id: section("sec-1")
            })
        );
        // 4 — a legal marker the run does not hold.
        assert_eq!(
            parse(&format!("{whole}{}", block("sec-9", "nine\n")), Some(&held)),
            Err(Refusal::UnknownMarker {
                id: section("sec-9")
            })
        );
        // 5 — a held section whose marker line was deleted.
        assert_eq!(
            parse(&one, Some(&held)),
            Err(Refusal::MissingMarker {
                id: section("sec-2")
            })
        );
        // 6 and 7 are the region table's two non-body rows, already asserted by
        // `every_region_is_classified_by_the_three_row_table`.
    }

    /// Rows 4 and 5 are **undecidable** without a run, so they are skipped when
    /// no held set is supplied — skipped, not passed. Every other row still
    /// fires, which is what makes the run-blind reading safe to use at the
    /// digest seam.
    #[test]
    fn without_a_held_set_only_the_run_aware_rows_are_skipped() {
        let unheld = block("sec-9", "nine\n");
        assert!(read(&unheld).is_ok(), "row 4 needs a run to decide");
        assert_eq!(
            read(&format!("{unheld}{unheld}")),
            Err(Refusal::DuplicateMarker {
                id: section("sec-9")
            }),
            "but row 3 does not"
        );
    }

    /// The order IS the disjointness argument, so it is asserted directly: each
    /// document below violates TWO rows at once and must report the
    /// LOWER-NUMBERED one. A single-fault fixture cannot see this, and a
    /// bolted-on reader would report 6 where the sketch mandates 4.
    #[test]
    fn a_two_fault_document_reports_the_lower_numbered_row() {
        let held = holding(&["sec-1", "sec-2"]);
        let one = block("sec-1", "one\n");
        let two = block("sec-2", "two\n");

        // 1 before 2 — a CRLF document that also carries a preamble.
        assert_eq!(
            parse(
                &format!("preamble\n{one}{two}").replace('\n', "\r\n"),
                Some(&held)
            ),
            Err(Refusal::CarriageReturnInDocument)
        );
        // 2 before 3 — a preamble above a duplicated marker.
        assert_eq!(
            parse(&format!("preamble\n{one}{two}{one}"), Some(&held)),
            Err(Refusal::MarkerFreeAddition)
        );
        // 3 before 4 — a duplicated marker beside an unheld one.
        assert_eq!(
            parse(
                &format!("{one}{two}{one}{}", block("sec-9", "nine\n")),
                Some(&held)
            ),
            Err(Refusal::DuplicateMarker {
                id: section("sec-1")
            })
        );
        // 4 before 5 — EDITING a marker's id produces both at once, and the
        // order resolves it to the token actually in front of the user.
        assert_eq!(
            parse(&format!("{one}{}", block("sec-9", "two\n")), Some(&held)),
            Err(Refusal::UnknownMarker {
                id: section("sec-9")
            })
        );
        // 5 before 6 — a deleted marker line above an emptied region.
        assert_eq!(
            parse(&format!("{}", block("sec-1", "")), Some(&held)),
            Err(Refusal::MissingMarker {
                id: section("sec-2")
            })
        );
        // 6 before 7 — an emptied interior region in an unterminated document.
        assert_eq!(
            parse(
                &format!("{}{}", block("sec-1", ""), block("sec-2", "two")),
                Some(&held)
            ),
            Err(Refusal::StructuralDeletion {
                id: section("sec-1")
            })
        );
    }
}
