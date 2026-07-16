# Implementation Plan SL-220: Ledgered value claims

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Seven phases realise design.md (D1–D14, §1–§8): evidence first, then four
worker code phases strictly additive until the REV-gated flip, then rendering,
then the operator-run migration. The design's ten verification suites (§8)
map onto the phases as: §8.9 → PHASE-01/07 (operational, VA/VH — the scripts
are throwaway and deliberately not unit-tested); §8.1/§8.2 → PHASE-02;
§8.3 → PHASE-03; §8.6 → PHASE-04; §8.4/§8.5 → PHASE-05; §8.7/§8.8 → PHASE-06;
§8.10 (VA, T2 invariant + approval-gate audit) rides PHASE-05 VA rows and
recurs at /audit.

## Sequencing & Rationale

**The SL-219 landing gate (PHASE-02 EN-1).** The design's substrate — the
`DomainSystem` split, estimate domain, v2 goldens — is NOT on trunk at plan
time: SL-219 sits at `reconcile` with its implementation bundle unintegrated
on `candidate/219/review-001` (merge-base `c3a21b17`). That bundle touches
every file SL-220's code phases touch (wire, resolve, store, compile-adjacent,
graph, elicit, compare, config, render, surface, view). Forking SL-220 code
phases from a trunk without it would build against the wrong shape and
guarantee integration conflicts. PHASE-01 (Python scripts, file-disjoint) is
exempt and can run immediately; every Rust phase requires SL-219 closed and
landed on main first. This is the plan's one external dependency.

**Additivity boundary (D12), made executable.** Earlier phases must change no
resolution outcome. The subtle seam is PHASE-03: the claims pass is computed
and carried through `Pipeline`, and compile input is adapted to `PairRow`
views — but compile's value-anchor *source* stays the facet builder
(`comparison_anchor_map`) until PHASE-05. Swapping the source earlier would
silently de-anchor facet-bearing corpora before the REV approves the policy:
with no claim rows in existence pre-verbs/pre-migration, `claims.anchor_map()`
is empty, and feeding it to compile IS the flip. The swap, the builder
deletion, and the `effective_raw_value` rewire therefore land together in
PHASE-05 behind the REV gate. PHASE-04's verb re-plumb is additive in the
D12 sense (no resolution outcome changes — minted rows resolve but nothing
consumes them until the flip), with one honest interregnum caveat: between
PHASE-04 and PHASE-05 landing, `value set` mints rows the resolver does not
yet read. Phases land in sequence on main within one dispatch; the window is
internal to the slice, disclosed here, and closed before /audit.

**Verbs before flip.** PHASE-04 precedes PHASE-05 so the re-assertion surface
(`value set --rater human`, `value pin`) exists the moment facet authority is
demoted — the §3 "operator re-assertion prompt" story requires the verbs to
be live when the finding starts firing.

**Render after flip.** PHASE-05 carries the minimal view motion the flip
cannot compile without (ReasonKind variants, the D11 `value_source` token
change, the vocabulary golden); PHASE-06 carries the full human-facing
surface (§6 shapes, row cells, `show` re-sourcing, elicit fragments,
findings render, disclosure, grep-gate). Both land before the migration runs,
so the operator sees honest provenance when reviewing the census. D11's
breaking JSON change and the flip ride the same release by construction —
same dispatch, adjacent phases.

**Operator phases.** PHASE-01's EX-4 (baseline evidence) and all of PHASE-07
are operator/orchestrator actions in the primary tree — workers never write
`.doctrine`. PHASE-07 is deliberately a plan phase rather than an audit
footnote: the census is a hard exit criterion (design § Verification), the
migration must run after the claims machinery ships and before /audit (§5
sequencing pin), and the post-flip regression diff is the R2 evidence
contract. Dispatch drives PHASE-02..06 through workers; PHASE-01 splits
(worker authors scripts, operator captures baseline); PHASE-07 has no worker
at all.

**Governance (D12, §7).** REV-024 is minted at plan time (this commit) against
ADR-015 (primary), SPEC-020 + its value-surface requirements REQ-278/279/280/
286, and PRD-014 — the REV-022/023 pattern, `originates_from` RFC-020.
Approval of the REV and the SPEC-020 normative amendments is PHASE-05's EN-2;
application (the actual text edits, surfaced-for-manual by `revision apply`)
is PHASE-07's EX-5, so canon is amended before the corpus goes live under the
new semantics. Additive/documentary spec work (claim-schema retention REQs,
PRD-011/SPEC-001 descent prose) remains a reconciliation obligation per D12
and is out of this plan.

## Notes

- **Open items pinned to /phase-plan (from the design, confirmed still open):**
  - PHASE-04 EN-2: exact WriteClass variant for `value pin`/`--retire`. At
    plan time `src/commands/guard.rs` carries `Read / Write / Orchestrator /
    MarkerClear / Hookmint`, with `Command::Value { .. } => Write("value")`.
    The pin verbs need per-subcommand routing out of the blanket `Value`
    arm; whether they join `Orchestrator` or mint a variant is D13
    adjudication material at phase-plan — the design pins only "worker-refused
    class + TTY gate".
  - `value::validate` semantics verified at plan time: `src/value.rs`
    accepts any finite f64 (negatives included), rejects NaN/±Inf — §1's
    payload check mirrors it exactly, no range policy.
- VT mandate keywords name design-pinned symbols (`FRAME_VALUE_ANCHOR`,
  `ordering_date`, `pin_outranks_all_tiers_under_derived_ord`, `PairRow`,
  `UnmigratedFacet`, D11 tokens). Workers own test names; mandates bind the
  load-bearing identifiers, not the test list.
- PHASE-02..06 are file-overlapping by design (wire → claims → verbs → flip →
  render each build on the prior); dispatch them serially, one worker per
  phase, no parallelisation.
- The empty-`b` degenerate `IdentityKey` (PHASE-02) keeps BTreeMap ordering
  valid via `Option<String>: Ord` — flagged so nobody "simplifies" it to a
  sentinel string, which would collide with a legal id.
