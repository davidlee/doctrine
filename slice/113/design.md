# Design: Shared entity mutation seam over atomic write

## §1 — Target behaviour

Every authored-entity TOML/MD update path in production code routes through
`fsutil::write_atomic`. No `std::fs::write` touches an authored entity file.
A clippy `disallowed-methods` guard permanently prevents backsliding. The
`write_atomic` temp-naming is hardened against same-process concurrent writers
via a process-global `AtomicU64` counter.

## §2 — Module impact

### Leaf tier: `src/fsutil.rs`

Add an `AtomicU64` counter to `write_atomic`. Temp name changes from
`.{name}.{pid}.tmp` → `.{name}.{pid}.{counter}.tmp`. The counter is a single
`static NEXT: AtomicU64` that `fetch_add(1, Relaxed)`s on every call —
zero-dependency, collision-free per-process. The `fs::write` call inside
`write_atomic` (writing the temp) carries an internal `#[expect]` for the
clippy guard — it is the canonical use site.

Existing test `write_atomic_creates_then_overwrites_leaving_no_temp` stays green
unchanged.

New test: two spawned threads `write_atomic` the same path concurrently — both
succeed, final content is whichever wrote last (as expected from atomic rename),
no collision panic, no stray temps.

### Engine / command tiers — 18 sites across 8 files

Every site follows the same mechanical transform:

```rust
// Before
std::fs::write(&path, doc.to_string())
    .with_context(|| format!("Failed to write {}", path.display()))?;

// After
crate::fsutil::write_atomic(&path, doc.to_string().as_bytes())
    .with_context(|| format!("Failed to write {}", path.display()))?;
```

| File | Lines | Function |
|---|---|---|
| `dep_seq.rs` | 178, 246, 356, 378 | `set_authored_status`, `remove_after` IO, `apply_with`, `append_string_array` |
| `concept_map.rs` | 1491, 1506, 1556, 1651 | add-edge, add-edge-force, remove-edge, rename-node |
| `backlog.rs` | 1816 | `run_tag` |
| `spec.rs` | 799 | `add_requirement` |
| `requirement.rs` | 396 | `set_kind` |
| `integrity.rs` | 483 | `renumber` |
| `relation.rs` | 784, 803 | `append_edge`, `remove_edge` |
| `main.rs` | 4795, 4856, 4858 | `apply_supersede` (3 writes) |
| `map_server/routes.rs` | 412 | `handle_edit_concept_map_edge` |

Two sites use non-`.with_context` error wrappers (see §6 VT-6 for analysis):

- `relation.rs:784,803` — `.map_err(|e| anyhow::anyhow!("write {} …: {e}", ...))?`
  becomes `.with_context(|| format!("write {} …", ...))?` (the `anyhow::Error`
  from `write_atomic` nests under the context instead of being stringified).
- `map_server/routes.rs:412` — `.map_err(|e| MapServerError::ConceptMapIoError(e.to_string()))?`
  works unchanged: `e.to_string()` is valid for both `io::Error` and `anyhow::Error`.

The `dep_seq.rs` shared cores back ~7 entity kinds — no per-kind duplication.
The read→mutate step stays byte-identical; only the write primitive changes.

## §3 — Clippy guard (D3)

Add `std::fs::write` to `clippy.toml` `disallowed-methods` (not `Cargo.toml` —
the lint level is already `deny`; the method list lives in `clippy.toml`).

Gate to non-test via `#![cfg_attr(test,
allow(clippy::disallowed_methods))]` in `main.rs` (the binary crate root;
no `lib.rs` exists). This exempts unit tests (~50 `fs::write` calls for
fixture setup). Integration tests (in `tests/`) are separate crates and
already unaffected by the crate's lint settings.

Production exceptions carrying `#[expect(clippy::disallowed_methods, reason = "...")]`:

1. `src/fsutil.rs` — `write_atomic` itself (the canonical temp write, the
   single deliberate use).
2. `src/ledger.rs:408` — `store()` writes runtime journal manifests
   (`.doctrine/dispatch/`), not authored entities.

## §4 — Design decisions

| ID | Decision | Rationale |
|---|---|---|
| D1 | No new entity wrapper (`save_meta` rejected) | The existing `write_atomic` is the seam; every site already produces a `String` body ready for `.as_bytes()` — no abstraction layer needed |
| D3 | Clippy guard, gated non-test | 50 test-fixture `fs::write` calls serve fixture setup; cluttering them with `#[allow]` adds noise without safety gain. Production exceptions (`ledger.rs`, `fsutil.rs` itself) carry documented `#[expect]` |
| D4 | `AtomicU64` counter in `write_atomic` | Zero-dependency, collision-free per-process, minimal change to the leaf seam. The pid already disambiguates across processes; the counter disambiguates across threads |

## §5 — Guarantee scope

**Swap-atomicity** — no reader-visible torn file, no half-written authored file
from an interrupted userspace write. The `rename` syscall is atomic on a single
filesystem; a concurrent reader sees either the old file or the fully-written new
one.

Not guaranteed: power-loss durability (no `fsync`). The temp sits in the same
directory as the target so the rename never crosses a mount.

## §6 — Verification alignment

| ID | What | How |
|---|---|---|
| VT-1 | Existing `write_atomic` unit test | Creates, overwrites, leaves no temp. Must stay green (behaviour-preservation gate). |
| VT-2 | Concurrent write test (new) | Two threads `write_atomic` same file — both succeed, no panic, no stray temps. |
| VT-3 | `dep_seq` shared-core tests | `set_authored_status`, `remove_after`, `append_string_array` exercise the write path. Stay green. |
| VT-4 | E2E suites | Supersede, relation link/unlink, spec add-requirement, backlog tag, concept-map edge mutate — all stay green. |
| VT-5 | Clippy gate | `just check` zero-warn — no `disallowed-methods` violations except documented `#[expect]` sites. Manual verification: introduce a bare `std::fs::write` in production code, confirm clippy fails. |
| VT-6 | `relation.rs` & `map_server/routes.rs` error context | `relation.rs` sites shift from fresh `anyhow!("write {} …: {e}")` to `.with_context(\|\| format!("write {} …"))?` — the underlying `anyhow` error from `write_atomic` nests under the context. Functionally equivalent; no existing tests assert on the error message format for these sites. `map_server/routes.rs` uses `.to_string()` which works for both `io::Error` and `anyhow::Error` — no adaptation needed (R1 resolved). |
