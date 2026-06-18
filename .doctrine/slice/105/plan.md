# Implementation Plan SL-105: after edge removal

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML.

## Overview

Three sequential phases, bottom-up: pure leaf first, then the thin command
shells for remove, then prune. Each phase builds on the previous; no phase is
independent enough to parallelise — they share the same CLI enum and the
`target` optional wiring.

## Sequencing & Rationale

### PHASE-01: Leaf

Must come first per ADR-001: leaf ← command, no cycles. The shell phases
consume `dep_seq::remove_after` and `dep_seq::remove`; they can't exist before
the leaf does.

The `resolve_dep_seq_src_path` refactor is included here because it's a pure
extraction — no new behaviour, just exposing the source-half of the existing
validation so PHASE-03's prune can use it without duplicating the work-like
gate logic.

All unit tests live here in the leaf module. The existing suite (append, read,
status seams) acts as the behaviour-preservation gate — must stay green
unchanged.

### PHASE-02: Remove shell

Wire `--remove` on both `Command::After` and `BacklogCommand::After`. This
also makes `target` optional (`required_unless_present("prune")`) which
PHASE-03 needs. The remove path itself is thin: validate source+target via
existing `resolve_dep_seq_src`, call `dep_seq::remove`, report count.

E2E goldens pin the CLI output format. The backlog shell gets identical
treatment — minimal surface area, same leaf delegation.

### PHASE-03: Prune shell

Adds `--prune` flag (`conflicts_with = "remove"`). The prune loop is the most
complex shell logic in this slice: iterate `DepSeq::after`, probe each target
for existence + terminality, dedup target IDs, remove all edges per unique
dangling target, report per-edge with rank and reason.

The target probe is a generic `toml::Value` read checking `status` — works for
both backlog items and slices since both carry a top-level `status` key.

Manual VA-1: after implementation, prune all 12 affected items to clear the 15
`overrides:` lines from `doctrine backlog list`.

## Notes

- The 15 current `overrides:` lines (2026-06-18 scan) are the acceptance
  smoke-test — `doctrine backlog list` footer must be empty after manual prune.
- `--remove` errors on no-match by design — not idempotent. Documented in
  design §5.
- PHASE-02 introduces `target: Option<String>` on both `Command::After` and
  `BacklogCommand::After`. Append callers unwrap with `expect("target required unless --prune")` — clap guarantees presence.
