// SPDX-License-Identifier: GPL-3.0-only
//! RV ledger transition table (SL-268 D4, engine tier): the write verbs, the
//! `can` predicate, the per-verb required status, and the finding-scoped
//! edit-preserving `toml_edit` writes that apply a transition.

use super::schema::{FindingRow, parse_finding_status};
use super::vocab::{FindingStatus, Role, Severity};

/// The five write verbs that move a finding's status (design §5). `status` and
/// the read/coordination verbs are not transition verbs and are not modelled
/// here — `can` answers "may this verb fire on a finding in `from` for `role`?".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Verb {
    Raise,
    Dispose,
    Verify,
    Contest,
    Withdraw,
}

impl Verb {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Raise => "raise",
            Self::Dispose => "dispose",
            Self::Verify => "verify",
            Self::Contest => "contest",
            Self::Withdraw => "withdraw",
        }
    }

    /// The role a verb statically requires, knowable without a finding (design
    /// §6 responsibility split): raise/verify/contest/withdraw are the raiser's;
    /// dispose is the responder's. The `with_turn` wrapper checks this; the
    /// per-finding `can` check (state-dependent) is the closure's job.
    pub(crate) const fn required_role(self) -> Role {
        match self {
            Self::Raise | Self::Verify | Self::Contest | Self::Withdraw => Role::Raiser,
            Self::Dispose => Role::Responder,
        }
    }
}

/// What one turn asserts — a finding transition, or the pass-level conclude.
///
/// [`Verb`] stays the **finding-transition** vocabulary: `can`, `required_for` and
/// `gate` are all keyed on a [`FindingStatus`], and concluding has no finding, so
/// a sixth `Verb` variant would force answers into that table that do not exist
/// (there is no status a conclude requires). Splitting here instead keeps the
/// transition table meaning what it says.
///
/// [`with_turn`] needs exactly two things from a turn — the role it statically
/// requires, and whether it is a contest, for the baton's counter. Both live here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TurnAct {
    /// One of the five finding transitions.
    Finding(Verb),
    /// The raiser declares the pass finished (SL-244 `sec-4`, IMP-392).
    Conclude,
}

impl TurnAct {
    /// The role this act statically requires. Concluding is the raiser's — it is
    /// the reviewer saying *I have finished reading*, which is exactly the claim
    /// the design run's `Conducted` arm repeats one layer out — and deliberately
    /// not `dispose`'s, which is per-finding and the responder's.
    pub(crate) const fn required_role(self) -> Role {
        match self {
            Self::Finding(verb) => verb.required_role(),
            Self::Conclude => Role::Raiser,
        }
    }

    /// Whether this turn is a contest, for the baton's observability counter.
    pub(crate) const fn is_contest(self) -> bool {
        matches!(self, Self::Finding(Verb::Contest))
    }

    /// The verb name a refusal reports. A conclude has no [`Verb`], so the
    /// role-mismatch refusal needs its own label rather than a borrowed one.
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Finding(verb) => verb.as_str(),
            Self::Conclude => "conclude",
        }
    }
}

impl From<Verb> for TurnAct {
    fn from(verb: Verb) -> Self {
        Self::Finding(verb)
    }
}

// ---------------------------------------------------------------------------
// Transition predicate (design §5, D-C4/D-C5)
// ---------------------------------------------------------------------------

/// Whether `verb` may fire on a finding currently in `from` (or, for `raise`,
/// not yet existing — `None`) when asserted by `role`. Pure and total: the
/// single-owner edge table (design §5), every other combination refused.
///
/// | verb     | from               | role      | → |
/// |----------|--------------------|-----------|---|
/// | raise    | (none)             | raiser    | open |
/// | dispose  | open \| contested  | responder | answered |
/// | verify   | answered           | raiser    | verified (terminal) |
/// | contest  | answered           | raiser    | contested |
/// | withdraw | open \| answered   | raiser    | withdrawn (terminal) |
pub(crate) const fn can(verb: Verb, from: Option<FindingStatus>, role: Role) -> bool {
    // Static role check first — the half `with_turn` also owns; refuse a
    // role/verb mismatch regardless of state.
    if !role_eq(role, verb.required_role()) {
        return false;
    }
    matches!(
        (verb, from),
        (Verb::Raise, None)
            | (
                Verb::Dispose,
                Some(FindingStatus::Open | FindingStatus::Contested)
            )
            | (Verb::Verify | Verb::Contest, Some(FindingStatus::Answered))
            | (
                Verb::Withdraw,
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

/// Apply a single-owner status transition (design §5): set the finding's
/// `status`, plus any responder-owned `disposition`/`response`. Edit-preserving —
/// the table is mutated in place, so comments / unknown keys / sibling findings
/// survive (the `governance.rs:290` contract at finding scope). User free-text
/// rides `toml_edit::value`, which quotes/escapes it (the structured-write twin of
/// the render path's `toml_string`).
pub(crate) fn apply_transition(
    table: &mut toml_edit::Table,
    new_status: FindingStatus,
    disposition: Option<&str>,
    response: Option<&str>,
) {
    table.insert("status", toml_edit::value(new_status.as_str()));
    if let Some(d) = disposition {
        table.insert("disposition", toml_edit::value(d));
    }
    if let Some(r) = response {
        table.insert("response", toml_edit::value(r));
    }
}

/// Append a fresh `[[finding]]` with id `F-<max+1>` (design §5, append-only —
/// never renumber, never reuse). Raiser-owned fields are fixed here at raise; the
/// status is seeded `open`; the responder pair is absent until a `dispose`.
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

/// The current authored status of a finding (for the per-finding `can()` gate).
pub(crate) fn finding_status_of(
    existing: &[FindingRow],
    finding_id: &str,
) -> anyhow::Result<FindingStatus> {
    let row = existing
        .iter()
        .find(|f| f.id == finding_id)
        .ok_or_else(|| anyhow::anyhow!("no finding `{finding_id}` in the ledger"))?;
    Ok(parse_finding_status(&row.status))
}

/// The canonical required status for each verb — the state a finding must be
/// in for the verb to act on it. Compound cases pick the first valid status.
pub(crate) fn required_for(verb: Verb) -> FindingStatus {
    match verb {
        Verb::Dispose | Verb::Withdraw | Verb::Raise => FindingStatus::Open,
        Verb::Verify | Verb::Contest => FindingStatus::Answered,
    }
}
