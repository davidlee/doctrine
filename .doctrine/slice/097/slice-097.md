# SL-097 — Build IMP-006 transactional supersede verb for knowledge records and wire RECORD Supersedes LifecycleOnly rule row

## Context

**SL-096** shipped the knowledge-record entity surface (SPEC-019). SPEC-019 **FR-006**
(supersession) was explicitly deferred, gated on the unbuilt IMP-006 transactional
supersede verb. **SL-062** built the ADR-first supersede verb (the `doctrine supersede`
command) but scoped it to ADR only — `adr::supersede_policy()` returns `None` for
every non-ADR kind.

**IMP-093** captures this gap as a backlog improvement: build the IMP-006 supersede
verb for knowledge records, and wire the RECORD `Supersedes` `LifecycleOnly` rule row.
**IMP-051** (triaged) is the same scope but at the spec level; this slice is the
concrete implementation.

The templates currently seed no `[relationships]` block — the four knowledge record
`.toml` templates end at `[evidence]`. The supersede verb needs `supersedes` (NEW) and
`superseded_by` (OLD, the ADR-004 §5 carve-out) arrays seeded as empty typed storage,
above the `[[relation]]` rows per the SPEC-018 F1 ordering invariant.

No knowledge records exist yet (`.doctrine/knowledge/` is empty), so there is no
migration burden.

## Scope & Objectives

1. **Add `[relationships]` block to the four knowledge record templates.**
   Each template (`knowledge-assumption.toml`, `knowledge-decision.toml`,
   `knowledge-question.toml`, `knowledge-constraint.toml`) gains a seeded empty
   `[relationships]` block between `[evidence]` and EOF, containing:
   ```toml
   [relationships]
   supersedes    = []   # record ids this one replaces (set by doctrine supersede)
   superseded_by = []   # set on superseded predecessor (verb-written, ADR-004 §5)
   ```

2. **Extract `supersede_policy()` to `src/supersede.rs` and cover record kinds.**
   Currently `adr::supersede_policy()` returns `Some` only for `ADR`. Extract
   `SupersedePolicy` + `supersede_policy()` into a new leaf module
   `src/supersede.rs`, then add arms for `ASM`, `DEC`, `QUE`, `CON` — each
   returning a `SupersedePolicy` with the same field names and a kind-appropriate
   terminal status:
   - `assumption` → `obsolete` (catch-all terminal)
   - `question` → `obsolete` (catch-all terminal)
   - `decision` → `superseded` (explicit)
   - `constraint` → `superseded` (explicit)

3. **Generalize `run_supersede()` to handle knowledge records.**
   The existing verb enforces same-kind. For records, cross-kind supersession is
   allowed per PRD-010 §6 (the matrix). ADR stays same-kind-only. Terminal-status
   flip is conditional for records: only non-terminal predecessors are flipped;
   already-terminal records (e.g. `validated`) stay as-is. `is_record_kind` and
   `is_terminal_for_kind` delegate to `knowledge::RecordKind` — single source of truth.
   The §6 matrix:
   | Predecessor  | May be superseded by                     |
   | ------------ | ---------------------------------------- |
   | `assumption` | assumption, decision, constraint         |
   | `question`   | question, decision, constraint, assumption |
   | `decision`   | decision, constraint                     |
   | `constraint` | constraint, decision                     |
   
   The verb must:
   - Resolve both NEW and OLD's `RecordKind` from their prefixes.
   - Validate the pair against the §6 matrix; refuse reopening directions.
   - Move OLD to a kind-appropriate terminal status (from the policy's
     `superseded_status`).
   - Co-write `supersedes` on NEW and `superseded_by` on OLD — the same
     parse-once/hold-both/write-once transactional pattern as the existing ADR verb.

4. **Add RECORD-sourced `Supersedes` `LifecycleOnly` rule row to `RELATION_RULES`.**
   A new row: sources `RECORD` (ASM/DEC/QUE/CON), label `Supersedes`, target
   `TargetSpec::Kinds(RECORD)`, `Tier::One`, `LinkPolicy::LifecycleOnly`. The F-1
   scaffolding pre-flight and the storage-excluded tier (typed `[relationships]`,
   not `[[relation]]`) reuse the existing governance pattern. The exact-coverage
   invariant test extends to cover the record source kinds.

## Non-Goals

- **No `spawns` or record→backlog relate label** — those are SPEC-019 FR-005,
  shipped by SL-096.
- **No POL/STD/slice supersession** — IMP-063 owns expanding `superseded` vocab
  beyond governance; this slice touches only ADR (existing) + the four record kinds.
- **No record migration** — no records exist; nothing to migrate.
- **No cross-kind supersession for ADR** — ADR stays same-kind-only.
- **No tier-1 `[[relation]]` storage for record supersession** — the pair stays
  in the typed `[relationships]` block (LifecycleOnly), matching the governance
  pattern.
- **No `doctrine supersede` CLI changes** — the verb surface (`supersede <NEW> <OLD>`)
  is unchanged; it already dispatches through `supersede_policy()`.
- **No `[[relation]]` storage for record supersession yet** — records use typed
  `[relationships]` (same as ADR today). SL-095 will migrate governance to
  `[[relation]]`; records follow later (tracked as IMP-095 in Follow-Ups below).

## Affected Surface

- `.doctrine/templates/knowledge-*.toml` × 4 — add `[relationships]` block
- `src/supersede.rs` (new) — extracted `SupersedePolicy` + `supersede_policy()` from `adr.rs`, with record arms
- `src/adr.rs` — remove extracted supersede policy code
- `src/main.rs` — generalize `run_supersede()` cross-kind gating and terminal-status
  selection
- `src/relation.rs` — add RECORD `Supersedes` `LifecycleOnly` rule row; extend
  exact-coverage invariant
- `src/relation_graph.rs` (tests) — golden churn for new rule row

## Risks & Assumptions

- **R1 — record template change and green suites.** Adding `[relationships]` to
  templates changes the expected TOML shape; `render_record_toml_seed` tests and
  any golden-captured record shapes must update. Existing `knowledge` tests must
  stay green.
- **R2 — (resolved) `supersede_policy` home.** Extracted to `src/supersede.rs`
  per D1. Behaviour-preservation of existing ADR suite is mandatory.
- **R3 — behaviour preservation.** The ADR supersede path must stay green,
  unchanged, with the same guards (self-edge, same-kind, F-1, F-D, idempotency),
  same CLI surface, and same output.
- **R4 — exact-coverage invariant churn.** The new rule row changes the
  RELATION_RULES table; golden tests in `relation.rs` and `relation_graph.rs`
  that enumerate labels or sources must update.
- **A1:** No knowledge records exist in `.doctrine/knowledge/` — confirmed empty.
- **A2:** The §6 matrix is the authoritative boundary from PRD-010; no additional
  governance decisions are needed.

## Verification / Closure Intent

- `doctrine supersede DEC-001 DEC-002` (same-kind) succeeds, DEC-002 → `superseded`
- `doctrine supersede CON-001 ASM-001` (assumption → constraint, §6 allowed) succeeds
- `doctrine supersede ASM-001 DEC-001` (decision → assumption, reopening) is refused
- `doctrine supersede QUE-001 DEC-001` (decision → question, reopening) is refused
- `doctrine supersede ADR-001 DEC-001` (cross-family) is refused
- `doctrine supersede ASM-001 ASM-001` (self-supersession) is refused
- `doctrine supersede DEC-001 ADR-001` (cross-family) is refused
- `doctrine link ASM-001 supersedes DEC-001` (link, not supersede verb) is refused
  (LifecycleOnly)
- `doctrine validate` reports no supersession drift
- RELATION_RULES exact-coverage invariant extended and green
- Existing ADR supersede tests green unchanged
- `just gate` clean

## Summary

Two small, coupled changes: seed `[relationships]` in record templates so the verb
has a stable attachment point, then extend the existing ADR-first supersede verb to
accept the four record kinds with cross-kind gating per the §6 matrix. The RECORD
`Supersedes` rule row closes the table gap. No records exist, no migration needed,
no CLI surface change — just widening the policy gate and relaxing the same-kind
guard for records, bounded by the matrix.

## Follow-Ups

- **IMP-063** — POL/STD/slice supersession (grow `superseded` vocab beyond governance)
- **IMP-047** — trinary actionability / record gating
- **IMP-095** — migrate record `Supersedes` from typed `[relationships]` to
  `[[relation]]` + typed `superseded_by` carve-out, following the governance pattern
  once SL-095 lands. Until then, records' typed storage is consistent with ADR's and
  valid under ADR-010 D4 (verb-written LifecycleOnly).
