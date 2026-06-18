# SL-095 — Implementation plan rationale

## Phase ordering

### PHASE-01 — `related` for slice & backlog

Self-contained, lowest risk, no refactoring. One RELATION_RULES row + test updates.
Done first because it's the smallest working increment and touches the relation table
that PHASE-02 and PHASE-03 compose over. If anything unexpected surfaces in the
exact-coverage invariant or overlay goldens, it surfaces here with minimal blast radius.

Independent of PHASE-02 and PHASE-03.

### PHASE-02 — Migrate governance `supersedes` to `[[relation]]`

The core mechanical change. Removes `supersedes` from the typed `Relationships`
struct and switches every reader (`relation_edges`, `supersession_pair`, `format_show`,
JSON show) to the `[[relation]]` source. The `supersedes` label is already in
RELATION_RULES as `LifecycleOnly` + `Tier::One` — this phase makes the slot live.

Done before PHASE-03 because:
- PHASE-03's verb refactoring needs the reader paths to already target `[[relation]]`
  (otherwise the verb writes to a slot nothing reads)
- The `StorageTarget` enum (PHASE-03) is meaningless before the storage shape changes

The one-time corpus rewrite is cosmetic — the code is forward-tolerant (serde ignores
unknown keys), so a missed file is harmless. Do it after the code changes land and
tests pass.

### PHASE-03 — Extend verb to POL/STD with `[[relation]]` writes

Consummates the migration. Adds `Superseded` to POL/STD status enums, extends
`supersede_policy` (in `src/supersede.rs` — SL-097's extracted home), adds
`StorageTarget` discriminant, and refactors `run_supersede` to dispatch on storage
mechanism. Governance kinds route through `relation::append_edge`; the typed
`superseded_by` carve-out on OLD stays `dep_seq::apply_string_append` (unchanged).

Depends on PHASE-02 for:
- Readers already targeting `[[relation]]`
- `Relationships` struct already cleaned up
- Templates already updated

Gates on SL-097 extraction complete (`src/supersede.rs` exists with ADR+RECORD arms) —
the `.after` edge ensures the extracted module is ready before POL/STD arms land.

**Read/write disconnect window.** Between PHASE-02 (readers on `[[relation]]`) and
PHASE-03 (verb writes to `[[relation]]`), a `doctrine supersede` would write to the
now-removed typed `supersedes` array while readers look at `[[relation]]` — the edge
would be invisible. This is harmless in practice: all governance `supersedes` arrays
are empty, and no supersede operations are expected between phases. The EN-3 criterion
documents this. If a supersede were accidentally run, it would be a no-op (empty array
→ empty array); the edge is simply not observed until PHASE-03's write path is active.

### SL-097 coexistence

SL-095 carries `after = [{ to = "SL-097", rank = 0 }]` — SL-097 lands first,
extracting `SupersedePolicy` + `supersede_policy()` from `adr.rs` into
`src/supersede.rs` with ADR+RECORD arms. PHASE-03 adds POL/STD arms and the
`StorageTarget` discriminant to the already-extracted module. The `.after`
edge is a soft sequence — no hard gate, but the plan assumes the extraction
module exists before PHASE-03 begins.

## Risk sequence

Risk decreases across phases:
- PHASE-01: one row, pure addition, no-behaviour-change for existing labels
- PHASE-02: structural refactoring, but all `supersedes` arrays are empty — no data
  loss risk, no visible output change
- PHASE-03: new status variants + verb dispatch — the only phase with net-new
  user-visible behaviour
