# Default value for valueless value-bearing kinds

## Context

SL-176's `fulfils` priority **value-burndown** reduces a backlog item's value by
the lifecycle-gated **raw `value` facet** of the slices fulfilling it. The
subtraction is **value-denominated** — it signals nothing for entities with no
authored value: they contribute `raw_value = 0`, so `r(I) = 0` and `delivered`
gains nothing. A valueless item is invisible to the undelivered-value signal, and
a valueless fulfilling slice burns nothing down.

SL-176 (D-value-floor-sibling, user-locked 2026-06-29) defers the fix here: give
value-bearing kinds a **default value of 1.0** when none is authored.

**Dependency (RV-191 F-1): SL-177 `needs` SL-176.** The fix is a single shared
`effective_raw_value` accessor consumed by *both* `base_score` and SL-176's
burndown `raw_value` seam — and that seam must exist first. SL-177 lands second;
interim, valueless items simply don't burn down (the posture SL-176 accepts).

## Scope & Objectives

1. **Shared `effective_raw_value(kind, &facets)` seam** in the priority tier — the
   single definition of an entity's value for priority purposes. Authored value
   wins; a value-bearing kind with no facet defaults to `DEFAULT_VALUE = 1.0`; any
   other valueless kind is `None`. Applied at the scoring seam, never by mutating
   authored TOML.
2. **`base_score`'s `value_dim`** consumes the seam (replaces its raw `None→0`
   branch).
3. **SL-176's burndown `raw_value`** is retrofitted to consume the same seam — the
   one change that makes the default reach burndown.
4. **`kinds::VALUE_BEARING` + `is_value_bearing`** ({slice, backlog}); promote and
   rename `surface.rs`'s function-local `WORK_PREFIXES`. **Not** `WORK` — that
   collides with `dep_seq::is_work_like` ({slice,backlog}∪REV). Records, governance,
   and **REV** excluded.

## Non-Goals

- **The burndown post-pass itself** — owned by SL-176; this slice retrofits only
  its `raw_value` read.
- **`fulfils` coverage-% derived display** — deferred follow-up in SL-176's ledger.
- **Render authored-vs-effective value signalling** — RV-191 F-5 → **IMP-211**.
- **Estimate/cost defaults** — only the *value* default is in scope (SL-172 owns
  cost/estimate anchoring).
- **Config tunability** — OQ-1 hard constant; tunable is a clean later swap.
- **Governance ratification** — rides reconciliation if needed (mirrors SL-176).

## Affected surface

- `src/kinds.rs` — add `VALUE_BEARING` + `is_value_bearing` + canary test.
- `src/priority/graph.rs` — the `effective_raw_value` seam + `DEFAULT_VALUE` const;
  `base_score` value-dim consumes it; SL-176 burndown `raw_value` retrofit.
- `src/priority/surface.rs` — drop local `WORK_PREFIXES`, consume `is_value_bearing`.
- (`src/value.rs` **untouched** — stays authored-facet-pure; RV-191 F-4.)

## Risks / Assumptions / Open questions

- **OQ-1** *(resolved — hard constant)* `DEFAULT_VALUE = 1.0`; tunability a later
  `cfg`-field swap, no seam change.
- **OQ-2** *(resolved — `VALUE_BEARING`)* Promote+rename the existing set
  (`surface.rs` `WORK_PREFIXES`, SL-089 D2); NOT `WORK` (RV-191 F-3).
- **OQ-3** *(resolved — shared seam)* One `effective_raw_value` feeds both
  `base_score` and burndown (RV-191 F-1).
- **A-1** Default at the scoring seam, not authored TOML — storage honest.
- **R-2** *(sequencing, hard)* SL-177 `needs` SL-176: the `raw_value` site must
  exist to retrofit.
- **R-4** *Standalone ordering change.* Valueless work items gain a baseline
  `value_dim`; **bounded** — unvalued+unestimated items take the absent-estimate
  anchor (> largest real estimate) as cost, so `value_dim ≈ 1.0/large` is small.
  Intended; user-acked.

## Verification / closure intent

- A valueless slice/backlog item scores `value_dim` as if `value = 1.0`.
- A backlog item fulfilled by a valueless delivering-status slice → `delivered`
  reflects `1.0`, `r(I) > 0` (the F-1 regression guard).
- A valueless record / REV is **unaffected** (`value_dim 0`, excluded from burndown).
- Authored value (incl. `0.0`, `< 1.0`) **unchanged** (no-clamp).
- Behaviour-preservation scoped to unrelated behaviour; valueless-work fixtures
  re-baselined by design (full blast radius enumerated in design §9.1).

## Follow-Ups

- **IMP-211** — signal effective-vs-authored value in `next`/`survey` (RV-191 F-5).
- `fulfils` coverage-% derived display (deferred from SL-176).
