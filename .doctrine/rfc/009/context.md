# RFC-009 bootstrap: file references for design / plan / review

## What this RFC proposes

**Epistemic records as the human-facing, relational substrate for design ambiguity** —
not a memory rival and not a freeform-prose alternative, but the typed interior of specs
and designs. The RFC is reframed off an earlier "add observation + hypothesis to complete
an empirical loop" draft; that framing died when RFC-008 → ADR-017 made gating
**kind-agnostic** (no per-kind gating posture to design for) and because symmetry is not
need. The RFC is **open**, organised in two fenced tiers:

- **Tier 1 — near-term, standalone.** Strengthen the existing kinds + edges, fix the
  memory↔record boundary, drive uptake through skills. Justified even if Tier 2 fails.
- **Tier 2 — the broader hypothesis (experimental).** A lightweight graph of typed
  linked entities beats markdown for the clarification/spec/coordination work that stays
  human once agents write+review code — pushing specs toward a graph of typed records,
  the way reviews are already structured turn-taking artifacts. May spawn a child RFC.

## Decisions (this draft)

| id | decision | status |
|---|---|---|
| D1 | memory↔record boundary — ownership/legibility test | **settled in draft** (records = graph-resident + human-facing; memory = agent cache; crisp edges over coverage; crystallise out of prose). Open thread: is memory's 2nd-class linkage endemic? |
| D2 | which kinds — adopt OBS+HYP as *decomposition units* (both-or-neither) or neither; risk-as-knowledge folds away | open |
| D3 | record/entity edges — typed lifecycle edges (`confirmed_by`/`refuted_by`), `shapes` epistemic-vs-affects split, which CM relationship types are real edges vs derivable | open — **bulk of near-term work** |
| D4 | concept maps — retire-as-is vs reify-as-view-over-concept-nodes; CON naming collision | open — modest/low-risk |
| D5 | uptake — how far skill-anchoring must go | open — the real failure mode |
| Tier 2 | spec-as-graph hypothesis | labelled bet, not for convergence here |

## Key files (what would change if D2/D3/D4 add kinds or edges)

### Kind registration (3 files, linear dependency)

| file | what lives there | change if a kind is adopted |
|---|---|---|
| `src/kinds.rs:27-30,37` | prefix constants (`ASM`, `DEC`, `QUE`, `CON`) and `RECORD` grouping | add new prefixes; extend `RECORD` slice — this is what makes existing record edges inherit automatically |
| `src/knowledge.rs:57-113` | `RecordKind` enum, per-kind `Kind` constants, `ALL` array, `kind()`/`prefix()`/`from_prefix()` dispatch | add variants + constants; extend `RecordKind::ALL` |
| `src/integrity.rs:119-136` | `KINDS` array — entity-engine registration driving scan/dispatch | add a `KindRef` row per new kind (after CON) |

### Status vocabulary (src/knowledge.rs)

| line(s) | what | change |
|---|---|---|
| `170-194` | per-kind status arrays | new kind's statuses (OBS: `observed`→`confirmed`/`retracted`; HYP: `proposed`→`confirmed`/`refuted`) — non-terminal statuses are exactly what makes a kind gate under ADR-017 |
| `197-203` | per-kind hide-sets | new hidden-sets |
| `206-209` | per-kind terminal sets | new terminal-sets (drives the trinary partition) |
| `212-220` | `statuses()` match arms | new arms |
| `224-228+` | `is_hidden()` / `hidden()` dispatch | new arms |

### Relation vocabulary (src/relation.rs)

| line(s) | what | change |
|---|---|---|
| `RELATION_RULES` table | edge-validity spine | existing `RECORD`-sourced rows (shapes, spawns, supersedes) inherit new kinds **for free** via the `RECORD` grouping — **no table change** for existing labels. D3's new edges (e.g. `confirmed_by` HYP→OBS, modelled `LifecycleOnly` like `supersedes`) would add rows |
| `src/kinds.rs:37` | `RECORD` grouping | extending it is the single lever that propagates existing record edges to new kinds |

See `edge-validation.md` for the full `RELATION_RULES` snapshot (which labels each source
kind authors, target gates, create-invalid edges).

### Lifecycle / CLI / scan (mostly automatic)

| file | what | change |
|---|---|---|
| `src/knowledge.rs:~57-113` | `record_scaffold()` / `RecordKind` | new kinds ride the existing scaffold; templates need per-kind variants |
| `src/commands/knowledge.rs` | `knowledge new/list/show` | `RecordKind::from_prefix(...)` must resolve — the `ALL` array drives the CLI |
| `src/catalog/scan.rs:~59-65` | `outbound_for` dispatch | knowledge arm reads `relation_edges` for any recognised prefix — extending `RecordKind` covers new kinds automatically |
| `src/relation_graph.rs` | `dep_seq_for` fallthrough | knowledge kinds hit `else` → empty dep/seq — unchanged (records are gating-inert as *sources* per ADR-017) |

### Templates / authored files

| path | what |
|---|---|
| `.doctrine/knowledge/{assumption,decision,question,constraint}/` | existing per-kind template dirs |
| new dirs per adopted kind | `record-NNN.toml` + `record-NNN.md` scaffold |

### D4 — concept maps (if reified)

- `src/` concept-map subsystem + `doctrine concept-map` command surface (CM-001, CM-002).
- Reifying a concept into a first-class numbered kind needs its **own prefix** (DEF/CPT —
  CON is taken by constraint) and a `KindRef` row; the CM DSL would become a projection
  over real concept-nodes rather than a private node container.

## Related entities (context for design decisions)

### Governing resolution
- **ADR-017** — actionability gating via inbound `needs` on unsettled records (kind-agnostic;
  retired the old gating motivation). `doctrine adr show ADR-017`
- **RFC-008** — gating association vs graph-effect (resolved → ADR-017)
- **RFC-003** — relation model review (`shapes` semantics, *derivable not relational*, D-axis) — bears on D3
- **ADR-016** — relation intent as a closed role dimension (record→record vs record→work split)

### Specs & origin
- **SPEC-019 / PRD-010** — epistemic & governance records (the Tier-2 spec-as-graph bet pushes on these)
- **SL-059** — knowledge record kinds (ASM/DEC/QUE/CON) — the slice that added the current four
- **IMP-182** — `/knowledge` authoring skill (the uptake seed for D5)

### Concept maps & seeds
- **CM-001, CM-002** — concept maps with orphaned virtual nodes (D4 subject)
- **ASM-001, DEC-001, QUE-001, CON-001** — seed knowledge records (2026-06-26)

## Boot order for a new agent

1. Read this file
2. Read `rfc/009/rfc-009.md` (the reframed RFC body — two tiers, D1–D5)
3. Read `rfc/009/edge-validation.md` (current relation rules snapshot)
4. Skim code touchpoints in order: `kinds.rs` → `knowledge.rs` → `integrity.rs` → `relation.rs` RELATION_RULES
5. Read ADR-017 (why gating is kind-agnostic) and RFC-003 (relation-model context for D3)
6. Route via `/route` — convergence that adds kinds/edges emits a Revision (ADR-013)
</content>
