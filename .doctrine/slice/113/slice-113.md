# Shared entity mutation seam over atomic write

## Context

The 2026-06-19 architecture audit found the entity engine (`entity.rs`:
`materialise`, `scan_ids`, `LocalFs`) covers entity **creation and listing** but
has **no mutation path**. Every in-place authored update is hand-rolled per kind:
read TOML → `toml_edit` splice → `std::fs::write(path, string)`, bypassing
`fsutil::write_atomic` (used by only 6 files).

Design recount (supersedes the audit's estimate): **22 authored `fs::write` call
sites across 11 files** —

- `dep_seq.rs:178,246,356,378` (shared write-cores — `set_authored_status` alone
  backs 7 kinds), `concept_map.rs:1491,1506,1556,1651`, `memory.rs` (3),
  `main.rs` (3, supersede), `relation.rs` (2), and one each in `requirement.rs`,
  `spec.rs`, `integrity.rs`, `backlog.rs`, `revision.rs`, `map_server/routes.rs`.
- The audit missed **supersede** (`main.rs`) and the **map-server concept-map
  route**; several "per-kind" status writes already funnel through the shared
  `dep_seq` cores. See `design.md` §5.3 for the authoritative table.

Two costs: (1) the same read→mutate→write dance is re-implemented per kind —
genuine parallel implementation against the "no parallel implementation" rule;
(2) non-atomic `fs::write` is an interrupted-write corruption risk for authored
TOML/MD that the existing `write_atomic` seam was built to prevent.

## Scope & Objectives

- The seam is the **existing** `fsutil::write_atomic` — no new function. The
  call-site reality (every site holds a fully-joined path + a `String` body; the
  per-kind `toml_edit` splice is the only bespoke part) makes the byte-write the
  sole shared thing, and that primitive already exists in the leaf IO seam
  (ADR-001). A new `entity::save_meta` wrapper is rejected (design D1).
- Migrate the authored `std::fs::write` sites onto `write_atomic` so every
  authored mutation is atomic and goes through one code path. Read→mutate logic
  stays byte-identical per kind. The migration set is reconciled against the
  `clippy` guard (the oracle), not a hand count (design §5.3).
- Harden `write_atomic`'s temp-naming with a process-global counter so concurrent
  same-process writers (the map-server) don't collide (design D4 — the one change
  to the leaf seam; existing `write_atomic` test stays green unchanged).
- Add a `clippy` `disallowed-methods` guard on `std::fs::write`; the deliberate
  runtime/derived exclusions carry a documented `#[allow]` (design §5.4 / D3).

Guarantee scope: **swap-atomicity** (no reader-visible torn file, no half-written
authored file from an interrupted userspace write) — *not* power-loss durability
(no `fsync`; design D4).

Closure intent: no authored-entity update path calls `std::fs::write` directly;
all route through `write_atomic`; the `clippy` guard makes it permanent; existing
suites stay green (behaviour-preservation gate).

## Non-Goals

- `canonical_id` consolidation → SL-114 (separate DRY seam).
- Redesigning the TOML/MD schema or the read/parse path.
- Forcing runtime-state (`.doctrine/state/`) or derived-cache writes through the
  authored-entity seam where atomicity is not required.

## Summary

Migrate the ~22 hand-rolled authored `fs::write` update sites onto the existing
`fsutil::write_atomic` seam, harden the seam's temp-naming for concurrent
same-process writers, and lock it with a `clippy` guard — closing a
parallel-implementation smell and a reader-visible-tearing risk in one move,
adding no new authored-mutation abstraction.

## Follow-Ups

- Related backlog: IMP-025 (content-hashed-path-set shared primitive),
  IMP-075 (`with_journaled_projection` extraction) — adjacent reuse cleanups.
