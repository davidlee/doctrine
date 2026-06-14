# Cross-kind dep/seq capture: extend needs/after authoring beyond backlog

Realises **IMP-033**. Surfaced during SL-046 design; deferred there as a
different semantic layer (capture-side schema change, not the reference/lineage
reader). Sequenced **before IMP-047** (trinary actionability), which rides this
slice's generalised dep/seq seam. Design locked in `design.md`; this scope
reflects the locked decisions.

## Context

The dep/sequence axis — hard `needs` (prerequisite, payload-free) and soft `after`
(sequence, per-edge `rank`) — is **authored only on backlog items today**
(`src/backlog.rs` `Relationships{needs, after}`; verbs `backlog needs` /
`backlog after`). The intent is cross-kind: a slice wants to sequence after, or be
blocked until, a prerequisite slice. There is no surface to author it.

Two facts fix the boundary:

- **These are NOT relation-vocabulary edges.** ADR-010 D1 carves the dep/sequence
  axis **out** of the unified relation contract (`RELATION_RULES` / the `link`
  writer); `needs`/`after` are typed Tier-2 **payload** edges (`after.rank`). This
  slice does **not** extend `link`/`RelationLabel`.
- **The consumer is already cross-kind.** `priority/graph.rs` (SL-047) builds the
  `dep`/`seq` overlays over `EntityKey` and emits edges **kind-agnostically**
  (DD-2). The only backlog-bound point is the dep/seq *read* gate
  (`backlog::kind_from_prefix`). So the original "which projector / does the reader
  admit new edges" question (scope OQ-1) **dissolves** — the reader is one gate
  away from cross-kind.

## Scope & Objectives

Extend the typed `needs`/`after` capture from backlog-only to **slices**, lift the
schema+write+read into a shared leaf (no parallel implementation), and generalise
the one consumer read-gate so slice dep/seq edges reach the existing cross-kind
`blockers`/`next` view. Storage stays bespoke/typed (ADR-010 D1).

In scope:

- Shared leaf `src/dep_seq.rs` — typed `DepSeq{needs, after}` + edit-preserving
  `append` (lifted from backlog's `append_relationship`) + path-based `read`.
- Generic top-level verbs `doctrine needs <SRC> <TGT>` / `doctrine after <SRC>
  <TGT> [--rank N]` (sibling to `link`); `backlog needs`/`after` delegate.
- Slice authoring: seeded `[relationships]` table in the slice scaffold (before
  `[[relation]]` rows); slice arm in the engine dep/seq dispatch.
- Generalise `priority/graph.rs` §3b read-gate from backlog-prefix to
  kind-dispatch (non-authoring kinds short-circuit, no disk read).
- Forward-edge target validation: target resolves against `integrity::KINDS` AND is
  a **work-like kind** (slice + 5 backlog); non-work kinds (governance/spec/req/
  knowledge) and self-edge refused at author time.
- **Canon moves first:** PRD-011 amendment claiming the cross-kind dep/seq
  *capture* intent (PRD-009 keeps the backlog instance; SPEC-001 already
  mechanises kind-agnostically).

## Non-Goals

- **Not the relation `link` writer / `RELATION_RULES`** — separate axis (ADR-010 D1).
- **Not `triggers`** — the watched-glob rider stays a third axis, out of scope.
- **Not actionability/blocker semantics** — no `priority/partition.rs` change; the
  third `Gating` status-class and the `gates` label are **IMP-047's**. SL-060
  leaves the `dep_overlay` open for IMP-047's second (labelled) producer; it does
  not unify the two producers.
- **Not specs/ADRs as dep/seq sources OR targets** — depending on governance is
  *pending revise-intent*, the role of a future **Revision** kind (**IDE-010**), not
  a `needs` edge. Slices only as source; targets are work-like only (slice +
  backlog). A non-work target is refused at author time — not "inert" (it would
  block while in a workable status). Cross-tier gating (governance→work) is IMP-047's
  labelled `gates`.
- **Not a backlog regression** — backlog `needs`/`after` + `priority` +
  `backlog order` goldens stay byte-identical (behaviour-preservation gate).
- **No shipped migration / create-on-absent leniency** — durable code stays
  strict; the pre-existing-slices gap is fixed out-of-band
  (`mem.pattern.design.product-not-compromised-by-project-local-ops`).

## Affected surface (concrete)

- `src/dep_seq.rs` (new leaf) — schema + append + read.
- `src/backlog.rs` — `needs`/`after`/`append_relationship`/`dep_seq_for` re-point
  to the leaf type; backlog arm keeps single `read_item` (for `promoted`).
- `src/slice.rs` + `install/templates/slice.toml` — seeded `[relationships]` (both
  empty arrays, before `[[relation]]` rows); slice dep/seq read arm.
- `src/relation_graph.rs` (or beside) — `dep_seq_for(root, kind, id) -> (DepSeq,
  promoted)` engine dispatch, mirroring `outbound_for`.
- `src/priority/graph.rs` — §3b read-gate generalised; emission unchanged.
- CLI wiring (`main.rs`) — top-level `needs`/`after` verbs.
- PRD-011 (`/spec-product` amendment).

## Risks / Assumptions

- **R1 — toml_edit positioning** of the seeded `[relationships]` table before
  `[[relation]]` rows. The one impl risk; the leaf never *creates* the table
  (strict), so positioning is a scaffold/backfill concern, pinned by a round-trip
  golden + SL-048-style storage post-check.
- **R2 — the lift perturbs backlog behaviour.** Byte-identical gate; mechanical
  move + thin delegate.
- **ASM-1 (load-bearing)** — the no-migration stance assumes *no upgrade-in-place
  clients with pre-existing slices*. True today (doctrine `install` scaffolds the
  table on fresh installs; only this dogfood repo has table-less slices). If
  doctrine gains upgrade-in-place users, a backfill/lazy-seed story is needed
  (follow-up).
- **A1** — `rank` is a pairwise-edge attribute, never the item-level priority
  scalar (PRD-009 §4 / PRD-011 OQ-001).

## Open Questions

- **OQ-1 — DISSOLVED.** Consumer already cross-kind (DD-2); `backlog_order.rs`
  untouched.
- **OQ-2 — RESOLVED (revised).** Targets restricted to work-like kinds (slice + 5
  backlog); non-work refused at author time. The "resolvability-only / inert" stance
  was refuted (a non-terminal non-work predecessor blocks via `channels::blocked_by`).
  Gradient inversion is IMP-047's `gates` / the Revision kind (IDE-010), not enforced
  here.
- **OQ-3 — RESOLVED.** Generic top-level verbs; backlog delegates.
- **OQ-4 — `triggers` cross-kind** — held out; revisit if a later slice wants it.
- **PARKED (forelobe) — semantic edge labels.** Whether dep edges carry a
  distinguishing label (`needs` / IMP-047 `gates` / `blocks`) vs collapsing into
  one unlabelled `dep_overlay`. Forced by IMP-047; not decided here.

## Verification / closure intent

- slice→slice `needs`/`after` authorable; round-trips `slice show` / `--json`.
- an authored slice→slice `needs` surfaces as a cross-kind blocker in
  `priority blockers` / holds the dependent in `next`.
- backlog + `priority` + `backlog order` goldens byte-identical (incl. verb
  message text through the delegate).
- validation refusals tested (unresolvable / free-text / self-edge / non-authoring
  SRC kind / non-work-like TGT kind).
- out-of-band backfill: storage post-check + round-trip `show`/`validate` clean.
- `just gate` clean; clippy zero warnings.

## Summary

Capture-side extension of the dep/seq axis (`needs`/`after`) from backlog-only to
slices, via a shared `dep_seq` leaf and a one-gate generalisation of the
already-kind-agnostic priority consumer. Distinct from the SL-046/047/048
reference/lineage `link` axis (ADR-010 D1 keeps dep/seq typed). Canon moves first
(PRD-011 amendment). Lays the cross-kind dep-overlay seam IMP-047 extends; defers
governance-revise-dependency to the Revision kind (IDE-010).

## Follow-Ups

- **IMP-047** — trinary actionability / `Gating` + the `gates` label producer into
  the same `dep_overlay` (sequenced after this slice).
- **IDE-010** — the Revision kind (governance-revise-dependency modelling).
- **`triggers` cross-kind** (OQ-4) — if a later slice needs it.
- **Upgrade-in-place backfill story** — if/when doctrine gains clients that upgrade
  in place over pre-existing slices (ASM-1).
- Any ADR minted in execution recorded + linked `governed_by`.
