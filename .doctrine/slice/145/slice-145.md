# Backlog relation source parity

## Context

RFC-003 triaged CHR-024's holistic relation-model review into four axes. **Axis A —
"backlog second-class"** is the named easy win: it is independent of the RFC's core
(the `references` + role grammar, Axis B) and ships now. CHR-024 findings F-1/F-2/F-4/F-5
all reduce to one defect: **BACKLOG kinds (ISS/IMP/CHR/RSK/IDE) are excluded from the
source-set of reference labels they legitimately need**, forcing the intent into prose
(governance-in-prose, review-in-prose) instead of a structured edge.

Concretely, in `RELATION_RULES` (`src/relation.rs`) today:

- `governed_by` sources `[SL, PRD, SPEC, CM, ASM, DEC, QUE, CON]` — **no BACKLOG**. A
  chore or improvement governed by an ADR/POL/STD cannot say so structurally
  (`validate_link` returns `IllegalForSource`).
- `related` has two rows — `GOV` (SameKind) and `[SL, RFC]` (AnyNumbered) — **no
  BACKLOG**. A backlog item cannot author a peer reference.
- `reviews` is `RV`-only, `TypedVerbOnly` — a non-RV reviewer outlet for a backlog item
  has no home (F-5).

This is a **source-set extension**, not a vocabulary redesign. It rides the existing
`sources: SET` machinery (ADR-010 D2; one rule serves many source kinds) and the
existing outbound-only model (ADR-004) — purely widening which kinds may author edges
that already exist.

## Scope & Objectives

In scope:

- Add `BACKLOG` to the `sources` set of **`governed_by`** so ISS/IMP/CHR/RSK/IDE may be
  governed by ADR/POL/STD (target gate unchanged: `Kinds(GOV)`).
- Make **`related`** authorable by BACKLOG sources (extend the `[SL, RFC]` AnyNumbered
  row's source set, or add a BACKLOG row — design's call; storage/target semantics
  unchanged).
- Update the lockstep vocabulary tests that pin `sources` against shipped accessors
  (`sources_match_shipped_accessors`, the VT-1/VT-2 family) to the widened sets, and the
  `IllegalForSource` goldens that currently assert a backlog item is refused.
- Verify the per-kind legality `read_block` (X2) and `validate_link` surface the widened
  legality (a backlog item authoring `governed_by`/`related` now *resolves*).
- **CLI end-to-end** — the widened sets must work through the actual verb surface, not
  just the rule table: `doctrine link <BACKLOG-id> governed_by|related <target>` authors
  → validates → persists a `[[relation]]` row in the backlog item's TOML → reads back on
  `backlog show` / `inspect` (outbound) → renders the derived inbound reciprocal on the
  target's `inspect`. `unlink` round-trips. Confirm no backlog-authoring path (e.g.
  `backlog new`/`relate`, if any) bypasses or duplicates the generic `link` seam.

Objective: a backlog item can structurally express "governed by this ADR" and "related
to that entity" through `doctrine link`, with the validator accepting it and `inspect`
rendering the derived inbound reciprocal.

## Non-Goals

- **The `references` + role grammar (Axis B).** No `references` label, no role enum, no
  `(source, label, role) → TargetSpec` retargeting. This slice does not touch label
  identity or the intent dimension — only source legality.
- **Coverage / close-gate (Axis C)** and **decomposition / `part_of` / altitude
  (Axis D).** Sibling work, sequenced separately.
- **Non-entity-target edges** (backlog→memory/file/glob — `drift` abuse, IMP-012,
  IDE-015). Out.
- **No migration of existing edges** — this only *permits* new sources; it rewrites no
  stored row.

## Affected Surface

- `src/relation.rs` — `RELATION_RULES` rows for `governed_by` and `related`; the
  `sources_match_shipped_accessors` / `IllegalForSource` test goldens.
- `src/relation.rs` `validate_link` / `read_block` legality paths (read-through; likely
  no code change, only widened acceptance + tests).
- Possibly `inspect` / `relation list` goldens if any pin the refused-source behaviour.

## Risks / Assumptions / Open Questions

- **Assumption:** widening a `sources` set is behaviour-preserving for every *existing*
  edge (the behaviour-preservation gate — existing suites stay green unchanged except
  the goldens that asserted the refusal). Confirm no overlay/accessor assumes BACKLOG is
  absent.
- **OQ-1 (review outlet, F-5) — RESOLVED (design D2):** defer the review outlet to Axis B.
  `reviews` is `RV`-only `TypedVerbOnly`; a non-RV backlog reviewer edge has no clean home
  without B's role grammar (which reserves `references(reviews)` for it). This slice ships
  `governed_by` + `related` only. F-5 stays open against B.
- **OQ-2 — RESOLVED (design D1):** extend the existing `related` `[SL, RFC]` AnyNumbered
  row to include BACKLOG (not a separate row). VT-1 enum-order unaffected (no new label).
- **OQ-3 — RESOLVED (design D3):** permit the edge only; no consumer reaction. RFC-003
  Layer-1 — graph-effect is a consumer decision, out of scope here.

## Verification / Closure Intent

- A backlog item (e.g. `CHR-NNN`) `governed_by` an ADR and `related` to any numbered
  entity both **resolve** through `doctrine link` (no `IllegalForSource`), persist to the
  source TOML, and survive an `unlink` round-trip.
- `doctrine inspect` / `backlog show` on both source and target render the outbound +
  derived inbound edge — the full author→persist→read loop, exercised through the CLI.
- The vocabulary lockstep tests (VT-1 enum order, VT-2 `sources_match_shipped_accessors`)
  pass against the widened sets; the former refusal goldens are updated to assert
  acceptance.
- Whole existing suite green unchanged (behaviour-preservation gate); `just gate` clean.

## Follow-Ups

- Axis B (`references` + role grammar) absorbs the review outlet (OQ-1) and the
  intent/role dimension — gated on the RFC's ratifying ADR.
- Prose-hunt pass for governance-in-prose / review-in-prose now expressible as edges
  (RFC "Open / deferred").
