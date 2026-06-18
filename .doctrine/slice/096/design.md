# SL-096 Design: Knowledge-record relation seam (SPEC-019 FR-005)

## Context

SL-059 shipped Slice A — the standalone entity surface: four `RecordKind`s on the
engine with per-kind lifecycles, typed facets, evidence, prefix→kind resolution,
priority partition entries, and the `doctrine knowledge` CLI. Relation edges were
a stub (`outbound_for` returning `Ok(vec![])`); RELATION_RULES carried no record
rules.

This slice (Slice B) delivers SPEC-019 **FR-005**: the outbound relation and
spawn-work seam over the cross-corpus relation contract. Supersession (FR-006)
is deferred — gated on the unbuilt IMP-006 transactional verb.

The companion semantic analysis amended SPEC-019 D6: source-set extensions are
rejected for `specs`/`slices`/`requirements`/`drift` because the inbound
rendering would lie (a slice showing `slices: [ASM-001]` reads as "this record
sliced this slice"). Instead, one epistemic label `shapes` covers record→artefact
relations with a semantically honest inbound `shaped_by`.

## Decisions

### D1 — Mint `Shapes` and `Spawns`; extend `GovernedBy` only

Two new `RelationLabel` variants:

- **`Shapes`** — epistemic influence from a record to any artefact in the
  epistemic-impact set (D2). Inbound: `shaped_by`. Target: explicit `Kinds(...)`
  set (D2). Covers all record→artefact influence (spec, slice, requirement,
  backlog item, governance, other records). Replaces the rejected source-set
  extensions for `specs`/`slices`/`requirements`.

  `Shapes` vs `GovernedBy` on governance targets:
  - Use `governed_by` when the record is **constrained by** an ADR/policy/standard
    (e.g. `CON-002 governed_by ADR-004` — the ADR binds the record).
  - Use `shapes` when the record **influenced, motivated, contextualized, or bore
    on** the governance artefact (e.g. `ASM-001 shapes ADR-012` — the record's
    truth shaped the ADR's content).

  Record↔record uses `shapes` for epistemic influence between records
  (`ASM-001 shapes DEC-003` — the assumption informed the decision).
  Use `supersedes` only for authoritative replacement (FR-006, deferred) —
  `shapes` is never a substitute for the supersession lineage.

- **`Spawns`** — work creation: the record generated this backlog item. Inbound:
  `spawned_by`. Target: `Kinds(&[ISS, IMP, CHR, RSK, IDE])`. Distinct from
  `shapes` — spawning is origin, not epistemic influence. A record may both
  shape and spawn the same backlog item.

One source-set extension (where inbound semantics are honest):

- **`GovernedBy`** — add `ASM, DEC, QUE, CON` to sources. Inbound `governs` is
  semantically honest: an ADR governs a record. Target set unchanged.

`Drift` is **not** extended — the semantic is unclear (what does it mean for
  a knowledge record to "drift"?) and there is no concrete v1 use case. Keeping
  the label out preserves the semantic-honesty contract. A record that needs to
  cite drift can do so via `shapes` pointing at the relevant entity or via
  free-text in the record body.

Record↔record relations use `shapes` with the record kind targets, not separate
`informs`/`bears_on` labels (IMP-053 collapsed). The record kind already carries
epistemic posture.

### D2 — `Shapes` target: explicit epistemic-impact set, not `AnyNumbered`

```rust
TargetSpec::Kinds(&[
    PRD, SPEC, REQ,          // product/spec truth + requirements
    SLICE,                   // execution slices
    ISS, IMP, CHR, RSK, IDE, // backlog work/risk/change/design
    ADR, POL, STD,           // governance artefacts
    ASM, DEC, QUE, CON,      // other knowledge records
])
```

Excludes `REV` (revisions are change acts, not shaped artefacts), `REC`
(reconciliation records are act records, not knowledge objects), `RV` (reviews
are process records), `CM` (concept maps are associations). New numbered kinds
must opt into `shapes` deliberately.

### D3 — `KnowledgeRecord` carries tier1 edges; `show` and `relation_edges` share one read path

`KnowledgeRecord` gains a `tier1: Vec<RelationEdge>` field, mirroring
`BacklogItem::tier1`. `read_record` populates it after `validate` via
`crate::relation::tier1_edges(kind.kind(), &text)`.

The `relation_edges` accessor delegates to `read_record` (one read path — DRY):

```rust
pub(crate) fn relation_edges(root: &Path, kind: RecordKind, id: u32) -> anyhow::Result<Vec<RelationEdge>> {
    let record = read_record(root, kind, id)?;
    Ok(record.tier1)
}
```

The `outbound_for` dispatch resolves `RecordKind` from the prefix and delegates
to `relation_edges`.

`format_show` renders each relation axis via `targets_for(&record.tier1, label)`:

```text
shapes: [SPEC-004, SL-046]
spawns: [ISS-001]
governed_by: [ADR-004]
```

An axis with no edges is silent (the read-tolerant empty-axis convention).
`show_json` projects the same axes into the JSON `relationships` object.

### D4 — No template changes needed; `[evidence]` is the last content

The existing templates end with the `[evidence]` block — a typed table, no
`[[relation]]` array. This satisfies the F1 invariant (typed tables before
`[[relation]]` arrays) trivially: there is no trailing content after `[evidence]`.

`append_edge` creates the `[[relation]]` array-of-tables on first `link`, appending
it after `[evidence]` at EOF. No template comment is seeded — `toml_edit`'s
preservation of trailing comments without a following key is unreliable across
versions, and a seeded empty `[[relation]]` row with `target = ""` would parse as
a legal-but-broken edge. Clean start: no `[[relation]]` block until the first
`doctrine link`.

### D5 — Supersession deferred to FR-006 / IMP-006

The `Supersedes` `LifecycleOnly` RECORD rule row, the cross-kind matrix, and
the transactional supersede verb are out of scope. Backlogged with a priority
boost alongside IMP-006.

## Code impact

| File | Change |
|---|---|
| `src/relation.rs` | 2 new `RelationLabel` variants (`Shapes`, `Spawns`); `name()`/`from_name()` arms; 2 new `RELATION_RULES` rows; extend `GovernedBy` source; local kind aliases for ASM/DEC/QUE/CON; RECORD source-group const |
| `src/knowledge.rs` | Add `tier1` field to `KnowledgeRecord`; read it in `read_record`; new `relation_edges` accessor; extend `format_show` and `show_json` to render relation axes |
| `src/catalog/scan.rs` | Swap `outbound_for` empty arm for `knowledge::relation_edges` call; remove `clippy::match_same_arms` expect |
| `templates/knowledge-*.toml` | No change — `[evidence]` ends the file; `append_edge` creates `[[relation]]` on first `link` |
| `src/relation_graph.rs` (tests) | Update `knowledge_kinds_author_no_outbound_never_panic` (→ records now DO author edges); extend exact-coverage invariant for RECORD source group |
| `src/relation.rs` (tests) | `distinct_labels` goldens; `from_name` round-trip for new labels; target-kind refusal for `Spawns` non-backlog target |
| `src/knowledge.rs` (tests) | VT: seeded record has empty edges; VT: authored `[[relation]]` rows read back; VT: illegal rows classified as `IllegalRow` |

## Verification alignment

- FR-005 (REQ-243): `doctrine link ASM-001 shapes SPEC-004` succeeds;
  `doctrine link DEC-001 spawns ISS-001` succeeds; `doctrine inspect SPEC-004`
  shows `shaped_by: [ASM-001]`; `doctrine inspect ISS-001` shows
  `spawned_by: [DEC-001]`
- `doctrine link ASM-001 shapes RV-001` refused (not in target set)
- `doctrine link ASM-001 spawns SL-001` refused (spawns targets only backlog)
- `doctrine link ASM-001 governed_by ADR-004` succeeds
- `doctrine inspect ADR-004` shows `governs: [ASM-001]` (proving neutral inbound)
- Exact-coverage invariant extended and green
- Existing suites green; `just gate` clean
- `doctrine knowledge show ASM-001` renders `shapes`, `spawns`, `governed_by`
  edges when present

## Risks

- **R1 — `distinct_labels` golden churn.** Two new labels change the
  `RELATION_RULES` distinct-label order. Update the golden — one-shot.
- **R2 — `read_block` source-kind check.** `knowledge::relation_edges` passes
  `kind.kind()` (the engine `Kind` for the record kind) to `read_block`. This
  is the same pattern backlog uses. The `lookup` call matches on `kind.prefix`,
  which is correct for ASM/DEC/QUE/CON.
- **R3 — SL-059 VT-1 test rename.** `knowledge_kinds_author_no_outbound_never_panic`
  becomes `knowledge_kinds_author_outbound_edges`. The existing test asserts
  `edges.is_empty()` — replace with a fixture that authors `[[relation]]` rows
  and asserts they round-trip through `outbound_for`.
- **R4 — template comment vs `toml_edit`.** A `# [[relation]]` comment at EOF
  could become dangling trivia and be lost or displaced by `toml_edit`'s
  `DocumentMut` round-trip. **Mitigated (D4):** no comment; let `append_edge`
  create the array on first use.
- **R5 — behaviour preservation.** The entity engine, existing relation machinery,
  and all per-kind suites (slice, ADR, spec, backlog, memory) must stay green
  unchanged. Only additive changes.

### D6 — `unlink` requires no new mechanism

The `link`/`unlink` verb is already shipped (SL-048). `unlink` removes a
`[[relation]]` row by matching `(label, target)` via `remove_edge` — the same
mechanism for every tier-1 kind. No record-specific `unlink` work is needed.
The knowledge `show`/`inspect` renderer already reads `[[relation]]` at read
time, so an unlinked edge disappears from both surfaces.

## Open questions

- **OQ-1 — IMP-006/FR-006 follow-up.** Supersession backlog item created.
  Priority boosted alongside IMP-006. The `Supersedes` RECORD `LifecycleOnly`
  rule row, cross-kind matrix, and transactional verb land in that slice.
