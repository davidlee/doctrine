# SL-095 — Bed in remaining relation gaps

## Design decisions

### D1 — `related` for slice & backlog: one RELATION_RULES row

Add one row to `RELATION_RULES`, sharing the existing `RelationLabel::Related` slot:

```rust
RelationRule {
    sources: &[SLICE, ISS, IMP, CHR, RSK, IDE],
    label: RelationLabel::Related,
    inbound_name: "related",
    target: TargetSpec::AnyNumbered,
    tier: Tier::One,
    link: LinkPolicy::Writable,
},
```

The existing GOV `Related` row (`sources: GOV`, `target: SameKind`) stays — two rows
at one slot, same as `Supersedes` already has (SL→SL + GOV→GOV). `lookup` disambiguates
by `(source ∈ sources, label)`.

**Target: `AnyNumbered`** — target must resolve to a real entity in `integrity::KINDS`.
Free text is refused. Enables `doctrine link SL-X related ADR-Y` and `doctrine link
IMP-X related SPEC-Y`. Inbound name is `"related"` for both directions (the label is
inherently symmetric — "can't be an only child's brother").

**No template or code changes** beyond the one table row + test updates. Slices and
backlog items already have `[[relation]]` blocks from SL-048.

### D2 — Migrate governance `supersedes` typed → `[[relation]]`: no runtime migrator

The migration is a **one-time corpus rewrite**, not shipped code. All governance
`supersedes` arrays are empty (confirmed: 13 ADR + 1 POL), so the migrator reduces to
removing the `supersedes = []   # ...` line from each `.toml` and updating the comment.

**The migration is cosmetic, not a correctness requirement.** Serde ignores unknown
fields by default — `Relationships` has no `deny_unknown_fields`. A pre-migration file
still carrying `supersedes = []` parses fine (the key is silently skipped), and
`tier1_edges` reads from `[[relation]]` rows, not the `[relationships]` table. The
code is forward-tolerant: a missed file or a new entity scaffolded from the old
template causes no breakage.

**Rationale:** prove the model works before building a permanent migrator. If
governance supersession is exercised post-migration and the split `[[relation]]` +
typed-carveout pattern proves itself, a proper migrator is a backlog item.

**Storage shape after migration** (governance TOML):

```toml
[relationships]
superseded_by = []   # set by doctrine supersede (verb-written, ADR-004 §5)
tags          = []   # free-form classification

# `related` and `supersedes` are uniform `[[relation]]` rows.
# Author `related` with `doctrine link ADR-NNN related <target>`.
# `supersedes` is LifecycleOnly — written only by `doctrine supersede`.
[[relation]]
label = "related"
target = "ADR-004"
```

The F1 ordering invariant holds: `[relationships]` precedes `[[relation]]` — same shape
as today, just without the `supersedes = []` line.

### D3 — `Relationships` struct drops `supersedes`

`src/governance.rs`:

```rust
struct Relationships {
    // supersedes REMOVED — migrated to [[relation]] (SL-095)
    #[serde(default)]
    superseded_by: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
}
```

`serde(default)` on the remaining fields tolerates a hand-trimmed file that omits one.
JSON surface changes: `relationships.supersedes` disappears from `--json` output.
Zero web/map consumers (confirmed: no references to `supersedes`, `relationships`,
`adr`, `governance` in `web/map/`).

### D4 — Reader switch: `relation_edges`, `supersession_pair`, `format_show`

**`relation_edges`** — remove the typed `supersedes` read; both labels come from
`tier1_edges`:

```rust
pub(crate) fn relation_edges(g: &GovKind, root: &Path, id: u32) -> ... {
    use crate::relation::{RelationEdge, RelationLabel, tier1_edges};
    let (_doc, toml_text, _body) = read_doc(g, &root.join(g.kind.dir), id)?;
    // Both supersedes and related are now [[relation]] rows.
    tier1_edges(&g.kind, &toml_text)
}
```

**`supersession_pair`** — reads `supersedes` from `[[relation]]` instead of the typed
field. MUST use `read_block` directly (the `(edges, illegal)` pair), NOT
`tier1_edges` (edges-only). `tier1_edges` silently drops `IllegalRow`s — a
hand-edited `[[relation]] label="supersedes" target="FREE-TEXT"` would be invisible
to the supersession cross-check, producing a false-clean report. `IllegalRow`s are
still caught by the general `validate` scan, but the cross-check must see them too.

```rust
pub(crate) fn supersession_pair(g: &GovKind, root: &Path, id: u32) -> ... {
    let (_doc, toml_text, _body) = read_doc(g, &root.join(g.kind.dir), id)?;
    let relation_doc = RelationDoc::parse(&toml_text)?;
    let (edges, _illegal) = read_block(&g.kind, &relation_doc);
    let supersedes: Vec<String> = edges
        .iter()
        .filter(|e| e.label == RelationLabel::Supersedes)
        .map(|e| e.target.clone())
        .collect();
    // superseded_by still read from typed Relationships (ADR-004 §5 carve-out).
    // Parse separately — one text read, two consumers (RelationDoc + Doc).
    let doc: Doc = toml::from_str(&toml_text)?;
    Ok((supersedes, doc.relationships.superseded_by))
}
```

**`format_show`** — `supersedes` joins `related` as a caller-supplied `Vec<String>`
read from `targets_for(edges, RelationLabel::Supersedes)`. The render order stays
`supersedes → superseded_by → related → tags` — byte-identical for empty arrays.

### D5 — Templates drop `supersedes` line

Remove from `install/templates/adr.toml`, `install/templates/policy.toml`,
`install/templates/standard.toml`:

```toml
[relationships]
superseded_by = []   # set on superseded predecessor (verb-written, ADR-004 §5)
tags          = []   # free-form classification

# `related` and `supersedes` are uniform `[[relation]]` rows (SL-048/SL-095).
# Author `related` with `doctrine link <ID> related <target>`.
# `supersedes` is LifecycleOnly — written only by `doctrine supersede`.
```

### D6 — Verb: extend to POL/STD with `superseded` status

Add `Superseded` variant to both `PolicyStatus` and `StandardStatus` enums:

```rust
// policy.rs
pub(crate) enum PolicyStatus {
    Draft, Required, Superseded, Deprecated, Retired,
}
// Add "superseded" to POLICY_STATUSES and to is_hidden

// standard.rs
pub(crate) enum StandardStatus {
    Draft, Default, Required, Superseded, Deprecated, Retired,
}
// Add "superseded" to STANDARD_STATUSES and to is_hidden
```

`SupersedePolicy` (extracted to `src/supersede.rs` by SL-097) gains POL and STD arms:

```rust
pub(crate) fn supersede_policy(kind: &Kind) -> Option<SupersedePolicy> {
    match kind.prefix {
        "ADR" | "POL" | "STD" => Some(SupersedePolicy {
            supersedes_field: "supersedes",   // for F-1 pre-flight on typed carve-out
            carveout_field: "superseded_by",
            superseded_status: "superseded",
        }),
        _ => None,
    }
}
```

POL and STD share the same `superseded` status string as ADR.

### D7 — Verb: dispatch write mechanism by kind

The verb currently writes typed arrays (`dep_seq::apply_string_append`). After
migration, governance kinds write `[[relation]]` rows via `relation::append_edge`.
Records (SL-097) keep typed arrays.

`SupersedePolicy` gains a storage discriminant:

```rust
pub(crate) enum StorageTarget {
    /// Write via `relation::append_edge` — governance post-SL-095.
    RelationRow,
    /// Write via `dep_seq::apply_string_append` — records (SL-097).
    TypedArray { field: &'static str },
}

pub(crate) struct SupersedePolicy {
    pub(crate) storage: StorageTarget,
    pub(crate) carveout_field: &'static str,
    pub(crate) superseded_status: &'static str,
}
```

Governance arms:
```rust
"ADR" | "POL" | "STD" => Some(SupersedePolicy {
    storage: StorageTarget::RelationRow,
    carveout_field: "superseded_by",
    superseded_status: "superseded",
}),
```

Records arm (SL-097):
```rust
"ASM" | "DEC" | "QUE" | "CON" => Some(SupersedePolicy {
    storage: StorageTarget::TypedArray { field: "supersedes" },
    carveout_field: "superseded_by",
    superseded_status: kind_specific,
}),
```

**Verb write path** dispatches on `policy.storage`:
- `RelationRow` → `relation::append_edge(&new_path, Supersedes, &old_ref)` — same
  idempotent semantics as `link` (`Noop` = already recorded)
- `TypedArray { field }` → existing `dep_seq::apply_string_append` path (unchanged)

The F-1 pre-flight for `RelationRow` changes: instead of checking a typed array, check
the `[relationships]` table exists (for `superseded_by`) and the file has no F1 trap
(the `trailing_typed_table_after_relation` check already in `append_relation_row`).

The typed `superseded_by` carve-out writes the same way for both paths — it's always
typed, always `dep_seq::apply_string_append`.

### D8 — Verb location: `src/supersede.rs`

SL-097 extracts `SupersedePolicy` + `supersede_policy()` out of `adr.rs` into
`src/supersede.rs`. SL-095's policy changes (POL/STD arms, `StorageTarget`) land there.
The `run_supersede` function stays in `main.rs` for now (extraction of the verb body
is a follow-on, not required for either slice).

## Code impact summary

| File | Change |
|---|---|
| `src/relation.rs` | +1 RELATION_RULES row (slice/backlog `related`); test updates |
| `src/governance.rs` | `Relationships.supersedes` removed; `relation_edges` simplified to `tier1_edges` only; `supersession_pair` reads from `tier1_edges`; `format_show` accepts `supersedes: &[String]` from caller; template comment updates |
| `src/adr.rs` | `supersede_policy` removed (moved to `src/supersede.rs` by SL-097) |
| `src/policy.rs` | `Superseded` added to `PolicyStatus`; STATUSES + hidden updated |
| `src/standard.rs` | `Superseded` added to `StandardStatus`; STATUSES + hidden updated |
| `src/supersede.rs` | (SL-097 creates; SL-095 adds) POL/STD arms + `StorageTarget` + `RelationRow` variant |
| `src/main.rs` | `run_supersede` dispatches on `StorageTarget`; `RelationRow` path uses `relation::append_edge`; imports update |
| `install/templates/adr.toml` | `supersedes = []` line removed; comment updated |
| `install/templates/policy.toml` | `supersedes = []` line removed; comment updated |
| `install/templates/standard.toml` | `supersedes = []` line removed; comment updated |
| `src/relation_graph.rs` (tests) | Golden churn for new `related` rule row |
| `src/relation.rs` (tests) | `lookup(SLICE, Related)` now `Some`; exact-coverage invariant updated |
| Corpus `.doctrine/adr/*/adr-NNN.toml` × 13 | One-time: remove `supersedes = []` line |
| Corpus `.doctrine/policy/001/policy-001.toml` | One-time: remove `supersedes = []` line |

## Verification alignment

- `doctrine link SL-X related ADR-Y` succeeds; `IMP-X related SPEC-Y` succeeds
- `doctrine link SL-X related FREE-TEXT` refused (`AnyNumbered` target)
- `doctrine supersede ADR-NEW ADR-OLD` writes `[[relation]] label="supersedes"` on NEW
- `doctrine supersede POL-NEW POL-OLD` succeeds (new policy arm)
- `doctrine supersede STD-NEW STD-OLD` succeeds (new standard arm)
- Every governance entity round-trips `show` byte-identical across the migration (empty
  arrays → no display line → no churn)
- `show --json` output drops `relationships.supersedes` (empty, no consumers)
- `relation_edges` for governance emits same edges post-migration
- `supersession_pair` returns same (empty) supersedes vector post-migration
- `doctrine validate` reports no supersession drift
- RELATION_RULES exact-coverage invariant updated and green
- Existing suites green; `just gate` clean

## Risks & mitigations

- **R1 — POL/STD `superseded` status variant.** No POL/STD entities exist to test the
  terminal flip on real data. Mitigation: unit test with seeded test TOML.
- **R2 — coexistence with SL-097.** Both slices touch `src/supersede.rs`. SL-095
  `.after` SL-096 but is independent of SL-097. If SL-097 lands first, SL-095 adds
  POL/STD arms + `StorageTarget` to the already-extracted module. If SL-095 lands
  first, SL-097 extracts the policy code (including SL-095's POL/STD arms) to
  `src/supersede.rs`. Either order is mergeable.
- **R3 — `read_doc` double-use.** `supersession_pair` needs both `read_block` (for
  `supersedes`) and `Doc` parse (for typed `superseded_by`). One text read, two
  consumers — `toml::from_str` for the typed struct, `RelationDoc::parse` for
  `read_block`. Acceptable for the validate path (not hot).
- **R4 — F1 guard tightening.** `append_edge` refuses a file whose typed table
  trails `[[relation]]` (F1 violation). The old `dep_seq::apply_string_append` had
  no such guard — a hand-edited file that worked before now gets refused. This is
  a correctness improvement (accepted), not a regression.
