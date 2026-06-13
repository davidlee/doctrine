# Structural cross-corpus relation edges: governance seam + spec-ADR + product-product

## Context

Slice **3 of 3** in the graph-relations work, and the realisation of **IMP-016**
(and **IMP-035**, the slice↔ADR slot): cross-corpus relations are **prose-only**
today. The capture surface is uneven —
slices have `specs`/`requirements`/`supersedes`; specs have `descends_from` +
members; backlog has `specs`/`slices`/`needs`/`after`/`drift`. But:
- governance kinds (POL/STD/ADR) carry a `[relationships]` block
  (`supersedes`/`superseded_by`/`related`/`tags`) that is **parsed but inert —
  never queried** (`src/governance.rs`);
- there is **no structural spec↔ADR edge** (a spec citing a governing ADR is
  prose);
- there is **no structural slice↔ADR edge** — a slice's `[relationships]` block is
  `{specs, requirements, supersedes}` with **no governance slot**, so a slice
  governed by an ADR can only cite it in prose (**IMP-035**, surfaced by the
  SL-046 ↔ ADR-010 interrogation);
- there is **no product↔product edge** (PRD-to-PRD links are prose).

SL-046 makes the graph read **all existing authored relations** (including the
inert governance block, read-only). This slice **mints the missing authored
edges** so the connective tissue is structured, queryable, and feeds the graph —
once a live reader (SL-046) already exists, so new edges are not born inert.

**Sequenced last on purpose.** Authoring capture before a reader recreates the
exact inert-seam bug this work exposed. SL-046 (read) + SL-047 (rank) must exist
first.

## Scope & Objectives

Implements **ADR-010 (accepted)** — the relation contract. Design locked in
`design.md`; objectives below reflect the resolved decisions (OD-1/2/3).

1. **The relation contract, code-authoritative.** A `RELATION_RULES` legal-set
   table (`src/relation.rs`) over `integrity::KINDS` — `(sources, label,
   inbound_name, TargetSpec, Tier, LinkPolicy)` — the single source driving
   writer, validator, the SL-046 reader, **and** cordage overlay allocation, with
   an exact-coverage invariant test.
2. **A uniform `link` / `unlink` verb** (`doctrine link <SRC> <LABEL> <TARGET>`)
   over a generic `append_edge`/`remove_edge` seam, gated by the table; governance
   relations become *authorable*, not hand-edited inert TOML.
3. **The missing edges, as labels:** `governed_by` (SL·SPEC·PRD → ADR·POL·STD —
   IMP-035 slice↔ADR + spec↔ADR); `consumes` (PRD→PRD seam consumption, OD-1).
4. **Uniform `[[relation]]` tier-1 storage, corpus-wide** — migrate every tier-1
   `[relationships]` block (slice, governance `related`, backlog `{slices, specs,
   drift}`) to the generic `[[relation]] label=… target=…` shape; tier-2/3 stay
   typed. Governance supersession (`supersedes`+`superseded_by`) **excluded**
   (OD-3 — its transactional verb is IMP-006). One-shot **deterministic** in-repo
   migrator (unshipped) gated by before/after black-box goldens (OD-2).
5. **Write-strict forward-edge validation** through `ensure_ref_resolves` (refuse
   creating a dangling numbered-kind target); read-tolerant `validate` (report
   danglers + illegal hand-edited rows + the supersession cross-check; never
   rewrite). Free-text targets (`drift`) carry unvalidated. ADR-004 holds:
   **outbound only**, reciprocity derived.
6. **The relation contract tech spec** (PHASE-01) — durable mechanism doc;
   SPEC-005/006/016 reference it. ADR-010 amended (D3 taken corpus-wide).

## Non-Goals

- **No reverse/inbound storage** — inbound stays derived (ADR-004; SL-046).
- **No graph/ranking build** — SL-046 reads these edges; SL-047 ranks. This slice
  is capture-surface + storage shape only.
- **No `cordage` change** (overlay *allocation* is table-driven in the adapter; the
  core crate is untouched).
- **No transactional supersede verb** — governance supersession stays typed; the
  verb is IMP-006 (a gov-only build here = parallel implementation).
- **No tier-2/3 re-modelling** — lineage (arity≤1), `interactions`, `members`,
  backlog `needs`/`after`/`triggers`, review/rec edges keep their typed guarantees.

> **Scope reversal (was a non-goal):** the original "additive only — existing
> relations untouched" non-goal is **dropped**. The user prioritised a correct,
> consistent final shape over minimal churn; tier-1 storage migrates corpus-wide.

## Affected Surface

- `src/relation.rs` — `RELATION_RULES` legal-set table + new `RelationLabel`
  variants (`governed_by`, `consumes`); generic `append_edge`/`remove_edge`/
  `read_block` seam.
- `src/relation_graph.rs` — table-driven overlay allocation; `read_block`-based
  reading; inbound rendering via `inbound_name`.
- `src/{slice,spec,governance,backlog}.rs` — tier-1 `[relationships]` → `[[relation]]`
  migration; `relation_edges` accessors shrink to shared `read_block` + their
  typed tier-2/3 edges.
- `src/main.rs` — `link` / `unlink` verbs.
- `src/integrity.rs` — forward-edge validation + `IllegalRow` + supersession
  cross-check (extend the existing dangling-citation logic, do not duplicate).
- New tech spec (relation contract); SPEC-005/006/016 reference it.
- One-shot deterministic corpus migrator (in-repo, unshipped) + before/after
  goldens; ADR-010 amendment.

## Risks, Assumptions, Open Questions

Prerequisite — **RESOLVED.** The relation-governance decision is **ADR-010
(accepted)**: the contract, tier partition, code-authoritative vocabulary,
write-accessor seam, outbound-only. SL-046 (read) + SL-047 (rank) are **done**.
The original open questions are resolved: verb shape → uniform `link` (ADR-010
D2); validation strictness → write-strict / read-tolerant (OD-2/§5.5);
reciprocal-meaningful labels → `inbound_name` per rule (X5).

Risks (carried into `/plan`; detail in `design.md` §9):
- **R1 (migration correctness)** — the deterministic migrator mutates committed
  authored TOML; gated by whole-corpus before/after goldens + git-reversible.
- **R2 (emit order)** — `read_block` must emit **axis-major, storage-independent**
  (canonical label order), or the SL-046 goldens break; the tier-1/tier-2 merge
  order is pinned per kind.
- **R3 (read_block legality)** — generic parser must enforce per-kind
  `(source,label)` legality (illegal hand-edited rows → validation findings, not
  live edges), preserving the guarantee the hardcoded readers had.
- **C3 (cross-check fires on existing corpus)** — pre-existing hand-authored
  `superseded_by` drift will be reported (intended).

Assumptions:
- Canonical id is the stable cross-kind target key.
- doctrine is dogfood-only — no client back-compat, hence the hard parser cutover.

## Verification / Closure Intent

- A `governed_by` / `consumes` / governance `related` edge authored via `link` is
  validated, persisted as `[[relation]]`, surfaced by `show` + the SL-046 query,
  and appears in the target's **derived inbound** view (ADR-004).
- `link` refuses an illegal `(source,label,target-kind)` triple and a dangling
  numbered target; `unlink` round-trips; free-text targets (`drift`) pass.
- `RELATION_RULES` exact-coverage invariant holds (reader/overlay/validator cannot
  diverge); inbound renders via `inbound_name`.
- **Behaviour preservation:** `backlog order` byte-identical; SL-046 reader emits
  the same edges post-migration; existing suites green; whole-corpus round-trip
  `show`/`show --json`/`inspect` goldens identical across the migration.

## Follow-Ups

- **IMP-006** — the transactional supersede verb (unblocks the OD-3 governance
  supersession migration exclusion).
- **IMP-032** — the supersession `validate` cross-check (corrected; implemented by
  this slice — close on SL-048 closure).
- Any further kinds' relationship seams not covered here.
- Reciprocal-display render helper across `show` surfaces (if it recurs).
