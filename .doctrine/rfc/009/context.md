# RFC-009 bootstrap: file references for design / plan / review

## What this RFC proposes

Two new epistemic record kinds — **observation (OBS)** and **hypothesis (HYP)** —
extending the knowledge taxonomy from 4 kinds (ASM/DEC/QUE/CON) to 6. The RFC
is open; decisions D-a through D-e remain unsettled. A design/plan/review boot
reads the code and the surrounding entities to understand what changes.

## Key files (what would change)

### Kind registration (3 files, linear dependency)

| file | what lives there | change if adopted |
|---|---|---|
| `src/kinds.rs:27-30,37` | prefix constants (`ASM`, `DEC`, `QUE`, `CON`) and `RECORD` grouping | add `OBS`/`HYP` prefixes; extend `RECORD` slice |
| `src/knowledge.rs:57-113` | `RecordKind` enum, per-kind `Kind` constants, `ALL` array, `kind()`/`prefix()`/`from_prefix()` dispatch | add `Observation`/`Hypothesis` variants; add `OBSERVATION_KIND`/`HYPOTHESIS_KIND` constants; extend `RecordKind::ALL` to 6 |
| `src/integrity.rs:119-136` | `KINDS` array — the entity-engine registration table that drives scan/dispatch | add two `KindRef` rows for OBS and HYP (after CON) |

### Status vocabulary (src/knowledge.rs)

| line(s) | what | change |
|---|---|---|
| `170-194` | per-kind status arrays | add `OBSERVATION_STATUSES` (e.g. `observed`→`confirmed`/`retracted`) and `HYPOTHESIS_STATUSES` (e.g. `proposed`→`confirmed`→`refuted`) |
| `197-203` | per-kind hide-sets | add `OBSERVATION_HIDDEN` / `HYPOTHESIS_HIDDEN` |
| `206-209` | per-kind terminal sets | add `OBSERVATION_TERMINAL` / `HYPOTHESIS_TERMINAL` |
| `212-220` | `statuses()` match arms | add OBS/HYP arms |
| `224-226` | `is_hidden()` dispatch | add OBS/HYP arms |
| `228-` | `hidden()` match arms | add OBS/HYP arms |

### Relation vocabulary (src/relation.rs)

| line(s) | what | change |
|---|---|---|
| `RELATION_RULES` table | the edge-validity spine | three existing `RECORD`-sourced rows (shapes, spawns, supersedes) inherit OBS/HYP automatically via the `RECORD` grouping — **no table changes needed** for existing labels. New OBS/HYP-specific edges (e.g. `confirmed_by` from HYP→OBS) would add rows |
| `src/kinds.rs:37` | `RECORD` grouping | extending `RECORD` from `[ASM,DEC,QUE,CON]` to `[ASM,DEC,QUE,CON,OBS,HYP]` makes the three existing `sources: RECORD` rules apply automatically |

### Record lifecycle / CLI (src/knowledge.rs + commands)

| file | what | change |
|---|---|---|
| `src/knowledge.rs:~57-113` | `record_scaffold()` / `RecordKind` | the new kinds ride the existing scaffold path; templates need OBS/HYP variants |
| `src/commands/knowledge.rs` (or analogous) | `knowledge new`, `knowledge list`, `knowledge show` | `RecordKind::from_prefix("OBS")` must resolve — the `ALL` array drives the CLI |
| `src/catalog/scan.rs:~59-65` | `outbound_for` dispatch | the knowledge arm (`RecordKind::from_prefix`) already reads `relation_edges` for any recognized prefix — extending `RecordKind` covers OBS/HYP automatically |
| `src/relation_graph.rs` | `dep_seq_for` fallthrough | knowledge kinds hit the `else` → `backlog::kind_from_prefix()` miss → empty dep/seq — unchanged |

### Templates / authored files

| path | what |
|---|---|
| `.doctrine/knowledge/assumption/` (and sibling dirs) | existing template dirs per kind |
| new: `.doctrine/knowledge/observation/`, `.doctrine/knowledge/hypothesis/` | kind dirs with `record-NNN.toml` + `record-NNN.md` scaffold |

## Related entities (context for design decisions)

### Dependent RFCs
- **RFC-008** — actionability gating (the gating mechanism gates records of these kinds; D-e asks whether taxonomy lands before gating)
- **RFC-003** — relation model review (intra-record `shapes` semantics, D axis for hierarchy)

### Upstream specs & PRDs
- **SPEC-019 / PRD-010** — epistemic & governance records (the product spec these kinds realize)
- `doctrine spec show SPEC-019`, `doctrine spec show PRD-010`

### Original implementation
- **SL-059** — knowledge record kinds (ASM/DEC/QUE/CON) — the slice that added the current four
- `doctrine slice show SL-059`

### Concept maps (relationship types with no epistemic outlet)
- **CM-001**, **CM-002** — `doctrine concept-map show CM-001`, `doctrine concept-map show CM-002`

### Seed records
- **ASM-001**, **DEC-001**, **QUE-001**, **CON-001** — `doctrine knowledge show <ID>`

### Edge-validity reference
- `rfc/009/edge-validation.md` — full `RELATION_RULES` snapshot (generated 2026-06-26): which labels each source kind can author via `link`, target-kind gates, and create-invalid edges

## Design decisions (from RFC body)

| id | question | status |
|---|---|---|
| D-a | Adopt OBS (settled empirical terminal)? | open |
| D-b | Adopt HYP (testable-proposition bridge)? | open |
| D-c | Risk-as-knowledge (lighter-weight RSK analog)? | open |
| D-d | Relation vocabulary for new kinds (confirmed_by, etc.)? | open |
| D-e | Sequencing with RFC-008 (taxonomy before gating)? | leaning: taxonomy first |

## Boot order for a new agent

1. Read this file
2. Read `rfc/009/rfc-009.md` (the full RFC body)
3. Read `rfc/009/edge-validation.md` (current relation rules snapshot)
4. Skim the code touchpoints listed above in order: `kinds.rs` → `knowledge.rs` → `integrity.rs` → `relation.rs` RELATION_RULES
5. Check RFC-008 and RFC-003 for dependency context
6. Route via `/route` — an RFC adoption is a Revision (ADR-013: governance change → REV)
