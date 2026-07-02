# Implementation Plan SL-180: Selector conformance hardening

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML.

## Overview

Two surfaces, one pure primitive (design §2). PHASE-01 delivers IMP-204 (the
design-time dry-run) and, as its foundation, the shared
`slice::selectors(root, id, intent)` reader. PHASE-02 delivers IMP-199 (the
import-belt scope refusal) on top of that reader, plus the pure
`conformance::undeclared_paths` primitive, the `--slice` plumbing, the dispatch
skill edits, and the layering proof.

## Sequencing & Rationale

**Why IMP-204 first (PHASE-01).** It is the cheaper, self-contained surface —
confined to `src/slice.rs`, reusing `conformance::compute`, `render_conformance`,
and `fold_name_status_line` verbatim. It also lands the shared selector reader
that PHASE-02 depends on, so the dependency runs 01 → 02 with no back-edge. And
it closes the *design-time* under-declaration gap before the *import-time* belt
starts refusing — the two form the defence-in-depth pair the SL-168 postmortem
called for, and catching gaps early (PHASE-01) reduces false positives at the
belt (PHASE-02).

**Why the phases are file-disjoint.** PHASE-01 touches only `slice.rs`;
PHASE-02 touches `conformance.rs`, `worktree/import.rs`, `worktree/mod.rs`, and
the two skill files. The only cross-phase coupling is PHASE-02 *reading* the
`slice::selectors` reader PHASE-01 authored — a consume, not a co-edit. This
keeps the default serial execution clean and would permit isolated dispatch
worktrees without conflict.

**Layering is a PHASE-02 exit gate, not an assumption (design §2, F2).** The
selector read sinks into `worktree::mod` (command tier), never `worktree::import`
(engine tier) — `import` takes `selectors: &[String]` as data. EX-5 makes
`just gate` green a hard exit criterion: the `worktree → slice` edge already
exists via `coordinate.rs` and the gate deduplicates edges at top-level-module
granularity, so no `tangle_baseline` growth is expected — but the gate is the
proof, verified after the edge lands, not asserted.

**Override policy (design §7).** No `--allow-undeclared`. The belt refuses with a
legible path list; the sanctioned unblock is `slice selector add … --intent
design-target --note "<why>"` then re-import. The selector upsert (with a
mandatory justification note) is the durable ledger.

## Notes

- **Token constant (STD-001 / POL-002).** `"undeclared-scope"` is added to
  `Refusal::token()` as an inline literal, matching the five existing refusal
  tokens in that same match arm — following the surrounding pattern rather than
  introducing a lone named const the siblings lack. The VT-2 goldens assert the
  token string as the load-bearing property.
- **Refusal stays `Copy` (design §3, F7).** Rather than widen the enum to
  `UndeclaredScope(Vec<String>)`, the shell re-derives the undeclared list with
  the same pure `conformance::undeclared_paths` — same fn, same inputs, no drift.
- **quotePath gap is pre-existing (design §4, F3).** PHASE-01 hardens the new
  range fold AND the adjacent registry fold in the same file; both are
  behaviour-preserving for the ASCII paths the current suite exercises.
