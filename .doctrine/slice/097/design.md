# SL-097 — Design

## Decisions

### D1 — Supersede policy lives in `src/supersede.rs`

Extract `SupersedePolicy` struct and `supersede_policy()` from `adr.rs` into a new
leaf module. `adr.rs` concerns ADR rendering, scaffolding, and status — the
supersession capability gate is cross-kind and belongs in its own single-responsibility
module. Record arms join the existing ADR arm; POL/STD/slice arms join later (IMP-063).

### D2 — Terminal status: leave already-terminal records as-is

For ADR, always flip OLD to `superseded` (existing behaviour, unchanged).
For records, only flip OLD to the kind-appropriate terminal status if OLD is
non-terminal — `knowledge::RecordKind::is_terminal()` is the single source of truth.
An already-`validated` assumption (e.g. hardened into a constraint) stays
`validated`; an `open` question becomes `obsolete`. The superseded terminal mapping
(when a flip IS needed, i.e. OLD is non-terminal):

| Kind | Policy `superseded_status` |
|------|---------------------------|
| assumption | `obsolete` |
| question | `obsolete` |
| decision | `superseded` |
| constraint | `superseded` |

### D3 — Cross-kind gating: §6 matrix for records, same-kind for ADR

`run_supersede()` detects whether both refs are record kinds. If yes: validate against
the PRD-010 §6 matrix. If no: existing same-kind guard (ADR). The matrix is two pure
functions (`is_record_kind`, `validate_matrix`) — no IO, no clock.

### D4 — RECORD Supersedes rule row: `TargetSpec::Kinds(RECORD)`, `LifecycleOnly`

A new `RELATION_RULES` row between the existing RECORD `Shapes`/`Spawns` rows and
`GovernedBy`. Same `inbound_name` (`"superseded by"`), same `Tier::One`, same
`LifecycleOnly` link policy as governance — but `TargetSpec::Kinds(RECORD)` (not
`SameKind`) because records admit cross-kind supersession. The pair stays in typed
`[relationships]` storage, excluded from `[[relation]]` migration.

### D5 — F-1 pre-flight: record templates seed `[relationships]`

The four record templates (`knowledge-{assumption,decision,question,constraint}.toml`)
gain a seeded empty `[relationships]` block between `[evidence]` and EOF, with
`supersedes = []` and `superseded_by = []`. No migration burden — no records exist.

## Target Behaviour

### Happy paths

```
$ doctrine supersede DEC-001 DEC-002
DEC-001 supersedes DEC-002
# DEC-002 → superseded; DEC-001.supersedes += DEC-002; DEC-002.superseded_by += DEC-001

$ doctrine supersede CON-001 ASM-001
CON-001 supersedes ASM-001
# ASM-001 stays validated (already terminal); CON-001.supersedes += ASM-001; ASM-001.superseded_by += CON-001
```

### Error paths

| Command | Refusal reason |
|---------|---------------|
| `supersede ASM-001 DEC-001` | reopening: decision → assumption not in §6 |
| `supersede QUE-001 DEC-001` | reopening: decision → question not in §6 |
| `supersede ASM-001 ASM-001` | self-supersession |
| `supersede DEC-001 ADR-001` | cross-family (DEC is record, ADR is not — same-kind guard for non-record) |
| `supersede ADR-001 DEC-001` | cross-family (ADR is not a record) |
| `link DEC-001 supersedes DEC-002` | LifecycleOnly — refused by `link`, names the `supersede` verb |

## Code Impact

### `src/supersede.rs` (new, ~40 lines)

```rust
//! Cross-kind supersession policy gate consumed by `doctrine supersede`.

use crate::integrity::Kind;

pub(crate) struct SupersedePolicy {
    pub(crate) supersedes_field: &'static str,
    pub(crate) carveout_field: &'static str,
    pub(crate) superseded_status: &'static str,
}

pub(crate) fn supersede_policy(kind: &Kind) -> Option<SupersedePolicy> { ... }
```

Five match arms: `"ADR"`, `"ASM"`, `"DEC"`, `"QUE"`, `"CON"`. All others → `None`.

### `src/adr.rs` (remove ~30 lines)

Remove `SupersedePolicy` struct and `supersede_policy()` function. No other changes.

### `src/main.rs` (modify `run_supersede`, ~+60 lines)

1. `mod supersede;` + call `supersede::supersede_policy()`
2. Replace hard same-kind guard with:
   - Both records → `validate_matrix()`
   - Otherwise → existing same-kind guard
3. Pure helpers (delegate to `knowledge.rs` — single source of truth):
   ```rust
   fn is_record_kind(prefix: &str) -> bool {
       crate::knowledge::RecordKind::from_prefix(prefix).is_ok()
   }
   fn validate_matrix(new_prefix: &str, old_prefix: &str, new: &str, old: &str) -> anyhow::Result<()> { ... }
   // Delegates to knowledge::RecordKind::is_terminal — never hardcodes status strings.
   fn is_terminal_for_kind(prefix: &str, status: &str) -> bool { ... }
   ```
4. Extract F-D idempotency to `check_not_already_superseded()` helper
5. Conditional status flip: only if OLD is non-terminal (preserves `validated`/`answered` nuance)

### `.doctrine/templates/knowledge-*.toml` ×4 (+4 lines each)

Insert after `[evidence]` closing:
```toml
[relationships]
supersedes    = []   # record ids this one replaces (set by doctrine supersede)
superseded_by = []   # set on superseded predecessor (verb-written, ADR-004 §5)
```

### `src/relation.rs` (RELATION_RULES + tests, ~+10 lines)

New rule row between `Spawns` and `GovernedBy`:
```rust
RelationRule {
    sources: RECORD,
    label: RelationLabel::Supersedes,
    inbound_name: "superseded by",
    target: TargetSpec::Kinds(RECORD),
    tier: Tier::One,
    link: LinkPolicy::LifecycleOnly,
},
```

Test updates:
- `distinct_labels_by_source`: add `"ASM", "DEC", "QUE", "CON"` to Supersedes expected
- `target_spec_matches_design`: exclude RECORD sources from `SameKind` assertion

### Test plan (new, in `main.rs` tests)

| Test | What it proves |
|------|---------------|
| `record_supersede_same_kind` | DEC-001 supersedes DEC-002; status flip + edges |
| `record_supersede_cross_kind_allowed` | CON-001 supersedes ASM-001 (assumption→constraint via §6) |
| `record_supersede_reopening_refused` | DEC-001 supersedes ASM-001 is refused (decision→assumption) |
| `record_supersede_question_reopening` | DEC-001 supersedes QUE-001 refused (decision→question) |
| `record_supersede_cross_family_refused` | DEC-001 supersedes ADR-001 refused |
| `record_supersede_self_refused` | ASM-001 supersedes ASM-001 refused |
| `record_supersede_already_terminal_no_flip` | ASM-001 (validated) superseded by CON-001; status stays validated |
| `record_supersede_idempotent` | Re-run writes no changes, prints "already recorded" |
| `link_supersedes_on_record_refused` | `link ASM-001 supersedes DEC-001` refused (LifecycleOnly) |
| `record_supersede_torn_recovery` | Torn state (NEW has edge, OLD doesn't) recovers on re-run |

## Verification Alignment

- `doctrine supersede <RECORD> <RECORD>` succeeds for same-kind + §6 allowed crossings
- `doctrine supersede <RECORD> <RECORD>` refused for reopening directions
- `doctrine supersede <RECORD> <ADR>` refused (cross-family)
- `doctrine link <RECORD> supersedes <RECORD>` refused (LifecycleOnly)
- `doctrine validate` clean (no supersession drift from new rule row)
- RELATION_RULES exact-coverage invariant extended and green
- Existing ADR supersede tests green unchanged
- Record template round-trip tests green (new `[relationships]` block parses)
- `just gate` clean
