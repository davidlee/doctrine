# Design SL-113: Shared entity mutation seam over atomic write

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

The entity engine (`entity.rs`) owns entity **creation** (`materialise*`,
`write_fileset`) and **listing** (`scan_ids`/`scan_named`), composing the leaf IO
seam `fsutil` (`write_atomic`, `safe_join`, `create_new_file`). It owns no
**mutation** path. Every in-place authored-file update is hand-rolled per kind:
read TOML → `toml_edit` splice → `std::fs::write(path, string)`. That byte-write
bypasses `fsutil::write_atomic`, which exists precisely to make authored writes
atomic (temp-write + rename).

Two costs: (1) ~22 call sites re-spell the same non-atomic write — a
parallel-implementation smell against "no parallel implementation"; (2)
`std::fs::write` is non-atomic — a crash mid-write tears a committed `*.toml`,
the corruption `write_atomic` was built to prevent.

## 2. Current State

Authored mutation, every kind, is `read_to_string` → `parse::<DocumentMut>()`
(or a pure helper) → mutate → `std::fs::write`. `write_atomic` is used by only 6
files; the authored *update* paths are not among them.

Two partial convergences already exist and matter:

- **`dep_seq.rs` holds shared authored write-cores.** `append` (`:178`), `remove`
  (`:246`), `set_authored_status` (`:356`), `append_string_array` (`:378`). Each
  ends in `std::fs::write`. `set_authored_status` is the status-flip seam for
  slice, requirement, backlog, revision, governance, knowledge/ASM — one write
  site behind seven kinds.
- **`relation.rs`** has the cross-kind edge writer (`append_edge`/`remove_edge`),
  also ending in `std::fs::write`.

The remainder are per-kind/per-command bespoke writes (memory, concept-map,
spec, integrity renumber, requirement `set_kind`, supersede, the map-server
concept-map route).

## 3. Forces & Constraints

- **ADR-001 (layering).** `fsutil` is the leaf IO seam; `write_atomic` is its
  "accountable, atomic" write. `entity` (engine) → `fsutil` (leaf) is the
  existing downward edge. Callers (command/engine) → `fsutil` is downward and
  legal. No new module dependency, no cycle.
- **Behaviour-preservation gate (AGENTS.md).** This touches shared machinery
  (`dep_seq` cores, `relation`). The existing suites are the proof and must stay
  **green unchanged** — no test edits.
- **Storage rule.** In scope = **authored** `*.toml`/`*.md` only. Runtime state
  (`state.rs` phase sheets, `ledger.rs` manifest, `worktree.rs` marker) and
  derived/install (`install.rs`, `skills.rs`) deliberately stay on `fs::write`.
- **Memory `mem...unified-read-seam-does-not-deliver-a-unified-write-seam`.** A
  shared writer needs only a shared write *primitive*, not a shared on-disk shape
  or shared mutation logic. SL-113 shares exactly the byte-write; the per-kind
  `toml_edit` splice stays bespoke and untouched. This is the writer-*safety*
  gain (atomic) the memory names, nothing wider.
- **`as simple as possible`.** The seam already exists. The hypothesised
  `entity::save_meta` is rejected (D1).

## 4. Guiding Principles

- Reuse the existing seam; add no abstraction (D1).
- Migrate the byte-write only; leave read→mutate logic byte-identical.
- Make the authored/runtime boundary machine-enforced and self-documenting (D3).
- Maximum leverage: migrating the `dep_seq`/`relation` shared cores atomicizes
  many kinds through one site each.

## 5. Proposed Design

### 5.1 System Model

No new module, no new function. The seam is the existing
`fsutil::write_atomic(path: &Path, bytes: &[u8]) -> anyhow::Result<()>`. SL-113
is a **migration** of authored byte-writes onto it, plus a **clippy guard**
making the migration permanent.

### 5.2 Interfaces & Contracts

Unchanged. `write_atomic` is the contract: write a sibling temp in the same
directory, `rename` over the target (atomic on one filesystem); a concurrent
reader sees old-or-new, never torn. Callers keep their existing signatures,
return types, and error mapping.

**Mechanical transform, every authored site:**

```
std::fs::write(p, doc.to_string())            →  fsutil::write_atomic(p, doc.to_string().as_bytes())
std::fs::write(p, next /* String */)          →  fsutil::write_atomic(p, next.as_bytes())
```

Surrounding error handling is preserved verbatim: `.with_context(|| …)` and
`.map_err(|e| DomainError…)` both compose over `write_atomic`'s `anyhow::Result`.
Add `use crate::fsutil;` where absent.

### 5.3 Data, State & Ownership

The 22 authored call sites (11 files):

| File | Sites | Role |
|---|---|---|
| `dep_seq.rs` | 178, 246, 356, 378 | shared cores: append / remove / `set_authored_status` (7 kinds) / `append_string_array` |
| `relation.rs` | 784, 803 | `append_edge` / `remove_edge` |
| `memory.rs` | 2514, 2618, 2876 | memory edits |
| `concept_map.rs` | 1491, 1506, 1556, 1651 | `set_dsl` writes |
| `main.rs` | 4795, 4856, 4858 | supersede |
| `requirement.rs` | 396 | `set_kind` (`set_status` funnels through `dep_seq`) |
| `spec.rs` | 799 | member append |
| `integrity.rs` | 483 | renumber `id` |
| `backlog.rs` | 1816 | (status funnels through `dep_seq`) |
| `revision.rs` | 888 | (status funnels through `dep_seq`) |
| `map_server/routes.rs` | 412 | concept-map via HTTP |

**Out of scope** (stay on `fs::write`, get `#[allow]` — §5.4): `state.rs:409`,
`ledger.rs:408`, `worktree.rs:1862`, `install.rs:586`, `skills.rs:637`.

Line numbers are an at-design snapshot; the plan re-locates by function before
editing.

### 5.4 Lifecycle, Operations & Dynamics

**Closure guard.** Add to `clippy.toml` `disallowed-methods`:

```toml
{ path = "std::fs::write", reason = "authored entity writes must route through fsutil::write_atomic (SL-113); runtime/derived sites carry an explicit #[allow]" }
```

`just gate` runs clippy bins/lib only (no `--all-targets`), so test code is
unlinted — no test-site noise. Deliberate non-authored production sites each
carry `#[allow(clippy::disallowed_methods, reason="…")]`, which documents the
authored/runtime boundary in code:

| Site | reason |
|---|---|
| `fsutil.rs:63` | the seam itself — `write_atomic`'s internal temp write |
| `state.rs:409` | runtime phase sheet — disposable, atomicity not required |
| `ledger.rs:408` | runtime coordination manifest |
| `worktree.rs:1862` | runtime worker marker |
| `install.rs:586` | derived asset unpack |
| `skills.rs:637` | derived asset unpack |

A future authored mutation reaching for `fs::write` then fails the gate.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1 (behaviour preservation).** Each migrated site is reached on exactly
  the same branches as before; no-op-write guards (`if !changed { return }`,
  status no-op, relation `Noop`/`Absent`) are untouched, so mtime-hold semantics
  hold. `write_atomic` is invoked only where `fs::write` was.
- **INV-2 (atomicity).** Every authored write is now temp+rename; a torn
  authored `*.toml` is structurally impossible. Owned by `write_atomic`'s own
  test.
- **E1 (borrow lifetime).** `write_atomic(p, doc.to_string().as_bytes())` — the
  `String` temporary lives to end of statement; the `as_bytes()` borrow is valid.
  No allocation-count change vs `fs::write` (which also materialised the String).
- **E2 (`catalog/test_helpers.rs:16`).** Uses `fs::write`. If compiled into the
  lib (not `#[cfg(test)]`-gated) it trips the new lint → needs an `#[allow]`.
  Plan-time gating check (see OQ-1 resolution / Q at §6).

## 6. Open Questions & Unknowns

- **OQ-1 (phase split).** One phase, or shared-cores (`dep_seq`/`relation`)
  before per-kind/command sites? Work is mechanical and behaviour-gated; lean
  **one phase**. Deferred to `/plan`.
- **Q (test-helper gating).** Confirm `catalog/test_helpers.rs` is `#[cfg(test)]`
  / test-only at phase-plan; if lib-compiled, add an `#[allow]` (E2). Cheap
  check, not a design risk.

## 7. Decisions, Rationale & Alternatives

- **D1 — the seam is the existing `fsutil::write_atomic`; add no new function.**
  Call-site reality: every site holds a fully-joined absolute path and a `String`
  body; the only shared thing is the byte-write, which `write_atomic` already is.
  *Rejected — (b) a thin `entity::save_authored` wrapper:* adds an indirection
  with no new capability (mutation logic stays per-kind, primitive stays in
  `fsutil`) — a miniature of the smell being closed. *Rejected — (c) a
  `(tree_root, rel)` containment-enforcing seam:* forces callers to thread a tree
  root they mostly don't hold; containment was already enforced at create time
  (no unsafe authored paths found). Cost without matching benefit.
- **D2 — authored-only scope.** Runtime/derived writes stay on `fs::write`.
  Matches the slice non-goal; gives a crisp closure invariant ("no *authored*
  update path calls `fs::write`"). `state.rs` phase sheets are `rm -rf`-able by
  design — runtime atomicity is a possible follow-up, not this slice.
- **D3 — closure via `clippy` `disallowed-methods` + explicit `#[allow]`.** The
  test-noise objection to a global ban does not apply (the gate skips tests). The
  `#[allow]` annotations convert the authored/runtime boundary from tribal
  knowledge into documented-in-code intent. *Rejected — audit-grep only:* no
  permanent regression guard.

## 8. Risks & Mitigations

- **R1 — context-frame drift.** Keeping `.with_context` nests `write_atomic`'s
  internal "Failed to rename" frame under the caller's "Failed to write {path}".
  Cosmetic; mitigate by confirming no test asserts on these strings at plan time.
- **R2 — map-server error mapping.** `routes.rs:412` maps to
  `MapServerError::ConceptMapIoError(e.to_string())`; `write_atomic`'s
  `anyhow::Error` stringifies cleanly — preserved.
- **R3 — missed authored site.** The slice scope itself undercounted (missed
  supersede + map-server). Mitigate: the `clippy` guard surfaces any remaining
  authored `fs::write` at gate time — a missed site fails the build, not silently
  ships.

## 9. Quality Engineering & Validation

- **VT-1 — behaviour-preservation gate (primary).** Full suite green,
  **unchanged**. The per-kind suites (status round-trips, no-op-writes-nothing,
  malformed-refuse, relation append/remove idempotence, supersede, concept-map
  edits) prove the read→mutate logic is intact. No test edits — that is the gate.
- **VT-2 — `just gate` green** with the new disallowed-method and the `#[allow]`
  inventory. Proves no stray authored `fs::write` and that exclusions are
  explicit.
- **VA-1 — atomicity present.** `write_atomic` owns its torn-write test in
  `fsutil.rs`; per-site the property is "routes through the seam", verified by the
  guard (VT-2), not by added per-site fault injection (no new signal).

## 10. Review Notes

(internal adversarial pass + optional external/inquisition recorded here)
