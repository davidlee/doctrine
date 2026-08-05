//! The published stage-machine diagram — the design run's four stages, its four
//! forward edges, and what each edge enforces, rendered as one shipped document.
//!
//! A sibling of [`super::gate`] rather than more of it: that module is about
//! *gating*, this one is about *describing*. Out-degree into `gate` only, and
//! rendering a string is pure, so the layering is untouched.
//!
//! Every table, cell and edge here is derived from the gate's macro output —
//! `Condition::ALL`, `CONTRACTS`, `boundary_conditions`, `cumulative_conditions`,
//! `Advance::ALL` and `boundary_runbook`. There is deliberately **no second
//! list**: a hand-written table would pass the golden (which pins bytes to code,
//! and a hand-written table *is* code) and would silently survive a vocabulary
//! change.

use super::Stage;
use super::gate::{Advance, cumulative_conditions};

/// Render the shipped stage-machine document.
pub(crate) fn render_artifact() -> String {
    let mut out = String::from(ARTIFACT_HEADER);
    out.push_str(&diagram());
    out
}

/// The stage machine, as a mermaid `stateDiagram-v2`.
///
/// Each edge is labelled with the count of conditions the crossing **enforces**,
/// not with its own boundary rows: the inherited ones are what a reader standing
/// at the top edge is least likely to have seen, and the difference between the
/// two numbers is the whole reason the edge table below exists.
///
/// Origin comes from walking [`Stage::ALL`] through [`Advance::from_stage`]
/// rather than from an `Advance::from()` accessor. `from_stage` is total on
/// non-terminal stages and yields the origin and the edge in one step, and the
/// accessor was refused for a stated reason: it would reconstitute the
/// `(Stage, Stage)` pair [`Advance`] exists to replace.
fn diagram() -> String {
    let mut out = String::from(DIAGRAM_OPEN);
    out.extend(
        Stage::ALL
            .first()
            .map(|entry| format!("    [*] --> {}\n", entry.as_str())),
    );
    out.extend(Stage::ALL.into_iter().filter_map(|stage| {
        Advance::from_stage(stage).map(|edge| {
            format!(
                "    {} --> {}: enforces {}\n",
                stage.as_str(),
                edge.to().as_str(),
                cumulative_conditions(edge.to()).len(),
            )
        })
    }));
    out.push_str(DIAGRAM_CLOSE);
    out
}

/// The artefact's fixed preamble — the shipped-generated-asset banner, the title
/// and the intro.
///
/// The banner states **provenance, not an edit procedure**. `funnel-machine.md`'s
/// change-the-table-and-re-render wording is correct in this repository and
/// useless in a client project, where there is no table and the file is a
/// read-only artefact of an installed binary. This wording is the policy for the
/// first generated asset in the shipped corpus; a second one reuses it rather
/// than inventing a variant.
const ARTIFACT_HEADER: &str = "\
<!-- GENERATED — rendered from Doctrine's design-run gate tables and pinned to
     them by test. Not hand-editable, and not overridable: this file describes
     the machine your installed `doctrine` binary actually enforces, so an edited
     copy would describe a machine that does not exist. -->

# Design run — stage machine

The machine a design run stands in: five stages, four guarded forward edges, and
the conditions each edge enforces. Every table below is rendered from the gate
your `doctrine` binary evaluates, so what you read here is what refuses you.
";

/// The diagram section's fixed opening.
const DIAGRAM_OPEN: &str = "
## The machine

Each edge is labelled with the number of conditions the crossing enforces —
its own, plus everything it inherits.

```mermaid
stateDiagram-v2
";

/// The diagram section's fixed close (the fenced block's terminator).
const DIAGRAM_CLOSE: &str = "```\n";

#[cfg(test)]
mod tests {
    use super::*;

    /// Every arrow inside the mermaid block, minus mermaid's entry marker — the
    /// rendered transition set `VT-2`'s edge half is about.
    ///
    /// Scoped to the fenced block rather than to the whole document, because the
    /// provenance banner is an HTML comment and its terminator is also `-->`.
    fn transitions(rendered: &str) -> Vec<String> {
        rendered
            .lines()
            .skip_while(|line| !line.starts_with("```mermaid"))
            .skip(1)
            .take_while(|line| !line.starts_with("```"))
            .map(str::trim)
            .filter(|line| line.contains("-->") && !line.starts_with("[*]"))
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn the_render_opens_with_the_provenance_banner_and_the_title() {
        let rendered = render_artifact();
        assert!(
            rendered.starts_with(ARTIFACT_HEADER),
            "the artefact opens with the fixed banner, title and intro"
        );
    }

    /// `VT-2`, edge half.
    #[test]
    fn every_edge_appears_exactly_once() {
        let rendered = render_artifact();
        let drawn = transitions(&rendered);

        for stage in Stage::ALL {
            let Some(edge) = Advance::from_stage(stage) else {
                continue;
            };
            let expected = format!(
                "{} --> {}: enforces {}",
                stage.as_str(),
                edge.to().as_str(),
                cumulative_conditions(edge.to()).len()
            );
            assert_eq!(
                drawn.iter().filter(|line| **line == expected).count(),
                1,
                "the diagram draws `{expected}` exactly once"
            );
        }

        assert_eq!(
            drawn.len(),
            Advance::ALL.len(),
            "the diagram's transition set is Advance's four values and nothing else"
        );
    }
}
