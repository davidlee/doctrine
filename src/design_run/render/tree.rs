// SPDX-License-Identifier: GPL-3.0-only
//! The whole-map tree — a rendering of the full turn envelope (`SL-266` sec-3;
//! `DEC-306`, `DEC-307`, `DEC-308`).

use std::collections::BTreeMap;

use owo_colors::{OwoColorize, Style};
use textwrap::core::display_width;
use textwrap::{Options, WordSplitter};

use super::super::inquiry::{InquiryLifecycle, Provenance};
use super::envelope::{MapAnswer, MapNode, TitleLookup, TurnEnvelope};

/// How the shell wants the tree laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TreeStyle {
    /// Terminal width, or `None` when stdout is not a terminal; the renderer
    /// applies its own [`TREE_PIPED_WIDTH`] fallback.
    pub(crate) width: Option<u16>,
    pub(crate) colour: bool,
}

// ── layout ────────────────────────────────────────────────────────────────

/// The width a piped tree lays out to.
const TREE_PIPED_WIDTH: usize = 100;
/// Below this many free columns beside its prefix, node text drops a line.
const TREE_MIN_TEXT_COLS: usize = 24;
/// Dropped node text keeps its rails while this many columns remain beside
/// them; below it, the text falls back to [`TREE_DROP_INDENT`].
const TREE_MIN_DROP_COLS: usize = 16;
/// The fixed indent dropped node text falls back to, rails not drawn — the
/// deep-and-narrow case, where the rails alone would fill the line.
const TREE_DROP_INDENT: usize = 8;
/// Columns between a node's id and its text.
const TREE_ID_GAP: usize = 2;

// ── glyphs ────────────────────────────────────────────────────────────────

const TREE_BRANCH: &str = "├── ";
const TREE_LAST_BRANCH: &str = "└── ";
const TREE_RAIL: &str = "│   ";
const TREE_NO_RAIL: &str = "    ";
/// The rail a node with children draws under its own mark.
const TREE_CHILD_RAIL: &str = "│";

const TREE_MARK_RESOLVED: &str = "●";
const TREE_MARK_OPEN: &str = "○";
const TREE_MARK_BLOCKED: &str = "◌";
const TREE_MARK_DEFERRED: &str = "◐";
const TREE_MARK_PRUNED: &str = "⊘";

const TREE_LETTER_USER: &str = "u";
const TREE_LETTER_AGENT: &str = "a";
const TREE_LETTER_SHAPING: &str = "s";
const TREE_LETTER_IMPORTED: &str = "i";

/// Marks whether a node is blocking.
const TREE_BLOCKING_MARK: &str = "*";
const TREE_NOT_BLOCKING: &str = " ";

// ── words ─────────────────────────────────────────────────────────────────

const TREE_SUFFIX_ARROW: &str = "←";
const TREE_SUFFIX_CURSOR: &str = "cursor";
const TREE_SUFFIX_STALE_CURSOR: &str = "cursor (moved up: declared cursor is settled)";
const TREE_SUFFIX_PINNED: &str = "pinned";
const TREE_NEEDS: &str = "needs";
const TREE_PARENT: &str = "parent";
const TREE_RECORD_NOT_FOUND: &str = "record not found";
const TREE_RECORD_UNREADABLE: &str = "unreadable";
const TREE_UNPLACED_HEADING: &str = "unplaced (parent missing or cyclic):";
const TREE_HEADER_SEPARATOR: &str = " · ";
const TREE_LEGEND_SEPARATOR: &str = "·";
/// The command that reproduces this view, less its slice.
const TREE_COMMAND: &str = "doctrine design tree";

// ── vocabulary ────────────────────────────────────────────────────────────

/// A node's rendered state: its lifecycle, with `open` split by whether its
/// `needs` are settled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Resolved,
    Open,
    Blocked,
    Deferred,
    Pruned,
}

impl State {
    /// Header and legend order.
    const ALL: [State; 5] = [
        State::Resolved,
        State::Open,
        State::Blocked,
        State::Deferred,
        State::Pruned,
    ];

    /// `None` for a lifecycle token outside the closed vocabulary.
    fn of(node: &MapNode) -> Option<State> {
        let lifecycle = [
            (InquiryLifecycle::Resolved, State::Resolved),
            (InquiryLifecycle::Open, State::Open),
            (InquiryLifecycle::Deferred, State::Deferred),
            (InquiryLifecycle::Pruned, State::Pruned),
        ]
        .into_iter()
        .find(|(lifecycle, _)| lifecycle.as_str() == node.lifecycle)
        .map(|(_, state)| state)?;
        Some(match lifecycle {
            State::Open if !node.blocked_by.is_empty() => State::Blocked,
            state => state,
        })
    }

    const fn mark(self) -> &'static str {
        match self {
            State::Resolved => TREE_MARK_RESOLVED,
            State::Open => TREE_MARK_OPEN,
            State::Blocked => TREE_MARK_BLOCKED,
            State::Deferred => TREE_MARK_DEFERRED,
            State::Pruned => TREE_MARK_PRUNED,
        }
    }

    /// The word the header counts and the legend explains.
    const fn word(self) -> &'static str {
        match self {
            State::Resolved => "resolved",
            State::Open => "open",
            State::Blocked => "blocked",
            State::Deferred => "deferred",
            State::Pruned => "pruned",
        }
    }

    const fn style(self) -> Style {
        match self {
            State::Resolved => Style::new().green(),
            State::Open => Style::new().cyan(),
            State::Blocked | State::Deferred => Style::new().yellow(),
            State::Pruned => Style::new().red(),
        }
    }
}

/// The cursor suffix: the open mark's colour, bold.
const TREE_CURSOR_STYLE: Style = State::Open.style().bold();
/// The pinned suffix.
const TREE_PINNED_STYLE: Style = Style::new().magenta();

/// Provenance label → (letter, legend word), in legend order.
const TREE_PROVENANCES: [(&str, &str, &str); 4] = [
    (Provenance::USER_DIRECTED, TREE_LETTER_USER, "user"),
    (Provenance::AGENT_PROPOSED, TREE_LETTER_AGENT, "agent"),
    (Provenance::SHAPING_QUESTION, TREE_LETTER_SHAPING, "shaping"),
    (Provenance::IMPORTED_PROSE, TREE_LETTER_IMPORTED, "imported"),
];

/// A node's provenance letter; a label outside the vocabulary renders whole,
/// disclosed rather than dropped (`STD-003`).
fn letter(provenance: &str) -> &str {
    TREE_PROVENANCES
        .iter()
        .find(|(label, _, _)| *label == provenance)
        .map_or(provenance, |(_, letter, _)| letter)
}

// ── the rendering ─────────────────────────────────────────────────────────

/// Render `envelope`'s whole map as a tree: header, placed nodes, the
/// `unplaced` section, legend, footer.
///
/// Reads only the envelope and the style (`SPEC-029` `REQ-433`). No text is
/// truncated; a line exceeds the width only when it holds a single prefix or
/// word wider than the space it has (`DEC-307`).
pub(crate) fn render(envelope: &TurnEnvelope, style: TreeStyle) -> Vec<String> {
    let width = style.width.map_or(TREE_PIPED_WIDTH, usize::from);
    let layout = Layout {
        envelope,
        width,
        colour: style.colour,
        id_column: envelope
            .map
            .iter()
            .map(|node| display_width(&node.id))
            .max()
            .unwrap_or(0),
    };
    let (placed, unplaced) = place(&envelope.map);

    let mut lines = header(envelope, width);
    for row in &placed {
        lines.extend(layout.node_lines(row));
    }
    if !unplaced.is_empty() {
        lines.push(TREE_UNPLACED_HEADING.to_owned());
        for node in unplaced {
            lines.extend(layout.node_lines(&Row::unplaced(node)));
        }
    }
    lines.extend(legend(width, style.colour));
    lines.push(footer(envelope));
    lines
}

/// The canonical slice reference this view names.
fn slice_ref(envelope: &TurnEnvelope) -> String {
    format!("SL-{:03}", envelope.run.slice)
}

/// The command that reproduces this view — the footer, so a verbatim relay
/// teaches the user the command (`DEC-309`).
fn footer(envelope: &TurnEnvelope) -> String {
    format!("{TREE_COMMAND} {}", slice_ref(envelope))
}

/// The header counts and, when the shell chose the run, how (`DEC-305`).
/// Counts only — no word that certifies completeness (`PRD-019` `REQ-425`).
fn header(envelope: &TurnEnvelope, width: usize) -> Vec<String> {
    let count = |state: State| {
        envelope
            .map
            .iter()
            .filter(|node| State::of(node) == Some(state))
            .count()
    };
    let nodes = envelope.map.len();
    let counts: Vec<String> = State::ALL
        .into_iter()
        .map(|state| (state, count(state)))
        .filter(|(state, n)| matches!(state, State::Resolved | State::Open) || *n > 0)
        .map(|(state, n)| format!("{n} {}", state.word()))
        .collect();
    let noun = if nodes == 1 { "question" } else { "questions" };
    let mut free = vec![
        [
            slice_ref(envelope),
            envelope.run.stage.to_owned(),
            format!("rev {}", envelope.run.revision),
            format!("{nodes} {noun}: {}", counts.join(", ")),
        ]
        .join(TREE_HEADER_SEPARATOR),
    ];
    if let Some(selection) = &envelope.selection {
        free.push(match selection.candidates {
            1 => "chosen: the only run open when scanned".to_owned(),
            n => format!("chosen: newest of {n} runs open when scanned"),
        });
        free.extend(
            selection
                .skipped
                .iter()
                .map(|skipped| format!("skipped {}: {}", skipped.path, skipped.reason)),
        );
    }
    free.iter().flat_map(|line| wrap(line, width)).collect()
}

/// The legend, built from the same constants the nodes render with, broken
/// only between entries.
fn legend(width: usize, colour: bool) -> Vec<String> {
    let letter_style = Style::new().dimmed();
    // (plain, painted): packed by the plain width, emitted painted.
    let entries = State::ALL
        .into_iter()
        .map(|state| {
            let word = state.word();
            (
                format!("{} {word}", state.mark()),
                format!("{} {word}", paint(colour, state.mark(), state.style())),
            )
        })
        .chain([(
            format!("{TREE_LEGEND_SEPARATOR} {TREE_BLOCKING_MARK} blocking"),
            format!("{TREE_LEGEND_SEPARATOR} {TREE_BLOCKING_MARK} blocking"),
        )])
        .chain(
            TREE_PROVENANCES
                .iter()
                .enumerate()
                .map(|(at, (_, letter, word))| {
                    let lead = if at == 0 {
                        format!("{TREE_LEGEND_SEPARATOR} ")
                    } else {
                        String::new()
                    };
                    (
                        format!("{lead}{letter} {word}"),
                        format!("{lead}{} {word}", paint(colour, letter, letter_style)),
                    )
                }),
        );
    let mut lines: Vec<(usize, String)> = Vec::new();
    for (plain, painted) in entries {
        let entry_width = display_width(&plain);
        match lines.last_mut() {
            Some((line_width, line)) if *line_width + 1 + entry_width <= width => {
                *line_width += 1 + entry_width;
                line.push(' ');
                line.push_str(&painted);
            }
            _ => lines.push((entry_width, painted)),
        }
    }
    lines.into_iter().map(|(_, line)| line).collect()
}

/// Fill `text` to `width` between words, never splitting one.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let options = Options::new(width.max(1))
        .break_words(false)
        .word_splitter(WordSplitter::NoHyphenation);
    textwrap::wrap(text, options)
        .into_iter()
        .map(std::borrow::Cow::into_owned)
        .collect()
}

// ── placement ─────────────────────────────────────────────────────────────

/// One node as placed: the rails its ancestors draw, and its own branch.
struct Row<'a> {
    node: &'a MapNode,
    /// The ancestors' rails, root first.
    rails: String,
    /// `Some(is_last_sibling)` for a placed node; `None` under `unplaced`.
    branch: Option<bool>,
    has_children: bool,
}

impl<'a> Row<'a> {
    /// The rails this node's own continuation lines draw: its ancestors', then
    /// its own sibling rail.
    fn own_rails(&self) -> String {
        let own = match self.branch {
            Some(true) => TREE_NO_RAIL,
            Some(false) => TREE_RAIL,
            None => "",
        };
        [self.rails.as_str(), own].concat()
    }

    fn unplaced(node: &'a MapNode) -> Self {
        Row {
            node,
            rails: String::new(),
            branch: None,
            has_children: false,
        }
    }
}

/// Walk the map from its roots in map order. Every node the walk cannot reach —
/// an absent parent, a parent cycle, or a descendant of either — is returned
/// apart, in map order, so none is dropped.
fn place(map: &[MapNode]) -> (Vec<Row<'_>>, Vec<&MapNode>) {
    let mut children: BTreeMap<Option<&str>, Vec<usize>> = BTreeMap::new();
    for (at, node) in map.iter().enumerate() {
        children.entry(node.parent.as_deref()).or_default().push(at);
    }
    let mut visited = vec![false; map.len()];
    let mut placed = Vec::new();
    // Depth-first, explicit stack: (index, rails, is_last).
    let mut stack: Vec<(usize, String, bool)> = sibling_frames(&children, None, "");
    while let Some((at, rails, last)) = stack.pop() {
        let (Some(seen), Some(node)) = (visited.get_mut(at), map.get(at)) else {
            continue;
        };
        if std::mem::replace(seen, true) {
            continue;
        }
        let below = [rails.as_str(), if last { TREE_NO_RAIL } else { TREE_RAIL }].concat();
        let kids = sibling_frames(&children, Some(node.id.as_str()), &below);
        placed.push(Row {
            node,
            rails,
            branch: Some(last),
            has_children: !kids.is_empty(),
        });
        stack.extend(kids);
    }
    let unplaced = map
        .iter()
        .zip(&visited)
        .filter(|(_, seen)| !**seen)
        .map(|(node, _)| node)
        .collect();
    (placed, unplaced)
}

/// `parent`'s children as stack frames, reversed so the first pops first.
fn sibling_frames(
    children: &BTreeMap<Option<&str>, Vec<usize>>,
    parent: Option<&str>,
    rails: &str,
) -> Vec<(usize, String, bool)> {
    let siblings = children.get(&parent).map_or(&[][..], Vec::as_slice);
    siblings
        .iter()
        .enumerate()
        .rev()
        .map(|(nth, at)| (*at, rails.to_owned(), nth + 1 == siblings.len()))
        .collect()
}

// ── one node ──────────────────────────────────────────────────────────────

/// The cursor/pin suffix and the style it leads with.
struct Suffix {
    text: String,
    lead: Style,
}

struct Layout<'a> {
    envelope: &'a TurnEnvelope,
    width: usize,
    colour: bool,
    /// The widest id's display width — every id pads to it.
    id_column: usize,
}

impl Layout<'_> {
    /// A node's lines: its prefix and wrapped right-hand text (sec-3 rules 1, 2, 4).
    fn node_lines(&self, row: &Row<'_>) -> Vec<String> {
        let node = row.node;
        let state = State::of(node);
        let branch = match row.branch {
            Some(true) => TREE_LAST_BRANCH,
            Some(false) => TREE_BRANCH,
            None => "",
        };
        let pad = " ".repeat(self.id_column - display_width(&node.id) + TREE_ID_GAP);
        let blocking = if node.blocking {
            TREE_BLOCKING_MARK
        } else {
            TREE_NOT_BLOCKING
        };
        let mark = state.map_or(node.lifecycle, State::mark);
        let letter = letter(node.provenance);
        let compose = |shown_mark: &str, shown_letter: &str| {
            format!(
                "{}{branch}{shown_mark} {shown_letter} {blocking} {}{pad}",
                row.rails, node.id
            )
        };
        // Measured plain: escapes occupy no columns.
        let prefix_width = display_width(&compose(mark, letter));
        let prefix = compose(
            &self.paint(mark, state.map_or_else(Style::new, State::style)),
            &self.paint(letter, Style::new().dimmed()),
        );
        let (body, suffix) = self.text(row);
        let text = match &suffix {
            Some(suffix) => format!("{body} {}", suffix.text),
            None => body,
        };
        let lead = suffix.as_ref().map(|suffix| suffix.lead);
        let body_style = if state == Some(State::Resolved) {
            Style::new().dimmed()
        } else {
            Style::new()
        };

        let free = self.width.saturating_sub(prefix_width);
        let own_rails = row.own_rails();
        if free < TREE_MIN_TEXT_COLS {
            // One level in, under the node's own rails (sec-3 rule 2, as amended
            // at VH-1); the bare indent only where the rails would fill the line.
            let railed = [
                own_rails.as_str(),
                if row.has_children {
                    TREE_RAIL
                } else {
                    TREE_NO_RAIL
                },
            ]
            .concat();
            let indent = if self.width.saturating_sub(display_width(&railed)) < TREE_MIN_DROP_COLS {
                " ".repeat(TREE_DROP_INDENT)
            } else {
                railed
            };
            let wrapped = wrap(&text, self.width.saturating_sub(display_width(&indent)));
            return std::iter::once(prefix.trim_end().to_owned())
                .chain(
                    self.paint_text(&wrapped, body_style, lead)
                        .into_iter()
                        .map(|line| format!("{indent}{line}")),
                )
                .collect();
        }
        let continuation = format!(
            "{own_rails}{:<fill$}",
            if row.has_children {
                TREE_CHILD_RAIL
            } else {
                ""
            },
            fill = prefix_width - display_width(&own_rails),
        );
        let wrapped = wrap(&text, free);
        self.paint_text(&wrapped, body_style, lead)
            .into_iter()
            .enumerate()
            .map(|(nth, line)| match nth {
                0 => format!("{prefix}{line}"),
                _ => format!("{continuation}{line}"),
            })
            .collect()
    }

    /// The right-hand text (sec-3 line anatomy) and the cursor/pin suffix.
    fn text(&self, row: &Row<'_>) -> (String, Option<Suffix>) {
        let node = row.node;
        let answered = match &node.answer {
            Some(MapAnswer::Record { record, title, .. }) => match title {
                TitleLookup::Found(title) => format!("{record} {title}"),
                TitleLookup::NotFound => format!("{record} ({TREE_RECORD_NOT_FOUND})"),
                TitleLookup::Unreadable(reason) => {
                    format!("{record} ({TREE_RECORD_UNREADABLE}: {reason})")
                }
            },
            Some(MapAnswer::Note { form, note }) => format!("{form}: {note}"),
            None if State::of(node) == Some(State::Blocked) => format!(
                "{TREE_NEEDS} {}: {}",
                node.blocked_by.join(", "),
                node.question
            ),
            None => node.question.clone(),
        };
        let body = match (row.branch, &node.parent) {
            (None, Some(parent)) => format!("{TREE_PARENT} {parent}: {answered}"),
            _ => answered,
        };
        let run = &self.envelope.run;
        let cursor =
            (run.cursor.as_deref() == Some(node.id.as_str())).then_some(if run.cursor_stale {
                TREE_SUFFIX_STALE_CURSOR
            } else {
                TREE_SUFFIX_CURSOR
            });
        let pinned = self
            .envelope
            .pinned
            .as_ref()
            .is_some_and(|pin| pin.id == node.id)
            .then_some(TREE_SUFFIX_PINNED);
        let lead = if cursor.is_some() {
            TREE_CURSOR_STYLE
        } else {
            TREE_PINNED_STYLE
        };
        let marks: Vec<&str> = cursor.into_iter().chain(pinned).collect();
        let suffix = (!marks.is_empty()).then(|| Suffix {
            text: format!("{TREE_SUFFIX_ARROW} {}", marks.join(", ")),
            lead,
        });
        (body, suffix)
    }

    /// Colour wrapped text: the body in `body`; the suffix — everything from
    /// the last arrow on — in `lead`, its `pinned` word in [`TREE_PINNED_STYLE`].
    fn paint_text(&self, wrapped: &[String], body: Style, lead: Option<Style>) -> Vec<String> {
        let split = lead.and_then(|_| {
            wrapped
                .iter()
                .enumerate()
                .rev()
                .find_map(|(nth, line)| line.rfind(TREE_SUFFIX_ARROW).map(|at| (nth, at)))
        });
        let lead = lead.unwrap_or(body);
        wrapped
            .iter()
            .enumerate()
            .map(|(nth, line)| match split {
                Some((at_line, at)) if nth == at_line => {
                    let (head, tail) = line.split_at(at);
                    [self.paint(head, body), self.paint_suffix(tail, lead)].concat()
                }
                Some((at_line, _)) if nth > at_line => self.paint_suffix(line, lead),
                _ => self.paint(line, body),
            })
            .collect()
    }

    /// One line's share of the suffix: `lead`, then `pinned` magenta.
    fn paint_suffix(&self, part: &str, lead: Style) -> String {
        match part.rfind(TREE_SUFFIX_PINNED) {
            Some(at) => {
                let (head, pinned) = part.split_at(at);
                [
                    self.paint(head, lead),
                    self.paint(pinned, TREE_PINNED_STYLE),
                ]
                .concat()
            }
            None => self.paint(part, lead),
        }
    }

    fn paint(&self, text: &str, style: Style) -> String {
        paint(self.colour, text, style)
    }
}

/// Apply `style` when colour is on; plain text otherwise. Colour is additive —
/// no state is colour-only (`DEC-307`).
fn paint(colour: bool, text: &str, style: Style) -> String {
    if colour && !text.is_empty() {
        text.style(style).to_string()
    } else {
        text.to_owned()
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test code — the repo's panic-avoidance denials target production paths"
)]
mod tests {
    use std::collections::BTreeMap;

    use textwrap::core::display_width;

    use super::super::super::inquiry::{InquiryLifecycle, Provenance};
    use super::super::super::run::GateFacts;
    use super::super::super::snapshot::DesignSnapshot;
    use super::super::envelope::{
        Detail, MapAnswer, MapNode, OutstandingBySeverity, PinnedSlot, RunSelection,
        SkippedSnapshot, TitleLookup, TurnEnvelope, project,
    };
    use super::{
        TREE_BLOCKING_MARK, TREE_DROP_INDENT, TREE_MIN_DROP_COLS, TREE_MIN_TEXT_COLS, TreeStyle,
        render,
    };

    const PLAIN: TreeStyle = TreeStyle {
        width: None,
        colour: false,
    };

    fn at(width: u16) -> TreeStyle {
        TreeStyle {
            width: Some(width),
            colour: false,
        }
    }

    /// A full envelope for SL-266 carrying exactly `map` — the tree reads only
    /// the envelope, so a fixture may hold shapes admission never would.
    fn envelope(map: Vec<MapNode>) -> TurnEnvelope {
        let run = DesignSnapshot::new("dr-test", 266, None);
        let mut envelope = project(
            &run,
            0,
            Detail::Full,
            OutstandingBySeverity::default(),
            &GateFacts::default(),
            "SL-266",
            &BTreeMap::new(),
            None,
        )
        .expect("an empty run projects");
        envelope.map = map;
        envelope
    }

    fn node(
        id: &str,
        parent: Option<&str>,
        lifecycle: InquiryLifecycle,
        provenance: &'static str,
        question: &str,
    ) -> MapNode {
        MapNode {
            id: id.to_owned(),
            parent: parent.map(str::to_owned),
            question: question.to_owned(),
            lifecycle: lifecycle.as_str(),
            provenance,
            blocking: false,
            blocked_by: Vec::new(),
            answer: None,
        }
    }

    fn open(id: &str, parent: Option<&str>, question: &str) -> MapNode {
        node(
            id,
            parent,
            InquiryLifecycle::Open,
            Provenance::AGENT_PROPOSED,
            question,
        )
    }

    fn resolved(id: &str, answer: MapAnswer) -> MapNode {
        MapNode {
            answer: Some(answer),
            ..node(
                id,
                None,
                InquiryLifecycle::Resolved,
                Provenance::AGENT_PROPOSED,
                "answered?",
            )
        }
    }

    fn record(record: &str, title: TitleLookup) -> MapAnswer {
        MapAnswer::Record {
            form: "create",
            record: record.to_owned(),
            title,
        }
    }

    fn pin(envelope: &mut TurnEnvelope, id: &str) {
        envelope.pinned = Some(PinnedSlot {
            id: id.to_owned(),
            question: String::new(),
            lifecycle: InquiryLifecycle::Open.as_str(),
            blocked: false,
            authority: "user-pinned",
        });
    }

    /// One node of every lifecycle, a derived-blocked node, all four
    /// provenances, blocking and not, the cursor and the pin.
    fn anatomy() -> TurnEnvelope {
        let mut envelope = envelope(vec![
            MapNode {
                blocking: true,
                ..node(
                    "inq-root",
                    None,
                    InquiryLifecycle::Open,
                    Provenance::USER_DIRECTED,
                    "Does the map render?",
                )
            },
            MapNode {
                parent: Some("inq-root".to_owned()),
                ..resolved(
                    "inq-done",
                    record(
                        "DEC-303",
                        TitleLookup::Found(
                            "Full envelope carries the whole inquiry map".to_owned(),
                        ),
                    ),
                )
            },
            MapNode {
                blocking: true,
                blocked_by: vec!["inq-root".to_owned(), "inq-later".to_owned()],
                ..node(
                    "inq-wait",
                    Some("inq-root"),
                    InquiryLifecycle::Open,
                    Provenance::SHAPING_QUESTION,
                    "Which cache key?",
                )
            },
            node(
                "inq-later",
                Some("inq-root"),
                InquiryLifecycle::Deferred,
                Provenance::IMPORTED_PROSE,
                "Later, maybe?",
            ),
            node(
                "inq-cut",
                None,
                InquiryLifecycle::Pruned,
                Provenance::AGENT_PROPOSED,
                "Out of scope?",
            ),
            node(
                "inq-pin",
                None,
                InquiryLifecycle::Open,
                Provenance::USER_DIRECTED,
                "Pin me?",
            ),
        ]);
        envelope.run.cursor = Some("inq-root".to_owned());
        pin(&mut envelope, "inq-pin");
        envelope
    }

    fn line_of<'a>(lines: &'a [String], id: &str) -> &'a str {
        lines
            .iter()
            .find(|line| line.contains(&format!(" {id} ")))
            .map(String::as_str)
            .expect("a node line")
    }

    /// Glyphs a node prefix is built from — rails, branches, marks, letters.
    const PREFIX_GLYPHS: [&str; 3] = ["│", "├──", "└──"];
    const MARKS: [&str; 5] = ["●", "○", "◌", "◐", "⊘"];

    /// `EX-2` rule 4: a line wider than `width` holds a single unbreakable item —
    /// a node prefix alone, or one word beside its prefix or rails.
    fn over_width_holds_one_item(line: &str, width: usize, ids: &[&str], footer: &str) -> bool {
        if display_width(line) <= width || line == footer {
            return true;
        }
        let mut tokens = line.split_whitespace().peekable();
        while tokens
            .next_if(|token| PREFIX_GLYPHS.contains(token))
            .is_some()
        {}
        if tokens.next_if(|token| MARKS.contains(token)).is_some() {
            // mark, letter, optional blocking mark, id
            tokens.next();
            tokens.next_if(|token| *token == TREE_BLOCKING_MARK);
            let id = tokens.next();
            assert!(
                id.is_some_and(|id| ids.contains(&id)),
                "a prefix ends with its id: {line}"
            );
        }
        tokens.count() <= 1
    }

    fn assert_no_line_breaks_rule_4(lines: &[String], width: usize, ids: &[&str]) {
        let footer = lines.last().expect("a footer");
        for line in lines {
            assert!(
                over_width_holds_one_item(line, width, ids, footer),
                "an over-width line holds more than one item at width {width}: {line:?}"
            );
            assert!(!line.contains('…'), "nothing is elided: {line:?}");
        }
    }

    /// Strip SGR escapes — spelled locally, because this leaf names no `crate::`
    /// item, tests included (`e2e_design_*` include the tree standalone).
    fn strip_ansi(line: &str) -> String {
        let mut out = String::with_capacity(line.len());
        let mut chars = line.chars();
        while let Some(char) = chars.next() {
            if char == '\u{1b}' {
                chars.by_ref().find(|inner| *inner == 'm');
            } else {
                out.push(char);
            }
        }
        out
    }

    fn words(text: &str) -> Vec<&str> {
        text.split_whitespace().collect()
    }

    /// `VT-1` (design `VT-3`, text half).
    #[test]
    fn title_lookups_render_their_own_text() {
        let lines = render(
            &envelope(vec![
                resolved(
                    "inq-found",
                    record(
                        "DEC-310",
                        TitleLookup::Found(
                            "Relay instruction rides design apply output".to_owned(),
                        ),
                    ),
                ),
                resolved(
                    "inq-que",
                    MapAnswer::Record {
                        form: "adopt",
                        record: "QUE-7".to_owned(),
                        title: TitleLookup::Found("Which cache key?".to_owned()),
                    },
                ),
                resolved("inq-missing", record("DEC-999", TitleLookup::NotFound)),
                resolved(
                    "inq-unread",
                    record(
                        "DEC-998",
                        TitleLookup::Unreadable("permission denied".to_owned()),
                    ),
                ),
                resolved(
                    "inq-note",
                    MapAnswer::Note {
                        form: "non-durable",
                        note: "said in the chat".to_owned(),
                    },
                ),
                resolved(
                    "inq-kept",
                    MapAnswer::Note {
                        form: "unresolved",
                        note: "revisit after launch".to_owned(),
                    },
                ),
            ]),
            PLAIN,
        );
        for (id, text) in [
            (
                "inq-found",
                "DEC-310 Relay instruction rides design apply output",
            ),
            ("inq-que", "QUE-7 Which cache key?"),
            ("inq-missing", "DEC-999 (record not found)"),
            ("inq-unread", "DEC-998 (unreadable: permission denied)"),
            ("inq-note", "non-durable: said in the chat"),
            ("inq-kept", "unresolved: revisit after launch"),
        ] {
            assert!(
                line_of(&lines, id).ends_with(text),
                "{id} renders {text:?}: {lines:#?}"
            );
        }
    }

    /// `VT-2` (design `VT-4`).
    #[test]
    fn anatomy_golden() {
        let golden = "\
SL-266 · exploring · rev 1 · 6 questions: 1 resolved, 2 open, 1 blocked, 1 deferred, 1 pruned
├── ○ u * inq-root   Does the map render? ← cursor
│   ├── ● a   inq-done   DEC-303 Full envelope carries the whole inquiry map
│   ├── ◌ s * inq-wait   needs inq-root, inq-later: Which cache key?
│   └── ◐ i   inq-later  Later, maybe?
├── ⊘ a   inq-cut    Out of scope?
└── ○ u   inq-pin    Pin me? ← pinned
● resolved ○ open ◌ blocked ◐ deferred ⊘ pruned · * blocking · u user a agent s shaping i imported
doctrine design tree SL-266";
        assert_eq!(render(&anatomy(), PLAIN).join("\n"), golden);

        let mut both = anatomy();
        both.run.cursor = Some("inq-pin".to_owned());
        assert!(line_of(&render(&both, PLAIN), "inq-pin").ends_with("Pin me? ← cursor, pinned"));

        let mut stale = anatomy();
        stale.run.cursor_stale = true;
        assert!(
            line_of(&render(&stale, PLAIN), "inq-root")
                .ends_with("← cursor (moved up: declared cursor is settled)")
        );
    }

    /// `VT-3` (design `VT-5`) — counts, and no word that certifies completeness.
    #[test]
    fn header_never_certifies() {
        let lines = render(
            &envelope(vec![
                resolved("inq-a", record("DEC-1", TitleLookup::NotFound)),
                resolved("inq-b", record("DEC-2", TitleLookup::NotFound)),
            ]),
            PLAIN,
        );
        let header = lines.first().expect("a header");
        assert!(
            header.ends_with("2 questions: 2 resolved, 0 open"),
            "{header}"
        );
        for word in ["complete", "done", "finished", "all"] {
            assert!(!header.contains(word), "{word:?} in {header}");
        }
    }

    /// `VT-4` (design `VT-6`) — an absent parent and a parent cycle.
    #[test]
    fn unplaced_nodes_are_rendered_not_dropped() {
        let map = vec![
            open("inq-a", None, "rooted?"),
            open("inq-b", Some("inq-gone"), "orphaned?"),
            open("inq-c", Some("inq-d"), "cycle one?"),
            open("inq-d", Some("inq-c"), "cycle two?"),
            open("inq-e", Some("inq-b"), "orphan's child?"),
        ];
        let ids: Vec<String> = map.iter().map(|node| node.id.clone()).collect();
        let lines = render(&envelope(map), PLAIN);
        assert!(lines[0].contains("5 questions"), "{}", lines[0]);
        let heading = lines
            .iter()
            .position(|line| line == "unplaced (parent missing or cyclic):")
            .expect("an unplaced section");
        for id in &ids {
            let found: Vec<usize> = lines
                .iter()
                .enumerate()
                .filter(|(_, line)| line.contains(&format!(" {id} ")))
                .map(|(at, _)| at)
                .collect();
            assert_eq!(found.len(), 1, "{id} renders exactly once: {lines:#?}");
            assert_eq!(found[0] > heading, id != "inq-a", "{id}'s placement");
        }
        assert!(line_of(&lines, "inq-b").ends_with("parent inq-gone: orphaned?"));
        assert!(line_of(&lines, "inq-c").ends_with("parent inq-d: cycle one?"));
    }

    /// `VT-5` (design `VT-7`).
    #[test]
    fn wrapping_and_overflow() {
        let long = "the resolver keys its cache on the canonical path, not the \
                    display path, so two spellings of one file share an entry";

        // Width 80: text wraps under its column, ancestors' rails continued.
        let lines = render(
            &envelope(vec![
                open("inq-top", None, "first?"),
                open("inq-kid", Some("inq-top"), long),
                open("inq-end", None, "last?"),
            ]),
            at(80),
        );
        let kid = lines
            .iter()
            .position(|line| line.contains(" inq-kid "))
            .expect("the child");
        let first = lines[kid]
            .split("inq-kid")
            .nth(1)
            .expect("text")
            .trim_start();
        let column = display_width(&lines[kid]) - display_width(first);
        let continued: Vec<&String> = lines[kid + 1..]
            .iter()
            .take_while(|line| line.starts_with("│   "))
            .collect();
        assert!(!continued.is_empty(), "the long question wraps: {lines:#?}");
        let mut text = vec![first.to_owned()];
        for line in &continued {
            assert!(display_width(line) <= 80, "{line:?}");
            let (rails, rest) =
                line.split_at(line.len() - line.trim_start_matches(['│', ' ']).len());
            assert_eq!(display_width(rails), column, "under its column: {line:?}");
            text.push(rest.to_owned());
        }
        assert_eq!(words(&text.join(" ")), words(long), "no word lost");

        // Below `TREE_MIN_TEXT_COLS` free columns the text drops a line and
        // keeps its rails: one level in, a child rail when it has children.
        let lines = render(
            &envelope(vec![
                open("inq-x", None, long),
                open("inq-y", Some("inq-x"), "kid?"),
                open("inq-z", None, "last?"),
            ]),
            at(40),
        );
        let node = lines
            .iter()
            .position(|line| line.ends_with(" inq-x"))
            .expect("a prefix-only node line");
        assert!(40 - display_width(&lines[node]) - 2 < TREE_MIN_TEXT_COLS);
        let rails = "│   │   ";
        let dropped: Vec<&String> = lines[node + 1..]
            .iter()
            .take_while(|line| line.starts_with(rails))
            .filter(|line| !line.contains(" inq-y"))
            .collect();
        assert!(
            dropped.len() > 1,
            "the text wraps under its rails: {lines:#?}"
        );
        for line in &dropped {
            assert!(!line[rails.len()..].starts_with(' '), "{line:?}");
            assert!(display_width(line) <= 40, "{line:?}");
        }
        let dropped: Vec<&str> = dropped.iter().map(|line| &line[rails.len()..]).collect();
        assert_eq!(words(&dropped.join(" ")), words(long));

        // Depth 12 at width 40: the prefix overflows, no text is lost.
        let chain: Vec<String> = (0..13).map(|depth| format!("inq-d{depth}")).collect();
        let map: Vec<MapNode> = chain
            .iter()
            .enumerate()
            .map(|(depth, id)| {
                let parent = depth.checked_sub(1).map(|up| chain[up].as_str());
                open(id, parent, if depth == 12 { long } else { "deeper?" })
            })
            .collect();
        let ids: Vec<&str> = chain.iter().map(String::as_str).collect();
        let lines = render(&envelope(map), at(40));
        assert_no_line_breaks_rule_4(&lines, 40, &ids);
        let deepest = lines
            .iter()
            .position(|line| line.ends_with(" inq-d12"))
            .expect("the deepest node");
        assert!(display_width(&lines[deepest]) > 40, "its prefix overflows");
        // Its rails leave fewer than `TREE_MIN_DROP_COLS`: the bare indent.
        assert!(40 < 13 * display_width("│   ") + TREE_MIN_DROP_COLS);
        let indent = " ".repeat(TREE_DROP_INDENT);
        let tail: Vec<&str> = lines[deepest + 1..]
            .iter()
            .take_while(|line| line.starts_with(&indent))
            .map(|line| line.trim())
            .collect();
        assert_eq!(words(&tail.join(" ")), words(long));

        // A wide-Unicode label is measured in columns, not chars or bytes.
        let lines = render(
            &envelope(vec![
                open("inq-設計", None, "wide?"),
                open("inq-abcd", None, "narrow?"),
            ]),
            PLAIN,
        );
        let text_column = |id: &str| {
            let line = line_of(&lines, id);
            display_width(line) - display_width(line.split(id).nth(1).expect("text").trim_start())
        };
        assert_eq!(text_column("inq-設計"), text_column("inq-abcd"));

        // The legend breaks only between entries.
        let lines = render(&anatomy(), at(30));
        let legend_start = lines
            .iter()
            .position(|line| line.starts_with("● resolved"))
            .expect("a legend");
        let legend = &lines[legend_start..lines.len() - 1];
        assert!(legend.len() > 1, "the legend wraps at 30: {legend:#?}");
        let glyphs = [&MARKS[..], &["·", TREE_BLOCKING_MARK, "u", "a", "s", "i"]].concat();
        for line in legend {
            assert!(display_width(line) <= 30, "{line:?}");
            let first = line.split_whitespace().next().expect("an entry");
            let last = line.split_whitespace().last().expect("an entry");
            assert!(glyphs.contains(&first), "starts on an entry: {line:?}");
            assert!(!glyphs.contains(&last), "ends on an entry: {line:?}");
        }

        // Width 16: header, `chosen:` and `skipped` lines wrap between words.
        let mut selected = anatomy();
        selected.selection = Some(RunSelection {
            candidates: 3,
            skipped: vec![SkippedSnapshot {
                path: ".doctrine/state/slice/247/design.toml".to_owned(),
                reason: "expected a table".to_owned(),
            }],
        });
        let lines = render(&selected, at(16));
        let ids = [
            "inq-root",
            "inq-done",
            "inq-wait",
            "inq-later",
            "inq-cut",
            "inq-pin",
        ];
        assert_no_line_breaks_rule_4(&lines, 16, &ids);
        let free = lines
            .iter()
            .take_while(|line| !line.starts_with("├──"))
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" ");
        for expected in [
            "SL-266 · exploring · rev 1 · 6 questions: 1 resolved, 2 open, 1 blocked, 1 deferred, 1 pruned",
            "chosen: newest of 3 runs open when scanned",
            "skipped .doctrine/state/slice/247/design.toml: expected a table",
        ] {
            assert!(free.contains(expected), "{expected:?} in {free:?}");
        }
        for width in [16, 24, 40, 80] {
            assert_no_line_breaks_rule_4(&render(&selected, at(width)), usize::from(width), &ids);
        }
    }

    /// `VT-6` (design `VT-8`).
    #[test]
    fn colour_is_additive() {
        for width in [None, Some(40), Some(16)] {
            let plain = render(
                &anatomy(),
                TreeStyle {
                    width,
                    colour: false,
                },
            );
            let coloured = render(
                &anatomy(),
                TreeStyle {
                    width,
                    colour: true,
                },
            );
            assert!(plain.iter().all(|line| !line.contains('\u{1b}')));
            assert!(coloured.iter().any(|line| line.contains('\u{1b}')));
            let stripped: Vec<String> = coloured.iter().map(|line| strip_ansi(line)).collect();
            assert_eq!(stripped, plain, "at {width:?}");
            let pinned = super::paint(true, super::TREE_SUFFIX_PINNED, super::TREE_PINNED_STYLE);
            let root = coloured
                .iter()
                .position(|line| strip_ansi(line).starts_with("├── ○ u * inq-root"))
                .expect("the root");
            let cursor_lines = coloured[root..]
                .iter()
                .take(6)
                .cloned()
                .collect::<Vec<_>>()
                .join("");
            let cursor = format!("{} {}", super::TREE_SUFFIX_ARROW, super::TREE_SUFFIX_CURSOR);
            assert!(
                cursor_lines.contains(&super::paint(true, &cursor, super::TREE_CURSOR_STYLE)),
                "the cursor takes the open mark's colour: {cursor_lines:?}"
            );
            assert!(
                coloured.iter().any(|line| line.contains(&pinned)),
                "the pin is magenta"
            );
            let start = coloured
                .iter()
                .position(|line| strip_ansi(line).starts_with(super::TREE_MARK_RESOLVED))
                .expect("a legend");
            let legend = coloured[start..coloured.len() - 1].join(" ");
            for glyph in [
                super::paint(
                    true,
                    super::TREE_MARK_RESOLVED,
                    super::State::Resolved.style(),
                ),
                super::paint(true, super::TREE_MARK_OPEN, super::State::Open.style()),
            ] {
                assert!(
                    legend.contains(&glyph),
                    "the legend paints its marks: {legend:?}"
                );
            }
        }
    }

    /// The `chosen:` line names the only candidate differently from several.
    #[test]
    fn selection_discloses_how_the_run_was_chosen() {
        let mut only = anatomy();
        only.selection = Some(RunSelection {
            candidates: 1,
            skipped: Vec::new(),
        });
        assert_eq!(
            render(&only, PLAIN)[1],
            "chosen: the only run open when scanned"
        );
    }
}
