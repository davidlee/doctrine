# Wire concept-map contextualizes edges into catalog scan

## Context

`RELATION_RULES` (relation.rs:370-378) declares `contextualizes` as
`Tier::One`, `LinkPolicy::Writable` for source kind `CM`. `doctrine link
CM-001 contextualizes SL-047` writes to `[[relation]]` in the TOML
successfully. But `outbound_for` in `catalog/scan.rs:53` returns
`Ok(Vec::new())` for `CM` — every read path (`inspect`, `catalog graph`,
`validate_relations`) is blind to these edges.

The comment at scan.rs:53 says "CM authors no outbound relations" — which
contradicts the RELATION_RULES entry. Every other governed kind (ADR, POL,
STD, RFC) follows the `read_doc → tier1_edges(kind, text)` pattern in
`governance.rs:268-278`. CM is the sole exception.

Additionally, concept maps store their primary edges in a DSL string
(`dsl = """..."""`) — the `[[relation]]` block is a secondary, currently
dead storage path.

## Scope & Objectives

### Fix

**Option A: Wire CM into the scan** (chosen — honours the RELATION_RULES intent).

1. Add `concept_map::relation_edges` function following the governance pattern:
   read TOML text → `RelationDoc::from_toml` → `tier1_edges`
2. Wire CM into `outbound_for`: replace `"REQ" | "CM" => Ok(Vec::new())`
   with `"CM" => crate::concept_map::relation_edges(root, id)`
3. Add `[[relation]]` scaffold to concept-map install template
4. Test: `link CM-001 contextualizes SL-047` → `inspect CM-001` shows edge
   → `catalog graph` includes it

### Option explicitly NOT chosen

- Option B (remove RELATION_RULES entry): destroys the writability semantic.
  Option A preserves the declared contract.
- Option C (hybrid DSL bridge): more invasive — DSL edges would need
  migration or dual-read. Deferred; this slice only fixes the
  `[[relation]]` path.

## Non-Goals

- No DSL → `[[relation]]` migration (existing DSL edges stay in DSL only)
- No DSL edge projection into the relation graph (future enhancement)
- No change to `concept_map add` / `concept_map remove` (they continue
  writing to DSL)

## Terrain

| File | Change |
|------|--------|
| `src/concept_map.rs` | Add `relation_edges` function (read TOML, parse `[[relation]]`, return edges) |
| `src/catalog/scan.rs:53` | Replace `"REQ" \| "CM" => Ok(Vec::new())` with separate arms — CM wired, REQ stays empty |
| `install/templates/concept-map.toml` | Add `[[relation]]` scaffold (empty block) |
| `src/concept_map.rs` (ConceptMapDoc) | Ensure `[[relation]]` rows aren't silently dropped on read |

## Dependencies

- None. Pattern exists in `governance.rs:268-278`. Infrastructure exists in
  `relation.rs` (`RelationDoc`, `tier1_edges`, `ReadRelations`).
