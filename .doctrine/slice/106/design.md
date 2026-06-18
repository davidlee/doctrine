# Design: Fold knowledge::set_record_status + reword F-1 refuse hints

## Decisions

### D1 — Fold `set_record_status` onto `set_authored_status` (IMP-061)

`knowledge::set_record_status` (`src/knowledge.rs:1346–1383`) is a byte-duplicate
of `dep_seq::set_authored_status` (SL-062). It independently reads, parses,
no-op-guards, malformed-checks, inserts, and writes — all of which
`set_authored_status` + `apply_status` already do.

**Delete `set_record_status` entirely.** Move the path-building from the deleted
fn body into `run_status` (the caller), following the slice/backlog precedent
where the shell resolves the path and passes it to the shared write-core.

Before (`run_status` lines 1406–1407):
```rust
let today = crate::clock::today();
set_record_status(&root, kind, id, state, &today)?;
```

After:
```rust
let today = crate::clock::today();
let name = format!("{id:03}");
let path = root
    .join(kind.kind().dir)
    .join(&name)
    .join(format!("{RECORD_STEM}-{name}.toml"));
let hint = format!(
    "malformed record {name}: missing seeded `status`/`updated` \
     — restore the missing keys and retry; the file is left untouched"
);
dep_seq::set_authored_status(&path, &[("status", state), ("updated", &today)], &hint)?;
```

**Behaviour-equivalence proof:**

| Concern | `set_record_status` | `set_authored_status` (via `apply_status`) |
|---|---|---|
| No-op guard | Checks `status` string equality | Checks all managed pairs excluding `updated` — same result since the only identity key is `status` |
| Malformed check | Requires `status` + `updated` keys present | Requires all managed keys present (`status`, `updated`) — identical |
| Write | Inserts `status` + `updated`, writes once | Inserts all managed pairs, writes once — identical |
| Changed-bool | Returns `()` (caller doesn't branch) | Returns `bool` — caller ignores it (same effect) |

No risk: the `updated`-excluded no-op guard and the strict malformed-refuse are
proven on the shared seam by SL-062's existing test suite.

### D2 — Reword three "regenerate via" refuse hints (IMP-066)

Three per-kind status setters still emit destructive guidance suggesting the user
regenerate an authored entity. Replace with the non-destructive pattern (option 3
from design discussion).

| File:line | Current | Replacement |
|---|---|---|
| `src/slice.rs:526` | `"malformed slice {name}: missing \`status\`/\`updated\` (regenerate via \`slice new\`)"` | `"malformed slice {name}: missing \`status\`/\`updated\` — restore the missing keys and retry; the file is left untouched"` |
| `src/backlog.rs:1390` | `"malformed backlog item {name}: missing seeded \`status\`/\`resolution\`/\`updated\` (regenerate via \`backlog new\`)"` | `"malformed backlog item {name}: missing seeded \`status\`/\`resolution\`/\`updated\` — restore the missing keys and retry; the file is left untouched"` |
| `src/knowledge.rs` | Old `set_record_status` body (deleted in D1) | Replaced inline in D1 above |

The `dep_seq` refuse (used by governance, requirement) already uses the correct
pattern. After D2, all six status-setter call sites speak the same language:
actionable ("restore and retry"), non-destructive ("untouched"), no suggestion to
regenerate.

## Code impact

| File | Change | Lines |
|---|---|---|
| `src/knowledge.rs` | Delete `set_record_status` fn (1346–1383); inline path+hint in `run_status`; delete the `set_record_status` doc comment (1337–1344) | ~-40, +15 |
| `src/slice.rs` | Reword hint string at line 526 | 1 line |
| `src/backlog.rs` | Reword hint string at line 1390 | 1 line |
| `src/dep_seq.rs` | Unchanged | 0 |

## Risks

- **R1 — behaviour preservation.** The only behavioural change is the refuse
  wording. The status-transition path is identical. Existing knowledge, slice,
  and backlog status tests must stay green unchanged.

## Verification

- `cargo test knowledge::` — all green
- `cargo test -- slice` — all green (including the refuse-message assertion at
  `dep_seq.rs:793–806` which checks no "regenerate" appears)
- `rg "regenerate via" src/` — zero hits
- `rg "fn set_record_status" src/` — zero hits
- `just gate` clean
