// SPDX-License-Identifier: GPL-3.0-only
//! RV ledger transition table (SL-268 D4, engine tier): the acts, the `can`
//! predicate and the admissible from-set it implies, and the finding-scoped
//! edit-preserving `toml_edit` writes that apply an act and journal its turn.

use super::schema::FindingRow;
use super::vocab::{FINDING_STATUSES, FindingStatus, Role, Severity, Vocab};

/// Every act a turn can assert (SL-268 D1): the seven finding transitions plus
/// the pass-level `conclude`. One vocabulary serves the role gate, the
/// transition table, the refusal and the turn journal's `act` string; `status`
/// and the read/coordination verbs are not acts and are not modelled here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Act {
    Raise,
    Dispose,
    /// The responder updates an already-answered finding's response and/or its
    /// disposition/route (design sec-4, SL-268 PHASE-05) — `answered` stays
    /// `answered`.
    Amend,
    Verify,
    Contest,
    /// The raiser reopens a verified finding, handing it back to the responder
    /// (design sec-4, SL-268 PHASE-05). Clears `[review].concluded` in the same
    /// write as its turn (D2, [`Act::clears_concluded`]).
    Reopen,
    Withdraw,
    /// The raiser declares the pass finished (SL-244 `sec-4`, IMP-392). It moves
    /// no finding, so the transition table admits no edge for it.
    Conclude,
}

impl Act {
    /// The act's name — the refusal label and the journal's `act` string.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Raise => "raise",
            Self::Dispose => "dispose",
            Self::Amend => "amend",
            Self::Verify => "verify",
            Self::Contest => "contest",
            Self::Reopen => "reopen",
            Self::Withdraw => "withdraw",
            Self::Conclude => "conclude",
        }
    }

    /// The role an act statically requires, knowable without a finding (design
    /// §6 responsibility split): dispose/amend are the responder's; every other
    /// act is the raiser's. Concluding is the reviewer saying *I have finished
    /// reading*, so it is deliberately not `dispose`'s. The `with_turn` wrapper
    /// checks this; the per-finding `can` check (state-dependent) is the
    /// closure's job.
    pub(crate) const fn required_role(self) -> Role {
        match self {
            Self::Raise
            | Self::Verify
            | Self::Contest
            | Self::Reopen
            | Self::Withdraw
            | Self::Conclude => Role::Raiser,
            Self::Dispose | Self::Amend => Role::Responder,
        }
    }

    /// Whether the act clears the pass's `[review].concluded` marker in the
    /// write that journals its turn (SL-268 D2, RV-396 `F-4`, design sec-3): a
    /// new finding or a reopened one means the pass is no longer finished, so
    /// `done` needs a fresh `conclude`. The one table — the turn guard reads it.
    pub(crate) const fn clears_concluded(self) -> bool {
        matches!(self, Self::Raise | Self::Reopen)
    }

    /// The flag that carries the act's required free-text account — `--basis`
    /// for `conclude` (what the pass examined), `--note` for every other act.
    /// The CLI refusal and the MCP error both name it from here.
    pub(crate) const fn note_flag(self) -> &'static str {
        match self {
            Self::Conclude => "basis",
            _ => "note",
        }
    }
}

// ---------------------------------------------------------------------------
// Transition predicate (design §5, D-C4/D-C5)
// ---------------------------------------------------------------------------

/// Whether `act` may fire on a finding currently in `from` (or, for `raise`,
/// not yet existing — `None`) when asserted by `role`. Pure and total: the
/// single-owner edge table (design §5), every other combination refused —
/// `conclude` included, since it moves no finding.
///
/// | act      | from               | role      | → |
/// |----------|--------------------|-----------|---|
/// | raise    | (none)             | raiser    | open |
/// | dispose  | open \| contested  | responder | answered |
/// | amend    | answered           | responder | answered |
/// | verify   | answered           | raiser    | verified (terminal) |
/// | contest  | answered           | raiser    | contested |
/// | reopen   | verified           | raiser    | contested |
/// | withdraw | open \| answered   | raiser    | withdrawn (terminal) |
pub(crate) const fn can(act: Act, from: Option<FindingStatus>, role: Role) -> bool {
    // Static role check first — the half `with_turn` also owns; refuse a
    // role/act mismatch regardless of state.
    if !role_eq(role, act.required_role()) {
        return false;
    }
    matches!(
        (act, from),
        (Act::Raise, None)
            | (
                Act::Dispose,
                Some(FindingStatus::Open | FindingStatus::Contested)
            )
            | (
                Act::Amend | Act::Verify | Act::Contest,
                Some(FindingStatus::Answered)
            )
            | (Act::Reopen, Some(FindingStatus::Verified))
            | (
                Act::Withdraw,
                Some(FindingStatus::Open | FindingStatus::Answered)
            )
    )
}

/// Const-context `Role` equality (the derived `PartialEq` is not `const`).
const fn role_eq(a: Role, b: Role) -> bool {
    matches!(
        (a, b),
        (Role::Raiser, Role::Raiser) | (Role::Responder, Role::Responder)
    )
}

/// The statuses `act` may fire from when asserted by its own role, in vocabulary
/// order — the from-set a state refusal reports (SL-268 sec-4). Computed from
/// [`can`], so the refusal cannot disagree with the table it explains.
pub(crate) fn admissible_from(act: Act) -> Vec<FindingStatus> {
    FINDING_STATUSES
        .iter()
        .filter_map(|s| FindingStatus::parse(s).ok())
        .filter(|s| can(act, Some(*s), act.required_role()))
        .collect()
}

// ---------------------------------------------------------------------------
// Finding-scoped edit-preserving toml_edit (the governance.rs:290 pattern,
// extended to a `[[finding]]` array element). Comments / unknown keys survive.
// ---------------------------------------------------------------------------

/// Locate the `[[finding]]` table whose `id == finding_id`, returning a mutable
/// handle. The lookup is by the authored `id` field, never by array position (an
/// append-only ledger never renumbers, but order is not identity).
pub(crate) fn finding_table_mut<'a>(
    doc: &'a mut toml_edit::DocumentMut,
    finding_id: &str,
) -> anyhow::Result<&'a mut toml_edit::Table> {
    let array = doc
        .get_mut("finding")
        .and_then(toml_edit::Item::as_array_of_tables_mut)
        .ok_or_else(|| anyhow::anyhow!("ledger has no findings"))?;
    array
        .iter_mut()
        .find(|t| t.get("id").and_then(toml_edit::Item::as_str) == Some(finding_id))
        .ok_or_else(|| anyhow::anyhow!("no finding `{finding_id}` in the ledger"))
}

/// The `[review]` metadata table, mutably — the home of the pass-level latch,
/// the counter seed and the review-level journal.
pub(crate) fn review_table_mut(
    doc: &mut toml_edit::DocumentMut,
) -> anyhow::Result<&mut toml_edit::Table> {
    doc.get_mut("review")
        .and_then(toml_edit::Item::as_table_mut)
        .ok_or_else(|| anyhow::anyhow!("ledger has no `[review]` table"))
}

/// Write the counter seed (SL-268 sec-2) into `[review]`: the legacy baton's
/// `(rounds, contests)` as `rounds_base`/`contests_base`. The caller writes it in
/// the same edit as the first journalled turn.
pub(crate) fn write_counter_seed(
    doc: &mut toml_edit::DocumentMut,
    (rounds, contests): (u32, u32),
) -> anyhow::Result<()> {
    let meta = review_table_mut(doc)?;
    meta.insert("rounds_base", toml_edit::value(i64::from(rounds)));
    meta.insert("contests_base", toml_edit::value(i64::from(contests)));
    Ok(())
}

/// Clear the pass's concluded marker (SL-268 D2): set `[review].concluded =
/// false` **in place**, and only when it is currently `true`. An absent or
/// already-`false` marker is left untouched, so a never-concluded ledger gains
/// no line. The caller writes it in the same edit as the clearing act's turn.
pub(crate) fn clear_concluded(doc: &mut toml_edit::DocumentMut) -> anyhow::Result<()> {
    let meta = review_table_mut(doc)?;
    if let Some(value) = meta
        .get_mut("concluded")
        .and_then(toml_edit::Item::as_value_mut)
        .filter(|v| v.as_bool() == Some(true))
    {
        let decor = value.decor().clone();
        *value = toml_edit::Value::from(false);
        *value.decor_mut() = decor;
    }
    Ok(())
}

/// The optional account one turn carries (SL-268 sec-2). `note` is the act's
/// own reasoning; `disposition`/`route`/`response` are set on a `dispose` or
/// `amend` turn, where they are also written to the finding as its current
/// answer.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TurnFields<'a> {
    pub(crate) note: Option<&'a str>,
    pub(crate) disposition: Option<&'a str>,
    pub(crate) route: Option<&'a str>,
    pub(crate) response: Option<&'a str>,
}

/// The journal key a turn row is appended under (`[[finding.turn]]` /
/// `[[review.turn]]`).
const TURN_KEY: &str = "turn";

/// Build one turn row in the design's key order (sec-2): `act`, `role`, then
/// `note`, `disposition`, `route`, `response`, each only when present. User
/// free-text rides `toml_edit::value`, which quotes/escapes it.
fn turn_row(act: Act, role: Role, fields: TurnFields<'_>) -> toml_edit::Table {
    let mut row = toml_edit::Table::new();
    row.insert("act", toml_edit::value(act.as_str()));
    row.insert("role", toml_edit::value(role.as_str()));
    for (key, value) in [
        ("note", fields.note),
        ("disposition", fields.disposition),
        ("route", fields.route),
        ("response", fields.response),
    ] {
        if let Some(v) = value {
            row.insert(key, toml_edit::value(v));
        }
    }
    row
}

/// Push a turn row onto `parent`'s journal, creating the journal on first use.
/// A `turn` key that is not an array of tables (a hand-edit) refuses rather than
/// being skipped: an unrecorded turn must not pass for a recorded one (STD-003).
fn push_turn(parent: &mut toml_edit::Table, row: toml_edit::Table) -> anyhow::Result<()> {
    parent
        .entry(TURN_KEY)
        .or_insert_with(|| toml_edit::Item::ArrayOfTables(toml_edit::ArrayOfTables::new()))
        .as_array_of_tables_mut()
        .ok_or_else(|| anyhow::anyhow!("ledger `{TURN_KEY}` is not an array of tables"))?
        .push(row);
    Ok(())
}

/// Apply one finding act (design sec-2 "One write"): set the finding's `status`,
/// any `disposition`/`route`/`response`, and append the act's turn row. This is
/// the **only** writer of a finding's state fields and its journal, so the two
/// move in one edit. Never removes a key (A2: absent means kept). Edit-
/// preserving: the table is mutated in place, so comments / unknown keys /
/// sibling findings survive (the `governance.rs:290` contract at finding
/// scope).
pub(crate) fn apply_act(
    table: &mut toml_edit::Table,
    act: Act,
    role: Role,
    to: FindingStatus,
    fields: TurnFields<'_>,
) -> anyhow::Result<()> {
    table.insert("status", toml_edit::value(to.as_str()));
    if let Some(d) = fields.disposition {
        table.insert("disposition", toml_edit::value(d));
    }
    if let Some(r) = fields.route {
        table.insert("route", toml_edit::value(r));
    }
    if let Some(r) = fields.response {
        table.insert("response", toml_edit::value(r));
    }
    push_turn(table, turn_row(act, role, fields))
}

/// Append a review-level turn (`[[review.turn]]`, design sec-2) — the journal of
/// acts that move no finding. Only `conclude` writes one.
pub(crate) fn append_review_turn(
    doc: &mut toml_edit::DocumentMut,
    act: Act,
    role: Role,
    note: Option<&str>,
) -> anyhow::Result<()> {
    let meta = review_table_mut(doc)?;
    let fields = TurnFields {
        note,
        ..TurnFields::default()
    };
    push_turn(meta, turn_row(act, role, fields))
}

/// Append a fresh `[[finding]]` with id `F-<max+1>` (design §5, append-only —
/// never renumber, never reuse). Raiser-owned fields are fixed here at raise; the
/// status is seeded `open`; the responder pair is absent until a `dispose`. The
/// raise turn is journalled inside the new finding, in the same edit (sec-2).
pub(crate) fn append_finding(
    doc: &mut toml_edit::DocumentMut,
    existing: &[FindingRow],
    severity: Severity,
    title: &str,
    detail: &str,
) -> String {
    let next = next_finding_id(existing);
    let mut row = toml_edit::Table::new();
    row.insert("id", toml_edit::value(&next));
    row.insert("status", toml_edit::value(FindingStatus::Open.as_str()));
    row.insert("severity", toml_edit::value(severity.as_str()));
    row.insert("title", toml_edit::value(title));
    row.insert("detail", toml_edit::value(detail));
    let mut turns = toml_edit::ArrayOfTables::new();
    turns.push(turn_row(Act::Raise, Role::Raiser, TurnFields::default()));
    row.insert(TURN_KEY, toml_edit::Item::ArrayOfTables(turns));
    if let Some(array) = doc
        .entry("finding")
        .or_insert_with(|| toml_edit::Item::ArrayOfTables(toml_edit::ArrayOfTables::new()))
        .as_array_of_tables_mut()
    {
        array.push(row);
    }
    next
}

/// The next append-only finding id: `F-<max+1>` over the existing `F-<n>` ids
/// (design §5). Robust to a gap / a non-conforming id (skipped in the max scan).
pub(crate) fn next_finding_id(existing: &[FindingRow]) -> String {
    let max = existing
        .iter()
        .filter_map(|f| f.id.strip_prefix("F-"))
        .filter_map(|n| n.parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    format!("F-{}", max + 1)
}

/// The current authored status of a finding (for the per-finding `can()` gate),
/// read fail-safe: an out-of-vocabulary status comes back as
/// [`Vocab::Unknown`] for the caller to refuse, never as a guessed `Open` that
/// an act could then move (SL-268 D15).
pub(crate) fn finding_status_of(
    existing: &[FindingRow],
    finding_id: &str,
) -> anyhow::Result<Vocab<FindingStatus>> {
    let row = existing
        .iter()
        .find(|f| f.id == finding_id)
        .ok_or_else(|| anyhow::anyhow!("no finding `{finding_id}` in the ledger"))?;
    Ok(Vocab::read(&row.status))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The refusal's from-set is `can`'s, and only `can`'s (SL-268 sec-4).
    #[test]
    fn admissible_from_is_computed_from_can() {
        use FindingStatus::{Answered, Contested, Open, Verified};
        let cases = [
            (Act::Dispose, vec![Open, Contested]),
            (Act::Amend, vec![Answered]),
            (Act::Verify, vec![Answered]),
            (Act::Contest, vec![Answered]),
            (Act::Reopen, vec![Verified]),
            (Act::Withdraw, vec![Open, Answered]),
            (Act::Conclude, vec![]),
            (Act::Raise, vec![]),
        ];
        for (act, expected) in cases {
            let got = admissible_from(act);
            assert_eq!(got, expected, "{act:?}");
            for status in FINDING_STATUSES
                .iter()
                .filter_map(|s| FindingStatus::parse(s).ok())
            {
                assert_eq!(
                    can(act, Some(status), act.required_role()),
                    got.contains(&status),
                    "{act:?} from {status:?}"
                );
            }
        }
    }
}
