# Implementation Plan SL-051: retire backlog order; fold ordering into list as default-on comparator

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two phases, split along the natural seam the green-gate forces: an in-crate
behaviour change (PHASE-01) then a black-box surface pin (PHASE-02). The design
(SL-051 §4–5) introduces no new graph mechanism — the `cordage` `BacklogOrder`
adapter and the `project`/`render_overrides` shell helpers are reused as-is; only
their call site moves from the standalone `order` verb into the default `list`
path as a pure comparator. The work is therefore a re-seat plus a verb deletion,
not new machinery, which keeps both phases small.

## Sequencing & Rationale

**Why two phases, and why this boundary.** The whole fold is one cohesive
behaviour change, but the root test suite cannot stay green across it in a single
sweep unless the in-crate change and the e2e goldens are separated. The old
`tests/e2e_backlog_order_golden.rs` is a root-package test (so `just check` runs
it) that drives `backlog order` and asserts the line-167 hard-error. The moment
PHASE-01 deletes the verb, that file fails to even spawn its subcommand. So
PHASE-01 must *remove* the orphaned golden to end green, and PHASE-02 authors its
replacement (`e2e_backlog_list_order_golden.rs`) against the now-shipped surface.
This is the design's "migrate + rename" (§5) realised across the phase boundary:
delete-then-reauthor, not edit-in-place. The transient window where ordering has
no e2e coverage is internal to the slice; the end state restores full coverage
plus the new default-on / opt-out / degrade goldens.

**PHASE-01 — the in-crate fold (red/green/refactor).** Driven by the in-crate
`list_rows` unit tests, which already exercise the survey spine. The tuple return
(`(stdout, stderr)`, §4.3) churns ~16 of them mechanically (destructure `(out, _)`,
assert on `.0`); the 4 `order_rows` tests are deleted with the verb they covered.
New unit coverage drives the genuinely new behaviour: the composed-sequence sort,
the `--by id` opt-out, the membership-set equality between them (A-2 at unit
level), and `compose`'s degrade contract on a `needs` cycle. The `main.rs` surgery
is mechanical but has a sharp edge the design calls out: there are **three** live
`Order` references, not two — the clap variant (887), the dispatch arm (2222), and
the access-classify arm (1685). Missing the classifier arm fails to compile on the
unknown variant, so it is an explicit exit criterion. The refactor beat is the
re-homing of `project`/`render_overrides`/`name_cycle`/`AbsentDrop` under `compose`
with no logic change — they survive the verb's deletion because `compose` still
needs them for the diagnostic.

**PHASE-02 — the surface pin.** Pure test authoring against the shipped surface,
following the repo's black-box-golden pattern (capture actual, verify it matches
the design's stated behaviour, pin byte-exact). The eight goldens map one-to-one
to the §6 VTs. The cycle-degrade golden is the deliberate inversion of the old
behaviour: where `order` hard-errored on a cyclic graph, default `list` now stays
total — id-sorted rows on stdout, the advisory on stderr, exit 0. That is the
single most important behavioural assertion in the slice and the reason the verb
could not simply be aliased.

## Notes

- **Why impl-before-golden (not strict red-first for PHASE-02).** The goldens pin
  a byte-exact CLI surface that must exist to be captured; this is the established
  doctrine e2e pattern, not a TDD lapse. The red/green discipline lives in
  PHASE-01's unit tests, which *do* drive the behaviour into existence.
- **Layering hold (DD-2, ADR-001).** The comparator stays in `backlog.rs` because
  it needs backlog-domain types (`BacklogOrder`/`project`/`ItemId`); `listing.rs`
  and `backlog_order.rs` are read-only across both phases. The `--by` opt-out rides
  the backlog `List` variant, never the shared `CommonListArgs`/`ListArgs`.
- **Out of scope, flagged at close, not here:** RSK-005 (adapter bimap corruption)
  and the PRD-009/`REQ-097` reconcile note (the capability is satisfied by `list`;
  the verb name was never the binding) — both are closeout reconcile items, not
  plan phases.
