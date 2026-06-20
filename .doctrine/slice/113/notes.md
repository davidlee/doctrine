# SL-113 — Implementation notes

Durable findings harvested from the runtime phase sheets. Authored, committed.
Feeds /audit + /reconcile.

## PHASE-01 — write_atomic hardening

- Temp name is now `.{name}.{pid}.{seq}.tmp` (`fsutil.rs`), `seq` from a
  function-local `static AtomicU64` (`fetch_add(1, Relaxed)`). Chose a
  function-local static over module-level (§5.2 sketch) to scope it to its sole
  user — no behavioural difference.
- VT-2 (`write_atomic_concurrent_writers_same_path_leave_no_torn_temp`) added;
  VT-1 unchanged (behaviour-preservation). Both green.

## PHASE-02 — site migration

### F1 — write_atomic overwrites a read-only *target file* (NEW production semantic)

`write_atomic` writes a temp in the target's directory then `rename`s over the
target. Unix `rename(2)` keys on **directory** write permission, not the target
file's mode — so it **succeeds over a `0o444` target file** where the old bare
`fs::write` failed with `EACCES`. Authored files are not normally read-only, so
the practical blast radius is nil and it is arguably an improvement (atomicity is
the point). But it means `write_atomic` is **not** a drop-in for code that relied
on target-file-mode to *induce* a write failure.

- **Fallout:** `spec::tests::spec_req_add_orphan_on_append_failure_left_uncommitted`
  forced failure via `chmod 0o444` on `members.toml`. Re-induced via a read-only
  **directory** (`0o555` on `…/product/001`), perms restored before tempdir reap.
  The test's behavioural assertion (failed member-append leaves the reserved
  requirement an uncommitted orphan) is preserved; only the failure-induction
  mechanism changed.
- **RECONCILE AT AUDIT:** design.md §3 / VT-1 claims the migration is
  behaviour-preserving with *"no test edits to prove the migration."* That claim
  is now inaccurate — one test fixture had to change. Not a regression in the
  production contract, but the design wording needs a reconciliation note (the
  read-only-target semantic is a real, if minor, behaviour change). Consulted with
  user 2026-06-20 → "fix fixture + document".

### F2 — facet_write was unclassified in the ADR-001 layering map

`src/facet_write.rs` (added by SL-118, commit `9976ed2a`, with `out=0` tracked
edges) was never added to `.doctrine/adr/001/layering.toml`. The SL-112 layering
gate flags a unit once it is an edge **source**; my `facet_write → fsutil` edge
gave it its first outbound edge, surfacing the gap as `Unclassified("facet_write")`.
Classified **leaf** (imports std + leaf only — `Path`, `anyhow::Context`, `fsutil`),
matching sibling shared-core `dep_seq`. Pre-existing SL-118 omission, fixed here
because this slice surfaced it. `leaf → leaf` is always legal — no tier ambiguity.

### F3 — error-wrapper changes (VT-6, expected by design R1/R2)

- `relation.rs` (`append_edge`/`remove_edge`): `map_err(anyhow!("…: {e}"))` →
  `with_context(|| "…")`. The OS error now nests in the anyhow source chain
  instead of being flattened into the outer message string. Added
  `use anyhow::Context;`. Cosmetic, not semantic — no test asserts on the format.
- `map_server/routes.rs`: wrapper unchanged — `MapServerError::ConceptMapIoError(
  e.to_string())` works for both `io::Error` and `anyhow::Error`.

## PHASE-03 — clippy guard

### F4 — design D3/EX-3 (#[allow]) reversed to #[expect] (RECONCILE AT AUDIT)

Design D3/§5.4/EX-3 mandate `#[allow(clippy::disallowed_methods, …)]`, explicitly
rejecting `#[expect]`. But the repo **enforces** `allow_attributes = "deny"`
(`Cargo.toml [lints]`) — bare `#[allow]` does **not compile** (14 existing
`#[expect]` in src, 0 `#[allow]`). The design author appears not to have known the
lint was denied. Used `#[expect]` (enforced canon outranks the design decision;
boot: plan/design ≠ higher authority than canon). Consulted with user 2026-06-20.

The design's stated reasons for `#[allow]` are moot here:
- *"#[expect] fires unfulfilled before the rule"* — the guard lands in the SAME
  commit as the annotations, so each `#[expect]` is fulfilled (the lint fires).
- *"brittle to call-graph drift"* — the only residual; the repo already accepted
  this tradeoff globally. A future stale `#[expect]` flags itself (a feature).

`state.rs` uses a **fn-level** `#[expect]` on `set_phase_status` — its lone
`fs::write` is the function's tail expression, which cannot carry a stable
stmt-level attribute (`stmt_expr_attributes` is unstable). The fn has exactly one
write, so the scope is precise.

VH-1 proven: removing any annotation re-exposes a bare production `fs::write` and
the guard errors with the SL-113 reason note (clippy non-zero).

**Reconcile:** design D3 + §5.4 table + PHASE-03 EX-3 wording (`#[allow]` →
`#[expect]`) at audit. The clippy.toml reason string was updated to say `#[expect]`.

### Style

All call sites use fully-qualified `crate::fsutil::write_atomic(…)` — the
established style in this codebase (cf. `crate::fsutil::safe_join` in memory.rs),
needs no per-file import. The byte arg gains `.as_bytes()` (`write_atomic` takes
`&[u8]`; `fs::write` took `impl AsRef<[u8]>`).
