# SL-158 design notes — Trinary actionability

Working notes for the `/design` pass. Reading list + sources consulted so a
reviewer can bootstrap. Decisions land in `design.md`; this file is the trail.

## ⚠️ STATUS: design PARKED — gated on an upstream RFC (2026-06-26)

The design conversation itself surfaced that the **gating-mechanism question is
not slice-shaped.** It is an unsettled model decision in RFC-007 that RFC-003's
Layer-1 ruling (structure ≠ graph-effect) reopens, with corpus-wide blast radius,
needing several deliberation passes. Decision (user): **resolve it in an RFC,
outside the slice.** SL-158 cannot proceed to a lockable design until the RFC
settles the mechanism. This notes file + the RFC carry the context forward.

### The requirement the RFC must honour (user, 2026-06-26)

1. **Gating canon is acceptable / can be a positive.** Governance is not itself
   actionable, but an *unsettled* governance record transitively gating the work
   that depends on it is sensible. So "records gate canon" is not disqualifying.
2. **Association must not be hostage to gating (the protect-this).** One must be
   able to associate an epistemic record wherever it is *semantically* sensible
   (`shapes`) WITHOUT that association forcing an *insensible actionability*
   effect. → RFC-003 Layer 1 applied to records: **association ≠ gating.** This is
   the decisive argument against blanket (P) shapes-projection, which couples them.
3. So the live design space = **how gating selects WHICH associations bite**: a
   role on `shapes`, a target-class filter, a distinct gating signal, or a mix.

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

## RFC-003 — bears directly on the gating-edge question (READ THIS)

Holistic relation-model review (CHR-024 deliverable, 2026-06-23). Two rulings that
reframe how gating is modelled:

- **Layer 1 — structure is not graph-effect.** "Whether an edge *gates work* … is a
  **consumer decision**, not a property of the relation." Relation contract =
  structural truth (`shapes` durable); graph-effect (gating/eviction/scoring) =
  a *projection* over selected edges, declared **in the consumer**.
- **Design law — derivable, not relational.** Don't encode in the relation what the
  consumer can project. RFC-003 explicitly files actionability
  (`needs`/`after`/"IMP-047 `gates`") in the **dep/seq layer, not the semantic
  relation model**; and names IMP-047 as "graph-effect is consumer policy; the
  gating layer."

This **tensions IMP-047 / RFC-007 wording** ("new gating `RelationLabel`(s) +
`RELATION_RULES` rows") — RFC-003 says gating is NOT a new vocabulary label; it is a
consumer projection over a structural edge (or a dep/seq-layer axis). Recency cuts
both ways: RFC-003 is 2026-06-23; RFC-007 (2026-06-26) repeats the older
label-wording, possibly inherited from IMP-047 without reconciling vs RFC-003.

## User prior-lean (DO NOT FORGET — challengeable)

In the RFC-003 design conversation the lean was **"shapes not edges"** — i.e. gating
is a **consumer projection over the existing `Shapes` relation**, not a new
`gates` edge/label. User: "we can challenge that view, just not forget it."

## Three mechanism options for the gating edge

- **(P) Shapes-projection (prior lean, RFC-003 Layer 1).** Keep `Shapes` as the
  durable semantic relation; the priority consumer (`graph.rs` edge-pass) projects
  record-sourced `Shapes` edges into the `dep_overlay`, oriented record→artefact.
  Partition split does the gating (Gating blocks, Terminal doesn't). **NO `relation.rs`
  change, NO `dep_seq` change** — just graph.rs edge-pass + partition. Minimal,
  canon-coherent with RFC-003.
- **(E) Distinct `gates` dep/seq axis.** New `[relationships].gates` axis alongside
  `needs`/`after`, extended in `dep_seq` + `dep_seq_for` + graph.rs + an authoring
  verb. RFC-003's "dep/seq layer" reading taken literally as a new axis.
- **(L) Distinct `Gates` RelationLabel + `RELATION_RULES` rows.** IMP-047/RFC-007
  literal wording. RFC-003 argues this is the **wrong layer** (graph-effect ≠
  vocabulary).

**Challenge to (P) — kept live:** `Shapes` targets a WIDE set (PRD/SPEC/REQ/SL/
ISS/IMP/CHR/RSK/IDE/ADR/POL/STD + the 4 record kinds). If every unsettled-record
shaping gates, blocking may be over-broad (a record shaping a `draft` SPEC blocks
it; intra-family ASM→QUE gating), and the author loses "informs but does not block."
Mitigations: only *unsettled* records gate (settled = inert); soft-shape control
(role/facet on shapes) deferrable if it bites.

## Open design questions (decide before readiness)

- **OQ-1** — name of the third class: `Gating` / `Ambient` / `Pending`.
- **OQ-2** — edge direction: outbound-from-record (fits ADR-004 outbound-only) vs
  dependent's `needs → record`. (Largely settled → outbound-from-record.)
- **OQ-3 (the live fork)** — mechanism: shapes-projection (P, prior lean) vs distinct
  `gates` axis (E) vs `Gates` label (L). RFC-003 favours P/E over L.

## Guardrails (from handover)

- Behaviour-preservation: ordinary workable/terminal items unchanged; existing
  priority suites green; trinary reduces to binary where no Gating node exists.
- Don't fork IMP-033's dep-overlay machinery — coordinate.
- Canon-first: SPEC-001/PRD-011 D-decision + requirement authored via
  design→reconcile, not hand-edited ahead of the engine.
- Jail: reservation needs `DOCTRINE_RESERVATION_FALLBACK=1`. `link` flag is
  `--role` not `--intent`.
