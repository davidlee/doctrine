# Hoist kind identity to a leaf `kinds` module to break relation layering cycles

## Context

ADR-001 mandates `leaf ← engine ← command, no cycles`. An architecture audit
(2026-06-19) found the rule's predicted failure mode has materialised: the
relation engine reaches **up** into command-tier modules for kind identity.

- `relation.rs:247-262` references `crate::slice::SLICE_KIND`,
  `crate::spec::{PRODUCT_SPEC,TECH_SPEC}_KIND`, `crate::review::REVIEW_KIND`,
  `crate::backlog::{ISSUE,IMPROVEMENT,CHORE,RISK,IDEA}_KIND`,
  `crate::concept_map::CONCEPT_MAP_KIND`, `crate::knowledge::*_KIND`,
  `crate::rec::REC_KIND`, `crate::revision::REV_KIND`. Each of those modules
  imports `relation` back → **7 confirmed cycles**.
- `relation_graph.rs:82,423,652` has a *related* upward reach but is **out of
  scope** (see Non-Goals): all three sites are behavior-entangled, not pure
  identity (`backlog::dep_seq_for`; gov supersession needing `dir` +
  `governance::supersession_pair`; `spec::interaction_types`), and `relation_graph`
  contributes **zero cycles** (its edges are non-cyclic). The cycle-break is fully
  achieved by `relation.rs` alone.

Root cause: each `*_KIND` constant lives in its command verb module, so the
engine must look up to know what kinds exist. Only the **prefix** (the canonical
kind identity, compared by `==` everywhere) is needed by the engine — the full
`Kind` cannot move down (its `scaffold: fn` binds it to the command module). The
fix inverts the dependency: a leaf-tier `kinds` module owns the prefix vocabulary;
the relation engine and the command modules both consume it.

This is the enabling change for SL-112 (a compiler-enforced engine crate
boundary is impossible while these cycles exist).

## Scope & Objectives

- Establish a single leaf-tier home for kind identity — a new `kinds` module
  holding the canonical prefix per kind and the relation source/target groupings
  (`GOV`/`BACKLOG`/`RECORD`). Not `entity` (kind-blind by design) nor `registry`
  (the FK-index seed). The full `*_KIND` consts stay in their command modules
  (scaffold-coupled); only the prefix is hoisted.
- Re-point `relation.rs` off every `crate::<cmd>::*_KIND` alias onto `kinds::*`;
  re-key the `RELATION_RULES` table element type from `&'static Kind` to
  `&'static str` (the engine compares kinds by prefix only). Public fn signatures
  (`lookup`/`tier1_edges`/`rels_block`) stay `&Kind` — zero caller churn.
- Re-point each command `*_KIND` const's `prefix:` field to `kinds::<X>` so the
  prefix literal lives in exactly one place (no parallel copy).
- Eliminate the 7 `relation` ↔ command cycles. `relation_graph` is out of scope
  (above); the `worktree → slice::run_phases` edge is a separate concern.

Closure intent: no `crate::<command_module>::*_KIND` import remains in
`relation.rs`; a dependency check (manual or the SL-112 gate once it exists) shows
the relation engine no longer depends on the command tier for kind identity;
existing suites stay green unchanged (behaviour-preservation gate).

## Non-Goals

- The engine **crate** split and the automated layering fitness gate → SL-112.
- The `worktree → slice` upward edge and the `retrieve ↔ memory` command-tier
  cycle (lower severity; tackle separately).
- Any change to relation semantics, the relation vocabulary, or storage.

## Summary

Invert kind-identity ownership: a leaf-tier `kinds` module owns the prefix
vocabulary; the relation engine and command modules both consume it. The engine
stops importing command modules for kind identity. Breaks the 7 cycles ADR-001
forbids and unblocks SL-112.

## Follow-Ups

- SL-112 consumes this to land the compiler-enforced boundary.
- **Widen the leaf identity interface (contingent on SL-112).** This slice hoists
  only the prefix as `&str` — narrow by scope, not a product position. SL-112's
  design must adopt `kinds` as its starting identity shape and, if it places
  `relation_graph` engine-side, widen it to `KindCore { dir, prefix, … }` so the
  engine can read `dir` from a non-command source (resolves the `:423` edge). The
  `&str`→`KindCore` re-key is mechanical and compile-enforced (design §8 R4).
- **`relation_graph` upward edges (contingent on SL-112).** Its three
  command-imports (`:82` `backlog::dep_seq_for`, `:423` gov supersession needing
  `dir` + `governance::supersession_pair`, `:652` `spec::interaction_types`) are
  behavior-entangled, not pure identity. Resolve **iff** SL-112's layer
  classification places `relation_graph` inside the engine crate; if it lands
  command-tier, these are legal lateral edges and the follow-up is moot.
- Audit remaining upward edges (`worktree → slice`, `retrieve ↔ memory`) for
  their own slices if they persist.
