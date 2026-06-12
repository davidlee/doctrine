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
2. **Edges from existing authored outbound relations only.** Emit typed edges
   from the relations each kind already authors — slice `specs`/`requirements`/
   `supersedes`; spec `descends_from` + members; backlog `specs`/`slices`/`drift`/
   `needs`/`after`; governance `supersedes`/`superseded_by`/`related`. The inert
   governance `[relationships]` becomes a **read-only** graph input here (no new
   authored fields — that is slice 3). ADR-004 holds: outbound only.
3. **Universal related/inbound query.** Given any entity id, report (a) its
   authored **outbound** relations and (b) its **derived inbound** references,
   computed from cordage `in_edges`/`reachable` — never a stored reverse field
   (REQ-074 / REQ-078 / D8).
4. **A CLI surface for the query.** Exact shape is a design decision (enrich
   `<kind> show` vs a dedicated verb) — see Open Questions.

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

- `src/backlog_order.rs` — the existing adapter; design decides generalise-in-place
  vs extract a shared all-kind adapter module (the backlog ordering must keep
  working unchanged — behaviour-preservation gate).
- `src/integrity.rs` — `KINDS` corpus-wide id table (read; the single id source).
- Per-kind relation readers — `src/slice.rs`, `src/spec.rs`, `src/governance.rs`,
  `src/backlog.rs` — read each kind's authored outbound relations.
- `src/main.rs` — CLI wiring for the query surface.
- `crates/cordage/` — **consumed, not modified** (`in_edges`, `reachable`,
  `GraphBuilder`).

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

Open questions (for `/design`, not resolved here):
- **Query CLI shape** — enrich each `<kind> show` with a related/inbound section,
  or a single dedicated cross-kind verb (e.g. `doctrine related <ID>`)? Affects
  discoverability and the column/render model.
- **Overlay typing for the relation view** — do spine edges land in a generic
  "reference" overlay distinct from `dep`/`seq`, or reuse existing overlays? The
  inbound view needs reachability over reference/lineage edges; actionability
  (`dep`/`seq`) is slice 2's concern.
- **Adapter structure** — generalise `backlog_order` vs a new shared adapter the
  backlog ordering also rides. Must not regress `backlog order`.

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
