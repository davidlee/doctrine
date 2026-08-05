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
use super::gate::{
    Advance, CONTRACTS, Condition, Contract, DerivationRule, boundary_conditions, boundary_runbook,
    cumulative_conditions,
};

/// Render the shipped stage-machine document.
pub(crate) fn render_artifact() -> String {
    let mut out = String::from(ARTIFACT_HEADER);
    out.push_str(&diagram());
    out.push_str(&edge_table());
    out.push_str(&condition_table());
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

/// One row per forward edge: what it guards, what it discharges, and — the
/// column this document exists for — what it **inherits**.
///
/// The inherited set is the enforced set minus the edge's own boundary rows,
/// computed rather than listed: an edge's own rows are local knowledge an agent
/// standing there already has, and the remainder is the part that refuses it for
/// something it did two stages ago.
fn edge_table() -> String {
    let mut out = String::from(EDGES_OPEN);
    out.extend(Advance::ALL.into_iter().map(|edge| {
        let enforced = cumulative_conditions(edge.to());
        let own = boundary_conditions(edge);
        format!(
            "| {} | {} | {} | {} | {} |\n",
            edge.as_str(),
            boundary_runbook(edge).name(),
            conditions(own.iter().copied()),
            conditions(
                enforced
                    .iter()
                    .copied()
                    .filter(|condition| !own.contains(condition))
            ),
            enforced.len(),
        )
    }));
    out
}

/// One row per condition, and beneath it the act each one is discharged by.
///
/// Every column is a projection of the same `CONTRACTS` row a contract receipt
/// injects its header from, so the two cannot disagree about what a condition
/// requires.
///
/// The discharging act rides a list rather than a fifth column: one row's remedy
/// is three lines, and a multi-line cell does not survive a markdown table.
/// Flattening it would forfeit the byte equality with what the receipt injects,
/// which is the only thing making *the same obligation, said once* checkable.
fn condition_table() -> String {
    let mut out = String::from(CONDITIONS_OPEN);
    out.extend(CONTRACTS.iter().map(|(condition, contract)| {
        format!(
            "| `{}` | {} | {} | {} |\n",
            condition.as_str(),
            contract.derivation.kind().as_str(),
            binding(contract),
            contract.reach.as_str(),
        )
    }));
    out.push_str(DISCHARGE_OPEN);
    out.extend(CONTRACTS.iter().map(|(condition, contract)| {
        format!("- `{}` — {}\n", condition.as_str(), contract.remedy())
    }));
    out
}

/// What a contract binds to, and therefore what invalidates a discharge of it —
/// the subject the derivation names, in the receipt's own field order.
fn binding(contract: &Contract) -> String {
    match contract.derivation {
        DerivationRule::Engine(source) => format!("engine({})", source.as_str()),
        DerivationRule::Attested(rule) => {
            let mut fields = vec![rule.binding.coverage.as_str().to_owned()];
            if !rule.binding.observed.is_empty() {
                let observed: Vec<&str> = rule
                    .binding
                    .observed
                    .iter()
                    .map(|fact| fact.as_str())
                    .collect();
                fields.push(format!("observes({})", observed.join(",")));
            }
            fields.join(" ")
        }
    }
}

/// A cell listing condition tokens, or the placeholder where there are none —
/// an empty markdown cell reads as a rendering fault rather than as a fact.
fn conditions(rows: impl Iterator<Item = Condition>) -> String {
    let cell = rows
        .map(|condition| format!("`{}`", condition.as_str()))
        .collect::<Vec<_>>()
        .join(", ");
    if cell.is_empty() {
        return NO_CONDITIONS.to_owned();
    }
    cell
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

/// Fixed prose and header opening the derived edge table.
const EDGES_OPEN: &str = "
## The edges

Each edge is guarded by its own conditions **and** by every cumulative condition
below it, re-derived against current content. A condition discharged two stages
ago is enforced again here; that is what the inherited column lists, and it is
the fact no single edge's view carries.

| edge | runbook | own conditions | inherited | enforces |
| --- | --- | --- | --- | --- |
";

/// Fixed prose and header opening the derived condition table.
const CONDITIONS_OPEN: &str = "
## The conditions

`kind` says who authors the state a condition is derived over: a `derived` row is
recomputed from the run's own state, an `attested` one from acts an actor
recorded. `binding` says what invalidates it — the subject the derivation names,
plus any canonical fact outside the run that must also still match. `reach` says
whether discharging it once is the end of it: an `edge-local` row is judged by the
edge that names it, a `cumulative` one by every edge above it as well.

| condition | kind | binding | reach |
| --- | --- | --- | --- |
";

/// Fixed prose opening the discharge list.
const DISCHARGE_OPEN: &str = "
### Discharging them

What each condition requires of you. A refusal names the conditions that failed;
these are the acts that clear them.

";

/// The placeholder for a cell that lists no conditions.
const NO_CONDITIONS: &str = "—";

/// The edge table's inherited column, by index — the cell `VT-3` reads back.
#[cfg(test)]
const INHERITED_COLUMN: usize = 3;

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::super::prompt::contract_block;
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

    /// One markdown row's cells, keyed by the token in its first column.
    fn row(rendered: &str, key: &str) -> Vec<String> {
        let prefix = format!("| {key} |");
        let line = rendered
            .lines()
            .find(|line| line.starts_with(&prefix))
            .unwrap_or_else(|| panic!("the artefact carries a row for `{key}`"));
        line.trim_matches('|')
            .split('|')
            .map(|cell| cell.trim().to_owned())
            .collect()
    }

    /// The condition tokens one cell lists — the no-conditions placeholder reads
    /// back as the empty set, which is what makes the floor assertions below
    /// distinguishable from a cell that failed to render at all.
    fn cell_tokens(cell: &str) -> Vec<String> {
        if cell == NO_CONDITIONS {
            return Vec::new();
        }
        cell.split(", ")
            .map(|token| token.trim_matches('`').to_owned())
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

    /// `VT-3` — the column the artefact exists for, and the one a hand-render
    /// gets wrong.
    #[test]
    fn inherited_column_is_the_enforced_set() {
        let rendered = render_artifact();

        for edge in Advance::ALL {
            let own = boundary_conditions(edge);
            let expected: Vec<String> = cumulative_conditions(edge.to())
                .into_iter()
                .filter(|condition| !own.contains(condition))
                .map(|condition| condition.as_str().to_owned())
                .collect();
            let cells = row(&rendered, edge.as_str());
            assert_eq!(
                cells
                    .get(INHERITED_COLUMN)
                    .map(String::as_str)
                    .map(cell_tokens),
                Some(expected),
                "`{}` inherits exactly the enforced set minus its own rows",
                edge.as_str()
            );
        }

        // The floor: a rendering that dropped the column entirely would satisfy
        // the equality above on the empty set for every edge.
        let locked = row(&rendered, Advance::ReviewingLocked.as_str());
        assert_eq!(
            locked
                .get(INHERITED_COLUMN)
                .map(String::as_str)
                .map(cell_tokens)
                .unwrap_or_default()
                .len(),
            5,
            "the top edge inherits five of the eight it enforces"
        );
        let first = row(&rendered, Advance::ExploringInquiring.as_str());
        assert!(
            first
                .get(INHERITED_COLUMN)
                .map(String::as_str)
                .map(cell_tokens)
                .unwrap_or_default()
                .is_empty(),
            "the bottom edge inherits nothing"
        );
    }

    /// `VT-2`, condition half.
    #[test]
    fn every_condition_appears_exactly_once() {
        let rendered = render_artifact();

        for condition in Condition::ALL {
            let prefix = format!("| `{}` |", condition.as_str());
            assert_eq!(
                rendered
                    .lines()
                    .filter(|line| line.starts_with(&prefix))
                    .count(),
                1,
                "`{}` has exactly one condition row",
                condition.as_str()
            );
        }

        assert_eq!(
            rendered
                .lines()
                .filter(|line| line.starts_with("| `"))
                .count(),
            Condition::ALL.len(),
            "the condition table's key set is the vocabulary and nothing else"
        );
    }

    /// `VT-4` — this table and the contract receipt are two renderings of one
    /// `CONTRACTS` row, and they may not disagree.
    #[test]
    fn diagram_and_contract_headers_agree() {
        let rendered = render_artifact();

        for (condition, _) in CONTRACTS {
            let edge = Advance::ALL
                .into_iter()
                .find(|edge| boundary_conditions(*edge).contains(&condition))
                .unwrap_or_else(|| panic!("`{}` guards an edge", condition.as_str()));
            let block = contract_block(edge, &BTreeMap::new());

            let header = format!("contract {} ", condition.as_str());
            let position = block
                .iter()
                .position(|line| line.starts_with(&header))
                .unwrap_or_else(|| {
                    panic!("the receipt injects a header for `{}`", condition.as_str())
                });
            let injected: Vec<&str> = block
                .get(position)
                .map(|line| line.split(' ').skip(2).collect())
                .unwrap_or_default();
            let discharge = block
                .get(position + 1)
                .and_then(|line| line.strip_prefix("  discharge: "))
                .unwrap_or_else(|| {
                    panic!(
                        "the receipt injects a discharge for `{}`",
                        condition.as_str()
                    )
                });

            let cells = row(&rendered, &format!("`{}`", condition.as_str()));
            assert_eq!(
                cells.get(1..).unwrap_or_default().join(" "),
                injected.join(" "),
                "`{}`'s kind, binding and reach are what the receipt injects",
                condition.as_str()
            );
            assert!(
                rendered.contains(&format!("- `{}` — {discharge}", condition.as_str())),
                "`{}`'s discharge is the receipt's, verbatim",
                condition.as_str()
            );
        }
    }
}
