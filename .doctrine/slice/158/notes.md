# SL-158 design notes — Trinary actionability

Working notes for the `/design` pass. Reading list + sources consulted so a
reviewer can bootstrap. Decisions land in `design.md`; this file is the trail.

## Reading list (handover order)

1. `doctrine slice show SL-158` — scope, non-goals, affected surface, OQs.
2. `doctrine rfc show RFC-007` — program: 3 workstreams, sequencing. ws1 keystone.
3. `doctrine backlog show IMP-047` — source item; richest mechanism spec.
4. `.doctrine/spec/tech/019/spec-019.md` — D7 / NF-003 / OQ-2: the consumer.
   Records parked all-`Terminal`, explicitly awaiting IMP-047.
5. `src/priority/{partition,channels,graph}.rs` — the engine seam.

## Other sources consulted

- `ADR-004` (superseded_by ADR-012) — relations stored **outbound-only**,
  reciprocity derived. One canonical authoring side; record→artefact authored
  **on the record**. Bears on OQ-2 (edge direction).
- `ADR-010` — relation modelling: unify contract + write seam (`RELATION_RULES`).
- `src/relation.rs` — `RelationRule` (sources, label, role, inbound_name,
  TargetSpec, Tier, LinkPolicy), `RELATION_RULES` table. Records (`RECORD`
  source-group) already author `Shapes` (→ wide artefact set incl SL/REQ/ISS…)
  and `Spawns` (→ backlog kinds). Both are currently **graph-inert** (not in any
  overlay).

## Key engine findings (the load-bearing facts)

### partition.rs — the binary that must become trinary
- `StatusClass { Workable, Terminal, Unrecognised }`. `status_class(kind, status)`
  is a `(prefix → KindPartition{workable,terminal})` lookup.
- VT-1 **drift canary** per kind: `workable ∪ terminal == <kind>_STATUSES`. Adding
  a `gating` set generalises this to `workable ∪ gating ∪ terminal == vocab`.
- Knowledge kinds (ASM/DEC/QUE/CON) currently `workable: &[]`,
  `terminal: <KIND>_STATUSES` (all-Terminal, the SL-059 interim per D7).
- Per-kind settle boundary (unsettled = Gating, settled = Terminal):
  - ASM: `held`,`testing` → Gating; `validated`,`invalidated`,`obsolete` → Terminal
  - DEC: `proposed` → Gating; `accepted`,`rejected`,`superseded` → Terminal
  - QUE: `open` → Gating; `answered`,`obsolete` → Terminal
  - CON: `active` → Gating; `waived`,`superseded`,`retired` → Terminal
- Existing tests that MUST change (consumer revision, not behaviour regression):
  `every_knowledge_status_classifies_terminal_never_workable`,
  `knowledge_partitions_cover_the_real_vocabularies` (canary form),
  `decision_accepted_diverges_hidden_from_status_class` (accepted stays Terminal).

### channels.rs — **needs (almost) no change** (the elegant part)
- `eligible = class == Workable` → Gating excluded from worklist automatically.
- `blocked_by` keeps dep-overlay predecessors with `class != Terminal` → a Gating
  predecessor blocks automatically; a settled (Terminal) one stops blocking.
- So the trinary class slots into the existing pole tests. **Settle→unblock falls
  out for free.** Behaviour-preservation holds: no Gating node ⇒ identical.

### graph.rs — the gating EDGE (the real second change)
- dep overlay fed only by `needs`/`after` via `relation_graph::dep_seq_for`.
- Reference/lineage overlays fed by `entity.outbound` filtered to `REF_LABELS`
  (NOT incl `Shapes`/`Spawns`). So record relations contribute **no graph edge**.
- To make a record gate: route a record→artefact edge into `dep_overlay` oriented
  **record→artefact** (record = predecessor/blocker, artefact = successor/blocked)
  — the same B→A flip `needs` uses. Then partition class gates it (Gating blocks,
  Terminal doesn't), unconditional edge.

## Open design questions (decide before readiness)

- **OQ-1** — name of the third class: `Gating` / `Ambient` / `Pending`.
- **OQ-2** — edge direction: outbound-from-record (fits ADR-004 outbound-only) vs
  dependent's `needs → record`.
- **OQ-emergent** — is the gating edge the existing `Shapes` label (every shaping
  record gates while unsettled), or a distinct minted `Gates` label (shaping ≠
  gating)? IMP-047 says "new RelationLabel(s)"; `Shapes` already exists and is
  inert. Needs resolution.

## Guardrails (from handover)

- Behaviour-preservation: ordinary workable/terminal items unchanged; existing
  priority suites green; trinary reduces to binary where no Gating node exists.
- Don't fork IMP-033's dep-overlay machinery — coordinate.
- Canon-first: SPEC-001/PRD-011 D-decision + requirement authored via
  design→reconcile, not hand-edited ahead of the engine.
- Jail: reservation needs `DOCTRINE_RESERVATION_FALLBACK=1`. `link` flag is
  `--role` not `--intent`.
