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

### D1 — Mint `Shapes` and `Spawns`; extend `GovernedBy` and `Drift` only

Two new `RelationLabel` variants:

- **`Shapes`** — epistemic influence from a record to any artefact in the
  epistemic-impact set. Inbound: `shaped_by`. Target: explicit `Kinds(...)` set
  (D2). Covers all record→artefact influence (spec, slice, requirement, backlog
  item, governance, other records). Replaces the rejected source-set extensions
  for `specs`/`slices`/`requirements`.

- **`Spawns`** — work creation: the record generated this backlog item. Inbound:
  `spawned_by`. Target: `Kinds(&[ISS, IMP, CHR, RSK, IDE])`. Distinct from
  `shapes` — spawning is origin, not epistemic influence. A record may both
  shape and spawn the same backlog item.

Two source-set extensions (where inbound semantics are neutral):

- **`GovernedBy`** — add `ASM, DEC, QUE, CON` to sources. Inbound `governs` is
  semantically honest: an ADR governs a record. Target set unchanged.

- **`Drift`** — add `ASM, DEC, QUE, CON` to sources. Inbound is irrelevant
  (`Unvalidated` target has no resolved node). Target unchanged.

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

### D3 — `relation_edges` accessor mirrors the backlog pattern

One accessor in `knowledge.rs` serving all four record kinds, discriminated by
`RecordKind`. Reads `record-NNN.toml`, parses `[[relation]]` via the shared
`RelationDoc::parse` + `read_block` pipeline. The `outbound_for` dispatch
resolves `RecordKind` from the prefix and delegates.

### D4 — Templates scaffold a relation comment, not empty rows

Each `templates/knowledge-*.toml` appends after `[evidence]`:

```toml
# [[relation]]
# label = "shapes"
# target = ""
```

The F1 invariant (typed tables before `[[relation]]` arrays) is satisfied:
`[facet]` and `[evidence]` precede the comment. The `link` verb appends real
`[[relation]]` rows at EOF. No seeded `[[relation]]` rows — the `""` target
would parse as a legal but broken edge.

### D5 — Supersession deferred to FR-006 / IMP-006

The `Supersedes` `LifecycleOnly` RECORD rule row, the cross-kind matrix, and
the transactional supersede verb are out of scope. Backlogged with a priority
boost alongside IMP-006.

## Code impact

| File | Change |
|---|---|
| `src/relation.rs` | 2 new `RelationLabel` variants (`Shapes`, `Spawns`); `name()`/`from_name()` arms; 2 new `RELATION_RULES` rows; extend `GovernedBy`/`Drift` sources; local kind aliases for ASM/DEC/QUE/CON; RECORD source-group const |
| `src/knowledge.rs` | New `relation_edges(root, kind, id)` accessor |
| `src/catalog/scan.rs` | Swap `outbound_for` empty arm for `knowledge::relation_edges` call; remove `clippy::match_same_arms` expect |
| `templates/knowledge-*.toml` | Add `# [[relation]]` comment footer after `[evidence]` |
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
- Exact-coverage invariant extended and green
- Existing suites green; `just gate` clean
- `doctrine knowledge show ASM-001` renders `shapes`, `spawns`, `governed_by`,
  `drift` edges when present

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
- **R4 — behaviour preservation.** The entity engine, existing relation machinery,
  and all per-kind suites (slice, ADR, spec, backlog, memory) must stay green
  unchanged. Only additive changes.

## Open questions

- **OQ-1 — IMP-006/FR-006 follow-up.** Supersession backlog item created.
  Priority boosted alongside IMP-006. The `Supersedes` RECORD `LifecycleOnly`
  rule row, cross-kind matrix, and transactional verb land in that slice.
- **OQ-2 — `drift` extension semantics.** A record linking to drift with
  `drift` label — what does it mean? A record contributing to a drift
  observation? Out of v1 scope; the extension is mechanically sound
  (Unvalidated target, no inbound rendering) and allows forward use.
