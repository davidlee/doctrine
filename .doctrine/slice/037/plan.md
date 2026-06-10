# Implementation Plan SL-037: Shared list column model: --columns projection + slug-free defaults

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases lift the per-kind list column projection into a shared model on the
`listing.rs` leaf, expose it as `--columns`, ship slug-free defaults, and pin the
churn with a cross-verb golden net. The shape is **foundation-and-proof → fan-out
→ net**: build the seam and prove it on the easiest kind first, migrate the
remaining kinds against a now-load-bearing model, then land the regression net
last over the final cross-verb surface.

## Sequencing & Rationale

**Why PHASE-01 bundles the model, the API, governance, and the memory guard.**
The riskiest part of this slice is not any single migration — it is whether the
`Column<R>` + non-capturing-extractor seam (D5) actually carries an end-to-end
`--columns` request without re-fracturing the spine (R1, the IMP-013 warning).
So the first phase proves the whole path on one kind rather than building dead
machinery. Governance is chosen as the prover: its `GovRow` is already all-String
with a pre-prefixed id, so it satisfies D5 with no row-type work and exercises the
shared adr/policy/standard render path. Bundling a live consumer also avoids a
dead-code suppression dance in the leaf. The `--columns` flag is born here too —
and the moment it rides the shared `CommonListArgs` it reaches `memory list`, so
the D9/R4 rejection guard **must** land in the same phase, or the flag is a silent
no-op on memory the day it ships (the footgun CHARGE III closed). `build()`'s
signature stays frozen (D6): the verb does `args.columns.take()` before `build`,
so the ~10 in-leaf tests stay green unchanged — the behaviour-preservation proof.

**Why backlog + slice share PHASE-02.** Both keep a title column and migrate
without a new row type: backlog reads its fields off `BacklogItem`; slice reuses
the existing `(Meta, Option<PhaseRollup>)` tuple and its `canonical_id` /
`decorated_status` / `phases_cell` helpers as non-capturing extractors. slice's
`?`/`⚠` drift markers and its phases cell become ordinary column *values*, not
config — the direct test that R1's "markers force hidden config" fear is unfounded.
They are paired because they are the two low-risk same-shape migrations; grouping
them keeps the high-risk spec migration isolated.

**Why spec is alone in PHASE-03.** Spec is the CHARGE I kind: its prefixed id is
subtype-dependent (`Product → PRD`, `Tech → SPEC`), so the subtype is external
context D5 forbids capturing in an extractor. The remedy is a pre-materialised
`SpecListRow` built per labelled block where the subtype is in scope — the same
`GovRow` pattern, resolving the id *before* extraction. It also owns the only
multi-block table layout and the slug→title default swap (D4). Isolating it keeps
the structurally-distinct, highest-risk migration from contaminating the simpler
ones, and lets its multi-block + omitted-empty-block behaviour (R3) be verified on
its own. It depends only on PHASE-01's model, not on PHASE-02.

**Why the golden harness is last (PHASE-04), not first.** The intentional churn
here is the table surface — slug drops out of every default. Pinning the old
with-slug tables first only to rewrite the goldens four times is busywork; the
behaviour-preservation gate that genuinely must not move (JSON + filter) is
already guarded by the pre-existing suites, which D2 keeps green unchanged. So the
per-phase unit tests carry the TDD load *during* migration, and the cross-verb
black-box golden net lands once over the final surface — the regression net IMP-014
asks for. It also carries the CHARGE IV coverage the inquisition demanded: memory
rejection, empty-list per verb, spec multi-block, governance breadth — asserting
every surface, not just the JSON envelope.

## Notes

- **JSON is deliberately not lifted** (D2 / IMP-013 descope). Every phase leaves
  the typed `*Row` structs and `json_rows` mappers untouched; `--columns` is a
  table projection only. IMP-013's `/close` resolution records this descope rather
  than claiming full delivery.
- **Memory stays bespoke this slice** (D9). Its migration is the IMP-017 follow-up,
  deferred-until-condition; PHASE-01 only adds the loud guard, which IMP-017 later
  removes.
- PHASE-02 and PHASE-03 both depend only on PHASE-01 and are mutually independent;
  the 02-before-03 order is by ascending risk, not a hard dependency.
