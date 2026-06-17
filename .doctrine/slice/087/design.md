# SL-087 Design: Trim the Memory section of the boot snapshot

## Current behavior

`produce()` for `SourceKind::Memories` calls `memory::list_rows(root,
Some(MemoryType::Signpost), ListArgs { status: ["active"], ... })` — a full
table render with columns `uid, type, status, trust, key, title`. The table is
~50 lines for ~20 active signpost memories. This violates ADR-005's PUSH-tier
compactness invariant: the snapshot carries discovery metadata that belongs in
the pull tier (`/retrieve-memory`).

## Target behavior

The Memory section renders two elements:

1. A reference instruction:
   `Run /retrieve-memory to surface relevant memories for your task.`
2. A compact list of memory keys, one per line, for active signpost memories
   only. No uid/type/status/trust/title columns.

Result: ~22 lines instead of ~50. Discoverability is preserved via the key
list; full detail is reached via the pull path.

## Code impact

Single change site: the `SourceKind::Memories` arm in `produce()` in
`src/boot.rs`.

### New narrow API on `memory`

```rust
pub(crate) fn boot_keys(root: &Path) -> Result<Vec<String>>
```

- **Visibility:** `pub(crate)` — called from `boot.rs` only.
- **Filtering:** Internal to the function: filters to `status == active` AND
  `kind == Signpost`. No parameters — the call site never varies these filters.
- **Sort:** Key-ascending (the canonical order for a compact key listing — no
  uid/created metadata in the output).
- **Keyless memories:** `Memory.key` is `Option<String>`. Memories with no key
  render their `uid` as the fallback key line — the uid is always present and
  unambiguous.
- **Data source:** Reuses `memory::collect_all()` (already `pub(crate)`) — same
  source, narrower projection. No new filesystem read path.

The existing seam pattern (`memory::list_rows` is the boot→memory boundary for
the current table) is preserved.

### Updated `produce()` arm

```rust
SourceKind::Memories => {
    let body = memory::boot_keys(root).map(|keys| {
        if keys.is_empty() {
            return String::new();
        }
        let mut body = String::from(
            "Run /retrieve-memory to surface relevant memories for your task.\n"
        );
        for key in &keys {
            body.push_str(key);
            body.push('\n');
        }
        body
    });
    section_or_marker(heading, body)
}
```

Routes through `section_or_marker` — consistent with every other producer arm:
an error returns the marker, an empty corpus returns the marker, and a populated
result renders the reference line + key list.

## Test impact

### Updated tests (existing behavior changes)

- `produce_markers_a_non_exec_source_and_carries_the_exec_path`: the Memory arm
  with no memories now returns the existing marker — *no change* to this test
  because boot_keys returns empty.

### New tests

- **VT:** With active signpost memories, `produce("Memory", &SourceKind::Memories,
  root, exec)` returns the reference line followed by key lines, one per memory.
- **VT:** Keys are sorted key-ascending (the canonical order for a compact key
  listing without metadata, per Design Decisions).
- **VT:** `memory::boot_keys()` returns the correct keys for active signpost
  memories; an empty corpus returns an empty vec.

### Unaffected tests

- Structure tests (`boot_sequence_orders_*`) — no section ordering change.
- `produce_static_*`, `produce_governance_*` — unrelated arms.

## Design decisions

| Decision | Rationale |
|---|---|
| New narrow API on `memory` | Follows existing seam pattern, testable in isolation |
| Keys only (no uid/type/status/trust) | Discoverability without metadata bloat; compact per ADR-005 |
| Reference line always present when keys exist | Points at the richer pull path; empty corpus gets marker |
| Keys sorted ascending | Canonical order for a compact listing without metadata |
| Uid fallback for keyless memories | Every active signpost gets a line; uid is always available |
| `section_or_marker` error path | Consistent with every other producer arm |

## Verification alignment

- **VA:** Regenerated `boot.md` has ~22 lines (1 reference + ~21 keys) instead
  of ~50 lines of full metadata table.
- **VT:** Existing and new tests pass.
- **VT:** `just check` / `just gate` — zero clippy warnings.
