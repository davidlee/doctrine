# Hoist kind registry to engine tier to break relation layering cycles

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
- `relation_graph.rs:82,423,652` has the same upward reach (backlog dep-seq,
  governance kinds, spec interaction types). Non-cyclic today, same defect class.

Root cause: each `*_KIND` constant lives in its command verb module, so the
engine must look up to know what kinds exist. The fix is to invert the
dependency — the engine should own kind identity; command modules consume it.

This is the enabling change for SL-112 (a compiler-enforced engine crate
boundary is impossible while these cycles exist).

## Scope & Objectives

- Establish a single engine-tier home for kind identity (a leaf/engine `kinds`
  module, or fold into `registry`/`entity`) holding the `*_KIND` constants, their
  prefixes, and the prefix↔kind mapping.
- Re-point `relation.rs` and `relation_graph.rs` to the engine-tier source so the
  engine no longer imports any command module for kind identity.
- Re-point command modules to consume the hoisted constants (re-export or import
  from the engine tier) so existing call sites keep compiling.
- Eliminate the 7 `relation` ↔ command cycles and the `relation_graph` upward
  edges for kind identity. The `worktree → slice::run_phases` edge
  (`worktree.rs:1742`) is a separate concern — out of scope here.

Closure intent: no `crate::<command_module>::*_KIND` import remains in
`relation.rs` / `relation_graph.rs`; a dependency check (manual or the SL-112 gate
once it exists) shows the engine tier no longer depends on the command tier for
kinds; existing suites stay green unchanged (behaviour-preservation gate).

## Non-Goals

- The engine **crate** split and the automated layering fitness gate → SL-112.
- The `worktree → slice` upward edge and the `retrieve ↔ memory` command-tier
  cycle (lower severity; tackle separately).
- Any change to relation semantics, the relation vocabulary, or storage.

## Summary

Invert kind identity ownership: move `*_KIND` from command modules to the engine
tier so the relation engine stops importing upward. Breaks the cycles ADR-001
forbids and unblocks SL-112.

## Follow-Ups

- SL-112 consumes this to land the compiler-enforced boundary.
- Audit remaining upward edges (`worktree → slice`, `retrieve ↔ memory`) for
  their own slices if they persist.
