// SPDX-License-Identifier: GPL-3.0-only
//! The RV adversarial-review ledger, engine tier (SL-268 D4; SL-040, ADR-007):
//! the closed vocabularies, the authored `review-NNN.toml` schema and readers,
//! the derived status, the verb transition table, and the blocker / pass-fact
//! predicates other kinds gate on. It reaches only leaf and engine modules; the
//! `doctrine review` command surface (verbs, baton, cache, rendering) is the
//! command-tier `review` module, which builds on this one.

mod derive;
mod gate;
mod schema;
mod transition;
mod vocab;

pub(crate) use derive::{
    FindingState, VocabDefect, derived_status, finding_states_of, vocabulary_defects,
};
pub(crate) use gate::{
    derived_status_string, observe_pass, read_pass_facts, relation_edges, unresolved_blockers_for,
};
pub(crate) use schema::{
    FindingRow, ReviewDoc, ReviewMeta, Target, authored_path, canonical_id, parse_ref,
    read_authored, read_review, read_reviews,
};
pub(crate) use transition::{
    TurnAct, Verb, append_finding, apply_transition, can, finding_status_of, finding_table_mut,
    required_for,
};
pub(crate) use vocab::{
    Await, FINDING_STATUSES, Facet, FindingStatus, REVIEW_STATUSES, ReviewStatus, Role, Severity,
    Vocab,
};

// Reached only by unit tests (review, slice, mcp_server, priority).
#[cfg(test)]
pub(crate) use derive::{EFFECT_UNKNOWN_SEVERITY, VocabField};
#[cfg(test)]
pub(crate) use gate::{
    BlockerRef, OutstandingCounts, gates_as_blocker, outstanding_by_severity, undisposed_blockers,
};
#[cfg(test)]
pub(crate) use transition::next_finding_id;
#[cfg(test)]
pub(crate) use vocab::{FACETS, ROLES, SEVERITIES};
