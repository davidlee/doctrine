# Design: Wire link/unlink CLI for memory relations

## Current vs Target

**Current**: `doctrine link mem_* <label> <target>` fails with `not a canonical ref` — `resolve_link_path` calls `parse_canonical_ref` which requires `PREFIX-NNN`. No write surface exists for memory relations; `[[relation]]` rows can only be hand-authored.

**Target**: `link`/`unlink` accept `mem_<32-hex>` UIDs as source, resolve to `memory.toml` in items/ (writable) or shipped/ (read-only, for unlink), and append/remove raw `[[relation]]` rows. Memory relations use free-form label strings (no `RELATION_RULES` vocabulary gate), consistent with the catalog's `CatalogEdgeLabel::Raw` treatment.

## Code Impact

| Path | Change |
|------|--------|
| `src/main.rs` (`run_link`) | Memory branch before the `parse_canonical_ref` gate: `MemoryRef::parse` → resolve path → `append_memory_relation` |
| `src/main.rs` (`run_unlink`) | Symmetric memory branch |
| `src/memory.rs` | `resolve_memory_toml_path(root, uid)` — items/ first, shipped/ fallback |
| `src/memory.rs` | `append_memory_relation(path, label, target)` — raw toml_edit append |
| `src/memory.rs` | `remove_memory_relation(path, label, target)` — raw toml_edit remove |

## Design Decisions

### D1: Fork in `run_link`/`run_unlink`, don't extend `resolve_link_path`

`resolve_link_path` is tightly coupled to numbered entities (`parse_canonical_ref` → `validate_link` → `RELATION_RULES`). Memory relations bypass all of that. A parallel arm is cleaner than threading `Option<Memory>` through the existing path.

Source detection uses the existing `MemoryRef::parse` — accepts both `mem_<32-hex>` UIDs and `mem.<type>.<domain>.<subject>` keys (e.g. `doctrine link mem.fact.skills.audit-already-clean related SL-001`). Both resolve to the same `items/<uid>/memory.toml`.

### D2: Raw labels — no `RelationLabel` validation

Memory `[[relation]]` rows carry free-form label strings. The catalog pipeline already handles them as `CatalogEdgeLabel::Raw`. Enforcing a vocabulary on the write path would create a second authority inconsistent with the read path. The only guard is rejecting empty labels/targets (blank graph edges are noise).

### D3: Best-effort target validation

Numbered-entity `link` validates targets against `RELATION_RULES` — the label determines whether the target must resolve and what kinds are legal. Memory relations use raw labels, so there's no rule to consult.

But we can still guard against the most common mistake: a canonical-ref typo. If the target string parses as `PREFIX-NNN` via `parse_canonical_ref`, check it resolves on disk (`ensure_ref_resolves`). If it doesn't resolve, refuse with a clear error ("target SL-999 does not resolve to an existing entity").

Free-text targets and memory UIDs pass through unvalidated — the catalog scanner classifies them at scan time and surfaces diagnostics for dangling refs.

### D4: Items/ only — shipped/ is read-only

`resolve_memory_toml_path` resolves exclusively to `MEMORY_ITEMS_DIR/<uid>/memory.toml`. Shipped/ is gitignored, materialized from the binary by `doctrine memory sync`, and `rm -rf`-able — writing to it is pointless (it regenerates). If the uid exists only in shipped/, both `link` and `unlink` error: `"memory {uid} is a shipped corpus record — clone to items/ first, then link"`.

### D6: F1 trap defence

Reuse the `trailing_typed_table_after_relation` guard from `relation.rs`. A hand-edited `memory.toml` with `[trust]` or `[ranking]` after `[[relation]]` would cause silent corruption on naive tail-append. Make the guard public (or duplicate the logic) and refuse before touching the file.

### D5: Reuse `AppendOutcome`/`RemoveOutcome`

The existing enums (`Wrote`/`Noop`, `Removed`/`Absent`) carry exactly the right semantics. Memory append/remove returns the same types so `run_link`/`run_unlink` output is consistent.

## Invariants

- Empty label or empty target → hard error before touching the file
- Idempotent re-link/unlink: byte-unchanged file on no-op
- Existing numbered-entity `link`/`unlink` behaviour unchanged
- `toml_edit` escaping prevents target injection

## Test Coverage

| Test | What it proves |
|------|---------------|
| `link mem_xxx rel SL-001` succeeds | End-to-end: resolution + append |
| Re-link is no-op | Idempotency |
| `unlink mem_xxx rel SL-001` succeeds | Removal |
| Re-unlink reports `not linked` | Unlink idempotency |
| Nonexistent uid (items/) → error | Path resolution failure surface |
| Uid only in shipped/ → error | Shipped/ read-only gate |
| Empty label → error | Blank-guard |
| Empty target → error | Blank-guard |
| Typed table after `[[relation]]` → error | F1 trap defence |
| Existing `link SL-001 governed_by ADR-001` unchanged | Behaviour-preservation gate |
