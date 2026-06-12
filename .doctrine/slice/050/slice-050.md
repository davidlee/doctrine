# Priority surface efficiency and conceptual-precision cleanup

## Context

Follow-up to the SL-047 reconciliation review (RV-007) and a subsequent
`/code-review` pass over the priority subsystem (`src/priority/*`,
`src/relation_graph.rs` scan seam, `src/main.rs` `run_inspect`). SL-047 shipped
the cross-kind `survey`/`next`/`blockers`/`explain` CLI and its `inspect`
actionability block; the architecture is sound (pure/impure split, single render
source of truth, byte-identical `inspect` preserved). The review surfaced a
cluster of efficiency, conceptual-precision, and diagnosis-quality defects that
were out of SL-047's closure scope but are real debt. This slice sews them up.

None of the findings is a correctness/data-integrity blocker — the surfaces are
read-only and advisory, and the behaviour-preservation gate held on output bytes.
The defects are redundant work the gate could not see, a misnamed metric, and a
surface that answers confidently for entities that do not exist.

## Scope & Objectives

The seven review findings, in priority order:

1. **Scan-seam double-parse** (`relation_graph::scan_entities`). `status_for`
   already deserializes the full `meta::Meta` (which carries `title`) for every
   non-RV/REC kind, then `title_for` re-opens and re-parses the *same* toml into
   a `TitleOnly` struct. Every entity's toml is parsed twice per scan. Read once;
   special-case only RV/REC (the genuinely status-less kinds whose strict
   `read_meta` fails).

2. **`inspect` double corpus walk** (`main::run_inspect`). A single-id `inspect`
   now runs two full corpus scans: `relation_graph::render`/`inspect` (scan #1)
   and `priority::surface::actionability_block` → `graph::build` (scan #2, plus
   per-backlog `dep_seq` reads). Additionally `scan_entities` derives `status_for`
   (incl. an RV finding-ledger parse) and `title_for` for every entity, neither
   of which `build_relation_graph` consumes. Eliminate the redundant derivation
   for the relation-only consumer and/or share one scan across the composition.

3. **`survey` comparator recompute** (`surface::survey`). `sort_by` calls
   `actionability` (→ `blocked` → `blocked_by`: `in_edges` walk + `BTreeSet` +
   per-predecessor `class_of`) and `consequence` for both operands on every
   comparison; the subsequent `map` recomputes the same per row.
   Decorate-sort-undecorate: materialise each node's sort key and row signals
   once.

4. **`explain` double transitive walk** (`surface::explain`). `blocked_by_transitive`
   is computed once for the chain and again for `dep_level.len()` — two
   `reachable` walks for one node. Reuse the first result.

5. **`dep_level` misnamed** (`view::ReasonKind::OrderContrib` /
   `surface::explain`). Documented "dep-topology level"; actually the *count* of
   non-terminal transitive blockers — not a depth, and it shrinks as prereqs
   complete. Either name it for what it is (`blocker_count`) or compute the real
   composed level. Conceptual-precision fix; touches the `explain` render + JSON
   surface (golden update).

6. **Non-existent-id diagnosis gate** (`surface::explain`/`blockers`/
   `actionability_block`). `parse_key` validates ref *shape* only; a well-formed
   ref to an unminted id sails through every `None`-returning lookup and renders a
   clean empty/`Unrecognised` result indistinguishable from a real isolated node.
   Add an existence check so the "why is this here?" surfaces refuse (or clearly
   flag) entities that are not in the corpus.

7. **Dead vocabulary** (`graph::Dangling`/`dangling`, `ref_overlays`,
   `view::ReasonKind::Fallback`, `OrderContrib.seq_rank` always `None`). Built
   and/or rendered on every call, read by no surface — five `#[expect(dead_code)]`
   suppressions are the tell. Decide per item: wire a real consumer, or drop it
   (stop assembling the `dangling` Vec per edge in `build()` if nothing reads it).
   The `seq_rank: None`-only path makes `render`'s `Some` arm + the JSON field
   untestable dead branches.

Closure intent: redundant parses/scans removed (assert one parse per entity per
scan, one scan per `inspect` composition); the survey comparator does no graph
work; `explain` walks the transitive set once; the misnamed field renamed or
re-derived with goldens updated; non-existent ids handled deliberately (test);
each dead-vocabulary item either consumed or removed with its suppression. Gate
green (`just check`), the 13 priority + 9 inspect goldens updated where the
surface text legitimately changes, behaviour-preservation otherwise held.

## Non-Goals

- **No new priority feature** — no new verb, no new channel, no policy-version
  bump beyond what a deliberate surface-text change to `explain` forces. This is
  cleanup, not capability.
- **No partition / actionability *semantics* change** — `eligible ∧ ¬blocked`
  (D12), the status-class table, and the consequence subset are settled (SL-047
  §10); untouched except where finding 5 renames a render label.
- **No cordage core change** (SPEC-001 D1).
- **No cross-kind dep/seq capture** (DD-2 / IMP-033) — still dormant.
- **No `inspect` relation-portion output change** — the relation surface stays
  byte-identical; only the redundant *work* behind it and the additive block are
  in scope.
- The `just check` cordage-suite gate hole (ISS-007) is separate; not this slice.

## Summary

Pure-tier and shell cleanup of the SL-047 priority surfaces: kill the double
parse and double scan, stop recomputing graph signals in the survey comparator
and the explain walk, name the order metric honestly, gate non-existent ids, and
resolve the shipped dead vocabulary. Quality and confidence-to-change work; no
behaviour the operator relies on changes except the `explain` order label.

## Follow-Ups

- ISS-007 (cordage-suite gate coverage) remains independently open.
