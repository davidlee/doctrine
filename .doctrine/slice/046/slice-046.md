# Cross-kind relation graph spine: all-entity adapter + related/inbound query

## Context

PRD-011 / SPEC-001 specify a graph-derived priority and relation view over
doctrine's entity graph. The generic core (`cordage`) is shipped and hardened
(SL-036 / SL-038 / SL-043): opaque nodes, typed overlays, reverse index
(`in_edges`), `reachable`, cycle policy, deterministic order, explanation.

But the only adapter into it is `src/backlog_order.rs` — it scans the **5 backlog
kinds only** and produces **one output: an ordering** (`backlog order`). Slices,
specs, requirements, ADRs, and governance kinds are never projected into the
graph. There is **no cross-kind relation surface**: no command answers "what is
this entity related to, and what references it?" The governance `[relationships]`
block (`supersedes`/`superseded_by`/`related`/`tags`) is parsed but **inert —
never queried** (`src/governance.rs`). Inbound references are derivable in-core
(ADR-004 outbound-only + cordage reverse index, PRD-011 H2) but unbuilt.

This is **slice 1 of 3** — the connective spine. It is buildable under current
canon (PRD-011 §2 already scopes the graph to all kinds; REQ-074 owns the inbound
view) with **no spec revision**. Slices 2 (cross-kind actionable survey/next CLI)
and 3 (new structural cross-corpus edges, IMP-016) sit behind it and need canon
to move first — see Follow-Ups.

## Scope & Objectives

1. **All-kind registry adapter.** Generalise the registry→cordage projection
   beyond backlog: scan every numbered entity kind (the `integrity::KINDS` id
   table) into the graph as opaque nodes keyed by globally-unique canonical id,
   re-mapping diagnostics back to doctrine ids (REQ-091).
2. **Edges from existing authored outbound relations only — reference/lineage,
   not dep/seq.** Emit typed edges from the *reference and lineage* relations each
   kind already authors — slice `specs`/`requirements`/`supersedes`; spec
   `descends_from`/`parent`/members; backlog `specs`/`slices`/`drift`; governance
   `supersedes`/`related`. The inert governance `[relationships]` becomes a
   **read-only** graph input here (no new authored fields — that is slice 3).
   ADR-004 holds: outbound only. **Excluded**: `needs`/`after` (dep/seq —
   actionability, slice 2's overlays), `triggers` (mask), `tags` (free-text).
   **Reader rule (D4):** project the canonical *outbound* direction only and derive
   reciprocals from `in_edges`; do **not** project governance `superseded_by` (it is
   the derived inbound of `supersedes`; storing it is the ADR-004 violation IMP-032
   reconciles in slice 3).
3. **Universal related/inbound query — direct-only.** Given any entity id, report
   (a) its authored **outbound** relations and (b) its **derived inbound**
   references, computed from cordage `in_edges` (one hop; no `reachable` walk — no
   `--transitive` in v1) — never a stored reverse field (REQ-074 / REQ-078 / D8).
4. **`doctrine inspect <ID>`** — the dedicated cross-kind verb (D1). SPEC-001's
   reserved `inspect` surface, shipped relation-only here; slice 2 layers
   actionability/blockers onto the same verb.

## Non-Goals

Boundary — explicitly **out**, deferred to later slices or untouched:

- **No actionability ranking / `survey` / `next` / `explain` / `blockers` / item
  scoring.** That is slice 2 and needs a PRD-011 revision (rank all kinds as
  actionable, per the broadened intent). This slice ships *relation visibility*,
  not prioritisation.
- **No new authored relation schema** — no spec↔ADR or product↔product fields, no
  activation of new governance links beyond reading what is already parsed. That
  is slice 3 (IMP-016) and needs cross-corpus relation governance.
- **No change to the `cordage` core** — it is shipped and locked (SPEC-001 D1).
  Consumed as-is via its public API.
- **No stored reverse index / inbound field** on any entity (ADR-004).
- **No `dep`/`seq` actionability semantics** beyond what `backlog_order` already
  does; the spine's edges feed the *relation/inbound* view, not blocking.

## Affected Surface

- `src/projection.rs` — **new (leaf)**: the generic `Projection<K>` bimap primitive
  (D3), shared by both adapters.
- `src/relation_graph.rs` — **new (engine)**: the all-kind scan → projection + ref
  overlays → `inspect` query.
- `src/backlog_order.rs` — its inline bimap swaps to `Projection<ItemId>`; scan +
  overlays + `OrderSpec` otherwise **unchanged** (behaviour-preservation gate).
- `src/integrity.rs` — `KINDS` corpus-wide id table (read; the single id source +
  prefix→kind resolution).
- Per-kind relation readers — `src/slice.rs`, `src/spec.rs`, `src/governance.rs`,
  `src/backlog.rs` — each gains a `pub(crate) relation_edges` accessor reading its
  own (currently private) `Relationships`.
- `src/main.rs` — `inspect` CLI wiring (command layer).
- `crates/cordage/` — **consumed, not modified** (`in_edges`, `GraphBuilder`).

## Risks, Assumptions, Open Questions

Risks:
- **Free-text / unvalidated refs** (`drift`, governance `related`) are not
  forward-validated (mem.pattern.entity.free-text-ref-not-forward-validated). The
  adapter must tolerate dangling/free-text targets — map to a node only when the
  target resolves, else surface as a dangler, never panic.
- **Duplicate node key corrupts the bimap** (RSK-005). The all-kind scan must
  guarantee distinct node keys; canonical ids are globally unique by prefix, but
  this must be asserted at the projection boundary, not assumed.
- **Corpus dir-walk** must skip the `NNN-slug` symlink alias beside each numeric
  entity dir (mem.pattern.entity.corpus-walk-skip-slug-symlink).

Assumptions:
- cordage `in_edges` + `reachable` suffice for the inbound view (confirmed —
  `crates/cordage/src/lib.rs`). No core change needed.
- Canonical id (prefixed, e.g. `SL-046`) is the stable, globally-unique node key
  across all kinds.

Open questions — **all resolved in `design.md`** (D1–D4):
- **Query CLI shape** → D1: dedicated `doctrine inspect <ID>`, relation-only,
  direct-only (SPEC-001's reserved surface; slice 2 layers priority onto it).
- **Overlay typing** → D2: one `Reject`/`Unbounded` overlay per relation label
  (label = overlay identity), distinct from `dep`/`seq`. Cycle/error semantics
  proven safe (`Reject` loses no edges; `in_edges` is composition-free, so no
  overlay- or union-acyclicity is assumed).
- **Adapter structure** → D3: extract a generic `Projection<K>` primitive;
  `backlog_order` and the new `relation_graph` adapter both ride it (backlog scan +
  overlays untouched — the gate).

## Verification / Closure Intent

- Over a seeded multi-kind corpus, the query returns correct authored **outbound**
  relations and correct **derived inbound** references for entities of every kind.
- Structural proof that **no stored reverse field** is introduced (ADR-004): the
  inbound view recomputes from authored outbound edges alone.
- Adapter **tolerates** free-text and dangling refs (no panic; danglers surfaced).
- Output is **deterministic** (no clock/RNG/map-iteration order — REQ-077 posture).
- **Behaviour-preservation gate**: the existing `cordage` and `backlog_order`
  suites stay green **unchanged**; `backlog order` output is byte-identical.
- The `cordage` core gains **no doctrine vocabulary** (REQ-079).

## Follow-Ups

- **Slice 2 — cross-kind actionable CLI** (`survey`/`next`/`explain`/`blockers`,
  ranking all kinds as actionable). Blocked on a **PRD-011 + SPEC-001 revision**
  (today the actionable channel is backlog-lifecycle-only).
- **Slice 3 — structural cross-corpus edges** (IMP-016: activate governance
  `[relationships]` as authored, add spec↔ADR / product↔product fields). Blocked
  on **cross-corpus relation governance** — kind-spec updates (SPEC-005/006/016)
  + likely a new ADR. ADR-004 outbound-only still governs.
- Deferred engine seams unaffected here: item-level priority scalar (PRD-011
  OQ-001), trigger file-set sources (IMP-026 / D6), `knowledge_record` consequence
  seam (PRD-010, unbuilt).

Filed during design (challenges captured now, fixes land downstream):
- **IMP-032** — governance `superseded_by` is a stored reciprocal; derive it, don't
  store it (ADR-004). SL-046's reader already ignores it; field removal + migration
  is slice 3.
- **IMP-033** — cross-kind dep/seq capture (extend `needs`/`after` to specs/slices).
  Capture-side; slice 3 / PRD-009 + the relation-governance ADR.
- **IMP-034** — interrogate refactoring *all* relations modelling to a uniform
  schema. Parallel with or direct successor to this slice; likely feeds slice 3's
  ADR.
