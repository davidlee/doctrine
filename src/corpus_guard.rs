// SPDX-License-Identifier: GPL-3.0-only
//! `corpus_guard` — the dispatch corpus-loss guards (SL-166, ISS-056).
//!
//! Three mechanism-level guards refuse the doctrine-verb-mediated paths by which
//! a `/dispatch` drive can silently delete or revert the authored `.doctrine`
//! corpus (design §5):
//!
//! - **g1** — refuse trunk-mutating verbs while HEAD sits on the integration
//!   buffer ([`REFUSE_ON_TRUNK`]).
//! - **g2** — refuse `coordinate()` setup when the fork base predates the corpus
//!   ([`BASE_CORPUS_STALE`]).
//! - **g3** — refuse a per-leg advance that clobbers authored [`DOCTRINE_PATHSPEC`]
//!   paths ([`CORPUS_CLOBBER`]).
//!
//! PHASE-01 lands the shared substrate: the named refusal tokens (STD-001) and
//! the corpus pathspec. The pure guard predicates grow here in PHASE-02/04; g2
//! extends `worktree::coordinate` in PHASE-03. Leaf tier (ADR-001): constants
//! only today, and any predicate stays pure over injected git readings — git I/O
//! lives in `git.rs`, which receives the pathspec as a parameter (never imports
//! this module, so no cycle).

/// Refusal prefix for g1 — a trunk-mutating verb invoked while HEAD is on the
/// integration buffer (`deliver_to`). The buffer ref and recovery are
/// interpolated at the call site (SL-166 design §5.2). Consumed in PHASE-04.
#[expect(dead_code, reason = "consumed by SL-166 PHASE-04 (g1)")]
pub(crate) const REFUSE_ON_TRUNK: &str = "refused: HEAD is on the integration buffer";

/// Refusal prefix for g2 — a `dispatch setup` whose fork base predates the
/// authored `.doctrine` corpus the authoring branch holds (SL-166 design §5.2).
/// Consumed in PHASE-03.
#[expect(dead_code, reason = "consumed by SL-166 PHASE-03 (g2)")]
pub(crate) const BASE_CORPUS_STALE: &str =
    "refused: fork base predates the authored .doctrine corpus";

/// Refusal prefix for g3 — a funnel advance that would delete or revert authored
/// `.doctrine` paths the target ref holds, unallowed (SL-166 design §5.2).
/// Consumed in PHASE-02.
#[expect(dead_code, reason = "consumed by SL-166 PHASE-02 (g3)")]
pub(crate) const CORPUS_CLOBBER: &str = "refused: advance would clobber authored .doctrine paths";

/// The authored-corpus pathspec — the directory g2/g3 reason over and the
/// projection strips (`dispatch.rs` `filter_tree`). Single-source (STD-001).
pub(crate) const DOCTRINE_PATHSPEC: &str = ".doctrine";
