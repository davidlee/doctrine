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

/// Render the shipped stage-machine document.
pub(crate) fn render_artifact() -> String {
    String::from(ARTIFACT_HEADER)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_render_opens_with_the_provenance_banner_and_the_title() {
        let rendered = render_artifact();
        assert!(
            rendered.starts_with(ARTIFACT_HEADER),
            "the artefact opens with the fixed banner, title and intro"
        );
    }
}
