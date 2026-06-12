# Cross-kind actionable survey/next/explain/blockers CLI

## Context

Slice **2 of 3** in the graph-relations work (PRD-011 / SPEC-001). SL-046 (the
spine) projects every entity kind into a `cordage` graph and exposes the
related/inbound relation view. This slice builds the **derived priority CLI** on
top of that graph: the operator-facing "what should I look at next, and why?"
surfaces SPEC-001 names but that do not yet exist —
`survey` / `next` / `explain` / `blockers` (and `inspect` if not folded into
SL-046's query).

**This slice broadens intent beyond the original canon** — the agreed intent is
that **all kinds rank as actionable work** (slices, requirements, specs, … with a
work-like lifecycle), not backlog alone. **The canon revision has now landed** (no
longer a blocker):
- **PRD-011** (commit `6d59397`) — renamed "Graph-Derived Priority and
  Actionability"; the actionable channel spans all admitted kinds via the
  **status×relations synthesis** (actionable when the item's own lifecycle status
  is workable *and* its admitted relations leave it unblocked; no kind barred as a
  kind). New FR-008/`REQ-237` anchors it.
- **SPEC-001** (commit `c3cb719`) — **D12** fixes the mechanism: `actionable =
  eligible ∧ ¬blocked`, uniform across kinds; the per-kind **workable|terminal
  status-class partition** is policy data. New FR-006/`REQ-238` anchors it.

This slice now designs against settled canon. **Design is locked** (`design.md`):
OQ-8 is settled (the work-only partition table, DD-3), SL-046 is `ready`
(design-locked → contract firm to design against; implementation sequences after
SL-046 lands, DD-1), and the dep/seq engine is built kind-agnostic with cross-kind
capture deferred (DD-2). See `design.md` §7 for DD-1..DD-4.

## Scope & Objectives

1. **`survey`** — the importance lens over all active items across kinds:
   `authored-priority → actionability → consequence → deterministic fallback`
   (SPEC-001 D10). High-priority **blocked** items float high with a blocked
   badge.
2. **`next`** — the do-now lens: filter to actionable, order by `order_key`
   (dep-topology → seq rank → fallback, D9). Blocked items absent.
3. **`blockers <ID>`** — direct blockers by default; `--transitive` walks the
   chain both directions (D11; REQ-073).
4. **`explain <ID>`** — always walks to root cause; renders the structured
   reasons (order_key, predecessors, evictions) as the "why" surface (D11).
5. Honour the derived-output invariants: deterministic (REQ-077), disposable
   cache stamped with policy version (REQ-094), never authored back (REQ-078),
   terminal/promoted excluded from default active output (REQ-075), cycle
   diagnostics degrade rather than emit a false order (REQ-076).

## Non-Goals

- **No relation/inbound query or all-kind graph projection** — that is SL-046,
  the prerequisite. This slice consumes that graph; it does not build it.
- **No new authored relation schema** — slice 3 (SL-048).
- **No `cordage` core change** (SPEC-001 D1) — `order_key`/`ordered`/`explain`/
  `reachable` consumed as-is.
- **No item-level authored-priority scalar** (PRD-011 OQ-001, owned by PRD-009) —
  if still unbuilt, survey's authored-priority slot stays empty (D10 explicitly
  allows this). No behaviour rests on it.
- **No architectural-trigger actionability mask** (IMP-026 / D6) — separate,
  prerequisite-bound.
- **No cross-kind dep/seq *capture*** (DD-2). v1 builds the dep/seq overlay +
  actionability policy **kind-agnostic** but consumes only existing backlog
  `needs`/`after`; so `¬blocked` is real for backlog, vacuous for other kinds
  (their actionability reduces to `eligible`). Cross-kind dep authoring is IMP-033 +
  an unsettled governance call (ADR-010 excluded the dep/seq axis); it auto-lights
  here with zero change when authored.
- **No *runtime*-state-derived actionability** (DD-4). v1 reads authored state only.
  **RV is admitted** — its active/done is authored-derived (`review::derived_status`
  over the committed finding ledger), so an `Active` RV is eligible. Only **REC** (no
  lifecycle) and the **slice phase-rollup** (gitignored runtime) stay context-only;
  REC is non-eligible via the status-less path (no diagnostic, not barred as a kind).
- **No persisted cache** (D6). v1 recomputes per query (SPEC-001 H1); `--json`
  stamps `policy_version`.

## Affected Surface

- `src/priority/` — **new**: `graph.rs` (priority adapter: shared scan → 3rd Graph
  with dep/seq overlays + node attrs + `OrderSpec`), `partition.rs` (OQ-8 table),
  `channels.rs` (eligible/actionable/blocked/blocking/consequence/order_key),
  `view.rs` (structured reasons + rows), `render.rs` (human + `--json`).
- `src/relation_graph.rs` (SL-046) — its `KINDS`-walk → `Projection` scan exposed as
  a `pub(crate)` seam, reused by the priority adapter (**fed into SL-046**, design D5);
  `inspect` extended with the actionability block.
- `src/main.rs` — four new CLI verbs (`survey`/`next`/`explain`/`blockers`) +
  `inspect` actionability extension.
- `crates/cordage/`, `src/projection.rs` — consumed, not modified.
- Per-kind modules — a thin authored-`status` accessor where not already exposed
  (cohesion: each module parses its own).

## Risks, Assumptions, Open Questions

Blocking dependency:
- ~~**Canon revision is a hard prerequisite** (PRD-011 + SPEC-001).~~ **Landed**
  (PRD-011 `6d59397`, SPEC-001 `c3cb719`) — see Context. Remaining hard
  prerequisite: **SL-046 lands first** (the graph spine this slice consumes).

Open questions — **all resolved in `design.md`**:
- ~~**Which per-kind statuses are "actionable"?** (SPEC-001 OQ-8)~~ Resolved by
  **DD-3**: the work-only partition table (`design.md` §5.3) — `active`/governing/
  `accepted`/satisfied = terminal-as-work, only in-flight authoring statuses
  workable; the revision out-clause covers governing artifacts that later need work.
- ~~**Cross-kind `consequence`**~~ Settled: v1 = the **work/lineage label subset** of
  existing reference edges (`specs`/`requirements`/`slices`/`descends_from`/`parent`/
  `members`; `reviews`/`owning_slice` bookkeeping excluded, Charge V; SPEC-001 OQ-2
  deferred knowledge_record). Computed as a pre-pass inbound tally; drives survey's
  tier-3 key + next's fallback mint order.
- ~~**CLI shape consistency**~~ Resolved (D5): `inspect` extended (same verb,
  SL-046 D1); `render.rs` rides `src/listing.rs` + SL-045/SL-046 precedent.

Assumptions:
- SL-046 lands first and exposes the scan as a `pub(crate)` seam (design D5,
  fed into SL-046) sufficient for the priority adapter.
- Determinism posture (no clock/RNG/map-order) inherited from cordage + adapter.

## Verification / Closure Intent

- `survey` and `next` **diverge** correctly on a ranked-but-blocked item (top of
  survey with badge; absent from next) — the D10 feature test.
- `blockers --transitive` and `explain` surface the full chain; list rows show
  only direct blockers (D11); toggling display depth never changes ordering.
- A `dep` cycle degrades with a diagnostic, never a false topological order
  (REQ-076).
- Deterministic output under input permutation (REQ-077).
- Actionable ranking spans **all admitted kinds**, not just backlog (the
  broadened intent) — proven over a seeded multi-kind corpus.
- Behaviour-preservation: `backlog order` and cordage suites stay green.

## Follow-Ups

- **Cross-kind dep/seq capture** (IMP-033 + governance) — authors `needs`/`after`
  on slices/specs; auto-lights this slice's already-kind-agnostic engine (DD-2).
- **Slice phase-rollup actionability** (DD-4) — read the gitignored runtime phase
  tree to enrich mid-flight-slice actionability; kept out of v1 so the scan stays over
  authored state. (RV active/done is authored-derived and already in v1.)
- **Persisted policy-stamped cache** (D6) — disposable, recompute-equivalent.
- **Coverage-driven requirement actionability** — the 2nd-enum (`CoverageStatus`)
  axis; v1 uses authored `ReqStatus` only.
- Architectural-trigger actionability mask (IMP-026 / SPEC-001 D6) once its
  file-set sources are built.
- Item-level authored-priority scalar (PRD-011 OQ-001 / PRD-009) — fills survey's
  empty slot when it lands.
- knowledge_record consequence seam (PRD-010, unbuilt).
