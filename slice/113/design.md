# Design: Shared entity mutation seam over atomic write

## §1 — Target behaviour

Every authored-entity TOML/MD update path in production code routes through
`fsutil::write_atomic`. No `std::fs::write` touches an authored entity file.
A clippy `disallowed-methods` guard permanently prevents backsliding. The
`write_atomic` temp-naming is hardened against same-process concurrent writers
via a process-global `AtomicU64` counter.

### §1.1 — Current state

`write_atomic` already handles ~11 call sites across 5 modules:

| Module | Lines | Context |
|---|---|---|
| `memory.rs` | 1759, 1797, 2273 | relation append/remove, verify |
| `review.rs` | 1399, 1607, 2051 | baton write, authored ledger, cache |
| `boot.rs` | 233, 520, 957, 1075, 1096 | boot snapshot, hooks, agent config |
| `skills.rs` | 929 | canonical agent defs |
| `coverage_store.rs` | 75 | save coverage file |

This slice converts the **remaining** 18 production `std::fs::write` call sites
on authored entity files — listed in §2.

## §2 — Module impact

### §2.1 — Tier analysis (ADR-001)

Every call site imports `crate::fsutil::write_atomic` (leaf tier). The
dependency direction is uniformly downward — no upward edge, no cycle.

| File | ADR-001 tier | Import validity |
|---|---|---|
| `fsutil.rs` | **leaf** | The seam itself — `write_atomic` lives here |
| `dep_seq.rs` | engine | `crate::fsutil` ← valid downward edge |
| `concept_map.rs` | engine | `crate::fsutil` ← valid |
| `backlog.rs` | engine | `crate::fsutil` ← valid |
| `spec.rs` | engine | `crate::fsutil` ← valid |
| `requirement.rs` | engine | `crate::fsutil` ← valid |
| `integrity.rs` | engine | `crate::fsutil` ← valid |
| `relation.rs` | engine | `crate::fsutil` ← valid |
| `main.rs` | command | `crate::fsutil` ← valid |
| `map_server/routes.rs` | command | `crate::fsutil` ← valid |

### Leaf tier: `src/fsutil.rs`

The public signature of `write_atomic` is unchanged (`pub(crate) fn
write_atomic(path: &Path, bytes: &[u8]) -> anyhow::Result<()>`). The only
change is the internal temp-file naming.

Add an `AtomicU64` counter:

```rust
static NEXT: AtomicU64 = AtomicU64::new(0);
```

Temp name changes from `.{name}.{pid}.tmp` → `.{name}.{pid}.{counter}.tmp`,
where `counter = NEXT.fetch_add(1, Relaxed)`. Zero-dependency, collision-free
per-process. The `fs::write` call inside `write_atomic` (writing the temp)
carries `#[expect(clippy::disallowed_methods, reason = "canonical temp-file write before atomic rename")]`.

Existing test `write_atomic_creates_then_overwrites_leaving_no_temp` stays green
unchanged (behaviour-preservation gate).

New test (VT-2): two spawned threads `write_atomic` the same path concurrently.
After both threads join:
- Both calls return `Ok(())`
- Exactly one target file exists, zero `.tmp` siblings
- The file content is one of the two written values (last-to-rename wins, which
  is non-deterministic but bounded to the two inputs)

### Engine / command tiers — 18 writes across 9 files

Every site follows the same mechanical transform:

```rust
// Before
std::fs::write(&path, doc.to_string())
    .with_context(|| format!("Failed to write {}", path.display()))?;

// After
crate::fsutil::write_atomic(&path, doc.to_string().as_bytes())
    .with_context(|| format!("Failed to write {}", path.display()))?;
```

| File | Lines | Enclosing function | Notes |
|---|---|---|---|
| `dep_seq.rs` | 178, 246, 356, 378 | `append`, `remove`, `set_authored_status`, `append_string_array` | Shared cores — back ~7 entity kinds |
| `concept_map.rs` | 1491, 1506 | `run_add_edge` | Normal + force-duplicate arms of the same match |
| `concept_map.rs` | 1556 | `run_remove_edge` | |
| `concept_map.rs` | 1651 | `run_rename_node` | |
| `backlog.rs` | 1816 | `run_tag` | Backlog item TOML |
| `spec.rs` | 799 | `append_member` | Members TOML (private helper) |
| `requirement.rs` | 396 | `set_kind` | Requirement TOML |
| `integrity.rs` | 483 | `renumber` | Entity id fixup |
| `relation.rs` | 784 | `append_edge` | `[[relation]]` row append |
| `relation.rs` | 803 | `remove_edge` | `[[relation]]` row remove |
| `main.rs` | 4795, 4856, 4858 | `run_supersede` | 3 writes: old status flip, new supersedes append, old status flip |
| `map_server/routes.rs` | 412 | `mutate_concept_map` | `POST /api/concept-map/:id` handler |

Two sites use non-`.with_context` error wrappers (see §6 VT-6 for full analysis):

- `relation.rs:784,803` — `.map_err(\|e\| anyhow::anyhow!("write {} …: {e}", ...))?`
  becomes `.with_context(\|\| format!("write {} …", ...))?`. API-contract equivalent
  (same `anyhow::Result<()>` return); display format changes — the OS-level error
  from `write_atomic` nests in the anyhow chain instead of appearing in the outer
  string.
- `map_server/routes.rs:412` — `.map_err(\|e\| MapServerError::ConceptMapIoError(e.to_string()))?`
  works syntactically unchanged. `anyhow::Error::to_string()` returns the
  outermore context message (e.g. "Failed to write temp /path/.x.12345.1.tmp"),
  while `io::Error::to_string()` gave the OS message (e.g. "No space left on
  device"). Different fragment, same wrapper — no adaptation needed, but the
  error display changes. No integration tests assert on MapServer error message
  format.

## §3 — Clippy guard (D3)

Add `std::fs::write` to `clippy.toml` `disallowed-methods` (not `Cargo.toml` —
the lint level is already `deny` at line 229; the method list lives in
`clippy.toml`).

```toml
{ path = "std::fs::write", reason = "Use crate::fsutil::write_atomic for authored entity files — atomic rename prevents torn writes" },
```

Gate to non-test via `#![cfg_attr(test,
allow(clippy::disallowed_methods))]` in `main.rs` (the binary crate root;
no `lib.rs` exists). This exempts unit tests (~50 `fs::write` calls for
fixture setup). Integration tests (in `tests/`) are separate crates and
already unaffected by the crate's lint settings.

Production exceptions carrying `#[expect(clippy::disallowed_methods, reason = "...")]`:

1. `src/fsutil.rs` — `write_atomic` itself: the canonical temp-file write
   before atomic rename.
2. `src/ledger.rs:408` — `store()`: writes runtime journal manifests
   (`.doctrine/dispatch/`), not authored entities.

## §4 — Design decisions

| ID | Decision | Rationale |
|---|---|---|
| D1 | No new entity wrapper (`save_meta` rejected) | The existing `write_atomic` is the seam; every site already produces a `String` body ready for `.as_bytes()` — no abstraction layer needed |
| D3 | Clippy guard, gated non-test via `main.rs` inner attribute | 50 test-fixture `fs::write` calls serve fixture setup; cluttering them with `#[allow]` adds noise without safety gain. Production exceptions carry documented `#[expect]` |
| D4 | `AtomicU64` counter in `write_atomic` | Zero-dependency, collision-free per-process, minimal change to the leaf seam. The pid already disambiguates across processes; the counter disambiguates across threads |

## §5 — Guarantee scope

**Swap-atomicity** — no reader-visible torn file, no half-written authored file
from an interrupted userspace write. The `rename` syscall is atomic on a single
filesystem; a concurrent reader sees either the old file or the fully-written new
one.

Not guaranteed: **power-loss durability** (no `fsync` on the temp file before
rename). A crash after the rename commits but before the data blocks reach stable
storage could leave a zero-length or partial target file. This is acceptable for
authored entity files — git tracks the content, and no in-flight write is
unrecoverable. The temp sits in the same directory as the target so the rename
never crosses a mount.

## §6 — Verification alignment

| ID | What | How |
|---|---|---|
| VT-1 | Existing `write_atomic` unit test | Creates, overwrites, leaves no temp. Must stay green (behaviour-preservation gate). |
| VT-2 | Concurrent write test (new) | Two threads `write_atomic` the same path. After join: both return `Ok(())`; exactly one target file exists, zero `.tmp` siblings; content is one of the two inputs (last-to-rename wins, non-deterministic). |
| VT-3 | `dep_seq` shared-core tests | `append`, `remove`, `set_authored_status`, `append_string_array` exercise the write path. Stay green. |
| VT-4 | E2E suites | Supersede, relation link/unlink, spec add-requirement, backlog tag, concept-map edge mutate — all stay green. |
| VT-5 | Clippy gate | `just check` zero-warn — no `disallowed-methods` violations except documented `#[expect]` sites. Manual verification: introduce a bare `std::fs::write` in production code, confirm clippy fails. (A compile-fail test is possible but disproportionate for one lint guard; the manual test proves the guard works once, and `just check` prevents backsliding permanently.) |
| VT-6 | Error-wrapper format changes | `relation.rs`: API-contract equivalent (same `anyhow::Result<()>`), display format changes (OS error nests in chain instead of appearing in outer string). `map_server/routes.rs`: `e.to_string()` works for both error types but produces different fragments — `io::Error` gives OS message, `anyhow::Error` gives outermost context. No existing tests assert on error message format for either site. Cosmetic, not semantic. |
