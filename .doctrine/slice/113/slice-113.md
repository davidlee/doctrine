# Shared entity mutation seam over atomic write

## Context

The 2026-06-19 architecture audit found the entity engine (`entity.rs`:
`materialise`, `scan_ids`, `read_meta`, `LocalFs`) covers entity **creation and
listing** — 28 modules route through it — but there is **no shared mutation
seam**. The update path is hand-rolled per kind: read TOML → mutate in memory →
`std::fs::write`. ~16 production modules do this directly, bypassing
`fsutil::write_atomic` (which only 6 files use):

- `dep_seq.rs:178,246,356,378` (5 sites), `concept_map.rs:1406,1421,1471,1566`
  (4 sites), plus one each in `memory.rs`, `revision.rs`, `requirement.rs`,
  `spec.rs`, `skills.rs`, `state.rs`, `worktree.rs`, `integrity.rs`,
  `install.rs`, `ledger.rs`, `backlog.rs`.

Two costs: (1) the same read→mutate→write dance is re-implemented per kind —
genuine parallel implementation against the "no parallel implementation" rule;
(2) non-atomic `fs::write` is an interrupted-write corruption risk for authored
TOML/MD that the existing `write_atomic` seam was built to prevent.

## Scope & Objectives

- Add an engine-tier mutation/save seam (e.g. `entity::update` /
  `entity::save_meta`) layered over `fsutil::write_atomic`, symmetric with the
  existing create/scan surface, honouring the TOML/MD storage tiering.
- Migrate the direct `std::fs::write` authored-file update sites onto the seam so
  every authored mutation is atomic and goes through one code path.
- Leave non-authored writes (runtime state, derived/regenerable artefacts) out
  unless they trivially benefit — the design names which call sites are in scope.

Closure intent: no authored-entity update path calls `std::fs::write` directly;
all route through the shared seam; `write_atomic` usage replaces the ad-hoc
writes; existing suites stay green (behaviour-preservation gate).

## Non-Goals

- `canonical_id` consolidation → SL-114 (separate DRY seam).
- Redesigning the TOML/MD schema or the read/parse path.
- Forcing runtime-state (`.doctrine/state/`) or derived-cache writes through the
  authored-entity seam where atomicity is not required.

## Summary

Give the entity engine a shared, atomic mutation seam and migrate the ~16
hand-rolled `fs::write` update sites onto it — closing a parallel-implementation
smell and an interrupted-write corruption risk in one move.

## Follow-Ups

- Related backlog: IMP-025 (content-hashed-path-set shared primitive),
  IMP-075 (`with_journaled_projection` extraction) — adjacent reuse cleanups.
