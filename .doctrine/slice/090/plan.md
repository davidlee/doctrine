# Implementation Plan SL-090: Wire link/unlink CLI and template for memory relations

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

Three sequential phases that build the memory-relation write surface bottom-up:
resolution → manipulation → CLI fork. Each phase is a thin, testable layer that
the next depends on.

## Sequencing & Rationale

### PHASE-01: Memory resolution helper

Before we can touch `memory.toml`, we need to *find* it. `resolve_memory_toml_path`
is the single chokepoint that translates a user-supplied `MemoryRef` (uid, uid-
prefix, or key) to the writable `items/<uid>/memory.toml` path. It reuses the
existing `resolve_uid_prefix` scan for prefix disambiguation and enforces the
D4 policy gate (shipped/ is read-only). This is pure infrastructure — no CLI
wiring, no relation manipulation — so it can be tested in isolation against
in-memory directory fixtures.

Why first: every subsequent phase needs to locate the right `memory.toml`. By
isolating resolution from manipulation, we keep each phase's test surface small
and avoid coupling the F1 guard to path discovery.

### PHASE-02: Memory relation write helpers

With the path resolved, we need to safely append/remove `[[relation]]` rows.
The existing `relation::append_relation_row`/`remove_relation_row` are tightly
coupled to `RelationLabel` (an enum of validated vocabulary variants). Memory
labels are free-form strings per design D2 — the catalog pipeline already
treats them as `CatalogEdgeLabel::Raw`.

Design D6 explicitly calls for duplicating the F1 guard in `memory.rs` (not
reusing the vocabulary-bound helpers). The duplicated helpers share the same
shape — `toml_edit::DocumentMut`, idempotency-first guard, F1 layout check,
`toml_edit::value()` escaping — but accept `&str` labels instead of
`RelationLabel`. The import direction is leaf → leaf (`memory.rs` imports
`AppendOutcome`/`RemoveOutcome` from `relation.rs`), which is cycle-free
since `relation.rs` imports nothing from `memory.rs`.

Why second: the helpers are pure over `&str` + `&Path` — they don't know about
roots, resolution, or CLI arguments. Tested in isolation with hand-crafted
TOML strings, they prove the write seam works before we expose it to users.

### PHASE-03: Wire link/unlink CLI fork

The integration layer. A thin fork in `run_link`/`run_unlink`: before the
existing `parse_canonical_ref` gate, try `MemoryRef::parse(source)`. On match:
resolve path (PHASE-01), best-effort target validation (D3), then
append/remove (PHASE-02). On non-match: fall through to the existing numbered-
entity path unchanged.

Target validation is best-effort: if the target parses as a canonical ref
(`PREFIX-NNN`), check it resolves on disk — `ensure_ref_resolves` from the
integrity layer. Free-text targets and memory UIDs (`mem_<hex>`) pass through
unvalidated, since memory labels are raw and there's no `RELATION_RULES` entry
to consult. The catalog scanner classifies dangling targets later and surfaces
diagnostics.

The CLI help text update is a one-line `///` doc change per command variant —
trivial but user-visible, so it lives in this phase rather than as noise in
PHASE-01/02.

Why last: the fork is the thinnest possible integration — it delegates
everything to PHASE-01 and PHASE-02. By the time we write it, both
dependencies are proven correct in isolation. The fork itself is a single
`if let Ok(mref) = MemoryRef::parse(source)` block per function.

## Notes

- The behaviour-preservation gate is VT-6 in PHASE-03: existing `link SL-048
  governed_by ADR-010` must pass unchanged. The fork's fallthrough path must
  be byte-identical to the current code.
- No `[[relation]]` template scaffolding needed — if `link` works, the
  template doesn't need noise (per scope Non-Goals).
- Memory-as-TARGET (numbered entity → memory) is explicitly deferred to a
  follow-up slice.
