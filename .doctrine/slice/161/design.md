# Design SL-161: DRY kind-registry seam

> Source: **IMP-184** (record-kind prefix-literal scatter) + the numbered-kind
> identity-table scatter (`mem.pattern.entity.numbered-kind-identity-table`).
> Decisions locked with the user in the `/design` pass (2026-06-27).

## 1. Design Problem

Two scattered-identity problems share a root cause — membership is hand-maintained
at consumption sites instead of reading from a single registry:

**A. Record-kind prefix literals (~12 sites).** `kinds::RECORD = &[ASM, DEC, QUE,
CON]` exists but ~12 sites hardcode the same four prefixes in match arms and array
literals. Adding/renaming a kind is a ~12-site grep-and-edit with no drift canary
on the literal match-arm sites. The panic-grade site is `catalog/scan.rs:62` —
`debug_assert!(false)` fallthrough panics every debug-build corpus scan when a
KINDS row lacks a scan arm.

**B. Numbered-kind identity table (advisory membership).** `integrity::KINDS` (21
rows) is hand-maintained; a missing `KindRef` row silently escapes `validate`. The
test pin is a separate literal list that must be updated in tandem. A missing row
is undetected.

These are structural twins. The fix: make every consumer read from one source.

## 2. Current State

- `kinds::RECORD` (`kinds.rs:37`) — the canonical 4-kind membership const. Only
  `relation.rs` uses it today (for `RELATION_RULES` source-sets).
- `RecordKind::ALL` + `RecordKind::from_prefix()` (`knowledge.rs`) — enum-based
  membership, used by `scan.rs` and `supersede.rs`.
- ~12 sites hardcode `"ASM"|"DEC"|"QUE"|"CON"` directly.
- `integrity::KINDS` — 21 hand-written `KindRef` rows + separate test pin.
- The backlog family (`kinds::BACKLOG` + `ItemKind::ALL` + `kind_from_prefix`)
  uses the same two-source pattern — no coherency test. This design follows that
  convention.

## 3. Forces & Constraints

- **ADR-001 layering:** `kinds.rs` is leaf; `knowledge.rs`/`integrity.rs` are
  engine. No cycles introduced — `is_record` reads `RECORD` (leaf), callers are
  engine/command.
- **Behaviour-preservation gate:** existing suites green. Pin tests that
  hardcoded record lists are updated to derive from `kinds::RECORD`, not deleted.
- **No new behaviour:** this is a mechanical DRY — no new kinds, no new CLI, no
  new edges.
- **Backlog convention:** follow the existing two-source pattern (`kinds::BACKLOG`
  for grouping + `ItemKind::ALL` for enum dispatch) rather than invent a new one.

## 4. Guiding Principles

- `kinds::RECORD` is the grouping source of truth; `RecordKind::ALL` is the enum
  dispatch source. Both are "the definition" — same as backlog.
- Predicate for boolean checks, direct iteration for collection contexts.
- `scan.rs` restructure follows the backlog pattern already in the `other` arm.

## 5. Proposed Design

### 5.1 `kinds.rs` — `is_record` predicate

Add to `src/kinds.rs`:

```rust
/// Membership predicate over [`RECORD`] — the single source for "is this a
/// knowledge-record kind?" so adding/renaming a record kind edits RECORD,
/// not every call site.
pub(crate) fn is_record(prefix: &str) -> bool {
    RECORD.contains(&prefix)
}
```

Leaf tier, zero imports. The existing `groupings_match_documented_membership`
test already pins `RECORD` content — no new test needed.

### 5.2 Site conversions

#### `dep_seq.rs` — delegate to `kinds::is_record`

**`is_record()` at line 29:**
```rust
pub(crate) fn is_record(kind: &'static crate::entity::Kind) -> bool {
    crate::kinds::is_record(kind.prefix)
}
```

**Test at `:264-275`:** Replace the hardcoded `matches!` filter with an assertion
that the predicate's result over `integrity::KINDS` equals `kinds::RECORD` sorted:
```rust
fn is_record_predicate_matches_kinds_record() {
    let mut from_pred: Vec<&str> = crate::integrity::KINDS
        .iter()
        .filter(|k| is_record(k.kind))
        .map(|k| k.kind.prefix)
        .collect();
    from_pred.sort_unstable();
    let mut want: Vec<&str> = crate::kinds::RECORD.to_vec();
    want.sort_unstable();
    assert_eq!(from_pred, want);
}
```

The `is_admissible_dep_target` function and the admissible-vector test at `:285`
continue to work — `is_record` now delegates to `kinds::RECORD` which contains
the same 4 prefixes.

#### `partition.rs:609` — record-row guard

```rust
// Before:
if ["ASM", "DEC", "QUE", "CON"].contains(&p.prefix) {

// After:
if crate::kinds::is_record(p.prefix) {
```

#### `scan.rs:62-68` — dispatch restructure

Remove the `"ASM" | "DEC" | "QUE" | "CON"` arm. Add record dispatch as a guard
in the `other` arm, before the backlog check:

```rust
other => {
    if let Some(record_kind) = crate::knowledge::RecordKind::from_prefix(other) {
        crate::knowledge::relation_edges(root, record_kind, id)
    } else if let Some(item_kind) = crate::backlog::kind_from_prefix(other) {
        crate::backlog::relation_edges(root, item_kind, id)
    } else {
        debug_assert!(false, "outbound_for: unrouted KINDS prefix `{other}`");
        Ok(Vec::new())
    }
}
```

`RecordKind::from_prefix` already gates membership. Fallthrough panic preserved.
This mirrors how backlog dispatch already works in the same arm.

#### `search.rs:33` — knowledge group alias

```rust
// Before:
("knowledge", &["ASM", "DEC", "QUE", "CON"]),

// After:
("knowledge", kinds::RECORD),
```

`DEFAULT_SEARCH_KINDS` (`:25`) and the `"all"` alias (`:38`) are full-corpus
lists — not the record subset. Left as-is.

#### `tag.rs` — subset inclusion test

`TAGGABLE` is a full-corpus `const &[&str]`. Record prefixes can't be extracted
from it at compile time. **Add a test:**

```rust
#[test]
fn record_kinds_are_taggable() {
    for prefix in kinds::RECORD {
        assert!(TAGGABLE.contains(prefix), "{prefix} missing from TAGGABLE");
    }
}
```

#### `relation.rs:1444-1445` — Shapes / Spawns census

```rust
// Before:
(RelationLabel::Shapes, &["ASM", "DEC", "QUE", "CON"]),
(RelationLabel::Spawns, &["ASM", "DEC", "QUE", "CON"]),

// After:
(RelationLabel::Shapes, kinds::RECORD),
(RelationLabel::Spawns, kinds::RECORD),
```

Lines `1422` (`References`) and `1427` (`Supersedes`) embed record prefixes in
larger mixed sets (SL+RFC+BACKLOG+RECORD, SL+GOV+RECORD). Left as-is — not the
record-only subset.

#### `catalog/test_helpers.rs:119` — `seed_knowledge`

```rust
// Before:
let kind_dir = match prefix {
    "ASM" => "assumption",
    "DEC" => "decision",
    "QUE" => "question",
    "CON" => "constraint",
    other => panic!("unknown knowledge prefix: {other}"),
};

// After:
let record_kind = crate::knowledge::RecordKind::from_prefix(prefix)
    .unwrap_or_else(|| panic!("unknown knowledge prefix: {prefix}"));
let kind_dir = record_kind.as_str();
```

#### `supersede.rs` / `superserde.rs`

Already use `RecordKind::from_prefix(prefix).is_some()` — no change. Verified as
a verification step.

### 5.3 Part B — KINDS count assertion

**`integrity.rs` — comment on KINDS:**
```rust
/// When adding a numbered kind, add its `KindRef` row here and bump the count
/// in `kinds_table_covers_the_numbered_kinds`.
```

**Test addition** (`kinds_table_covers_the_numbered_kinds`):
```rust
assert_eq!(KINDS.len(), 21, "add/remove a KindRef row? bump this count");
```

Two edits in one file. The count assertion catches both missing and extra rows.
No macro — the 21-row table in one file with a co-located test doesn't warrant
macro ceremony.

### 5.4 What stays as-is

| Site | Why |
|---|---|
| `search.rs:25,38` | Full-corpus lists, not record subset |
| `tag.rs:17` TAGGABLE literal content | Full-corpus `const`; subset test added instead |
| `relation.rs:1422,1427` | Mixed supersets, not record-only |
| `integrity.rs:817` KINDS test prefix pin | Self-validating census of ALL kinds via KINDS |
| `kinds::GOV`, `kinds::BACKLOG` | Out of scope |
| `scan.rs` ADR/POL/STD singletons | Each calls `governance::relation_edges` with different `GovKind` param — not a uniform group |

## 6. Verification / Test Impact

### Tests that change (by design)

| Test | Change |
|---|---|
| `dep_seq::is_record_is_exactly_knowledge_records` | Replaced: assert predicate over KINDS == `kinds::RECORD` sorted |
| `partition::non_knowledge_rows_have_empty_gating` | Guard reads `kinds::is_record()` instead of literal array |
| `relation::sources_match_shipped_accessors` | Shapes/Spawns rows read `kinds::RECORD` |
| `integrity::kinds_table_covers_the_numbered_kinds` | Added `KINDS.len()` count assertion |
| `tag::record_kinds_are_taggable` | New test |

### Tests that stay green (behaviour preservation)

- All knowledge-record tests (create, status, supersede, show, list)
- All scan/catalog tests (corpus scan, relation graph)
- All search/tag tests (search finds records, tag sets/clears on records)
- `just gate` zero warnings

### Post-change grep

- `"ASM".*"DEC".*"QUE".*"CON"` literal clusters in `src/` → zero (except
  `kinds.rs` RECORD definition + its pin test)
- `RecordKind::from_prefix` call sites → unchanged (already DRY)

## 7. Risks & Mitigations

- **R1 — `scan.rs` restructure changes dispatch order.**
  *Mitigation:* the guard fires only when no explicit singleton arm matched;
  behaviour identical for the record family. Existing scan tests gate this.

- **R2 — `dep_seq` test rewrite accidentally narrows coverage.**
  *Mitigation:* new test asserts predicate output == `kinds::RECORD` over all
  KINDS — stronger than the old pin (old test asserted against hardcoded subset;
  new test asserts set equality).

- **R3 — `seed_knowledge` panic message changes.**
  *Mitigation:* test-only helper; panics on unknown prefix either way. Behaviour
  unchanged for valid prefixes.

## 8. Open Questions

None remaining — all resolved in `/design` pass.

## 9. Review Notes

### Internal adversarial pass (2026-06-27)

**F1 (MINOR) — `dep_seq` test circularity concern, benign.**
The rewritten test filters KINDS with `is_record` and asserts == `kinds::RECORD`.
Since `is_record` delegates to `kinds::RECORD`, this is circular in the forward
direction. But the reverse direction is useful: it catches a prefix that IS in
KINDS (as a record kind) but NOT in `kinds::RECORD` — a false positive. The test
validates KINDS↔RECORD set equality. Benign.

**F2 (MINOR) — `dep_seq.rs:285` admissible vector still hardcodes records.**
The test assertion at :285 lists `"ASM", "DEC", "QUE", "CON"` inline as part
of the admissible-target census. Not DRY'd because the vector is work-like ∪
record — a superset — and the work-like half (`SL, ISS, IMP, CHR, RSK, IDE,
REV`) isn't in scope. The record subset could be derived from `kinds::RECORD`
but the resulting code would be less readable for what it tests (the full
admissible census). Accepted as-is.

**F3 (VERIFIED) — `supersede.rs:50,57` not a group-membership site.**
The `supersede_policy` match groups `ASM|QUE` (one policy) and `DEC|CON`
(another) — per-kind-pair dispatch, not record-group membership. Out of scope.
A future `RecordKind`-based refactor could clean this up.

**F4 (VERIFIED) — `relation_graph.rs` has no separate record dispatch.**
`outbound_for` is re-exported from `catalog::scan`; `dep_seq_for` has no record
arm (records don't author dep/seq). No additional sites to DRY.

**F5 (VERIFIED) — `scan.rs` comment about Slice A/B is obsolete.**
The comment on the record arm ("Kept a SEPARATE arm from REQ … Slice B swaps
it") describes historical phasing that already landed. Removed with the arm —
no value in preserving it.

**F6 (VERIFIED) — no other `outbound_for`-style dispatch functions.**
`dep_seq_for` (`relation_graph.rs`) routes SL, REV, and backlog — no record
arm. The record family only appears in `outbound_for`.

No findings overturned any design decision. All integrated.
