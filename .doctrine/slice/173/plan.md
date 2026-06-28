# Implementation Plan SL-173: backlog list --after / --needs dependency-sequence edge filter

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

One phase. The change is a single cohesive addition to one file
(`src/backlog.rs`): two repeatable list filters that ride the existing
backlog-local retain seam. There is no natural fracture line — splitting "add
flags" from "add filter logic" would leave a dead intermediate state (flags
parsed but ignored) with no independent value, against the no-parallel-impl and
small-cohesive-unit ethos. So the work is one TDD phase: red on `list_rows`
behaviour, green via the predicate + `norm_ref`, refactor.

## Sequencing & Rationale

PHASE-01 covers the whole surface from `design.md`:

1. Write failing `list_rows` unit tests first (VT-1..VT-6) — the pure compute
   half is the test seam, so behaviour is provable without spawning the CLI.
2. Add the `norm_ref` helper and the two local retains at the seam (after the
   `--kind` retain, before `any_tagged`).
3. Wire the `--after`/`--needs` clap args and thread them through
   `run_list`/`list_rows` (as params, like `kind` — not `ListArgs`).
4. Green, then refactor (dedup the two near-identical retains if it reads
   cleaner; keep the predicate pure).

No phase ordering question arises (single phase). The ordering machinery
(`compose`) is deliberately untouched — it borrows the full corpus before
`retain`, so membership and order stay independent.

## Notes

- The seam discipline is load-bearing: the retains MUST sit before the
  `any_tagged` computation so the dynamic tags-column visibility reflects the
  final displayed set, not the pre-filter set.
- `parse_canonical_ref` (integrity), not backlog `parse_ref`, because edges can
  point at any kind (`SL-169`), not just backlog prefixes.
- Behaviour-preservation gate: no existing `backlog`/`listing` suite changes —
  the new tests are additive; the shared listing axes and ordering are untouched.
- Test seam is proven and ready: `test_support::Fixture { rels: Some(RelLit {
  needs, after, .. }) }` already seeds edge-bearing items; existing tests call
  `list_rows` directly (the deterministic String-returning compute boundary).
  Adding the `after`/`needs` params to `list_rows` ripples to its existing test
  call sites (the wrapper near the bottom of the test module + direct callers) —
  pass `&[]` there. Mechanical, expected, not a behaviour change.
