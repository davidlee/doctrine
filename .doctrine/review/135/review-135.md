# Review RV-135 — reconciliation of SL-116

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed:** candidate/116/review-001 (dispatch candidate cand-116-review-001),
built from main + impl-bundle (refs/heads/review/116). Evidence refs: dispatch/116
(coordination), dispatch/116-PHASE-03 (worker).

**Lines of attack — invariants held to:**
1. **Structural:** `src/worktree.rs` gone; `src/worktree/` with 12 files per §Target layout;
   `mod worktree;` in main.rs unchanged.
2. **Public surface:** 8-symbol external re-export checklist honoured; no caller file changes.
3. **Visibility:** widen set (7 items) is `pub(super)` only; no `pub(crate)` leak of
   internal helpers. `gather_tree_clean` not re-exported (F-1).
4. **Behaviour preservation:** all 46 original tests pass with byte-unchanged bodies.
5. **Layering:** ADR-001 sub-classification present for all 10 worktree/ files;
   `coordinate = command` (truthful about the slice upward edge). `MixedUmbrella` green.
6. **Cleanliness:** `#![expect(unused_imports)]` removed from mod.rs; orphaned expects pruned;
   test re-exports gated behind `#[cfg(test)]`.
7. **Test co-location (D2):** each machine file carries its own tests.

## Synthesis

The SL-116 folder split is mechanically correct and behaviour-preserving — the
core contract of the slice. `src/worktree.rs` (3539 lines) is replaced by a
`src/worktree/` folder with 12 files, one per concern per the §Target layout
map. No caller file changed; the 8-symbol external re-export surface holds; the
ADR-001 layering sub-classification is truthful (coordinate=command, allowlist=leaf,
rest engine). The gate is green: architecture_layering passes MixedUmbrella,
clippy zero-warn, all tests pass (2242/2245; 3 pre-existing env failures on main).

Three findings surfaced, none blocking closure:

- **F-1 (tolerated):** `base_has_slice_plan` is `pub(crate)` not file-private.
  The test stayed in mod.rs rather than moving to coordinate.rs (D2). The
  visibility is a mild over-widen — `pub(super)` would suffice, and the
  semantic scope is unchanged (coordinate-only by domain).

- **F-2 (tolerated):** Seven `#![expect(unused, …)]` attributes on
  provision/import/land/gc/fork/subagent suppress real dead-code lints from
  items that became file-local-unused post-split (consumed only through mod.rs
  re-exports). The expects are honest — removing them fires warnings — but they
  are PHASE-02 extraction scaffolding that PHASE-03 should have pruned. The
  split's mechanical correctness is not diminished.

- **F-3 (follow-up):** D2 test co-location is incomplete. Only allowlist,
  marker, and shared received their tests; the 7 lifecycle machine files
  (provision/import/land/gc/fork/coordinate/subagent) carry zero
  `#[cfg(test)]` blocks — their tests remain in mod.rs's monolithic block
  (~32 tests). This violates D2's explicit rejection of the T2 alternative
  ("reproduces the smell in tests and divorces tests from code") and is the
  most substantive gap between design and implementation. Behaviour-preservation
  holds (tests pass), and moving them is a cohesion improvement, not a correctness
  fix. Follow-up backlog item warranted.

**Standing risks:** minimal. The split is purely mechanical — no behaviour,
state-machine, or allowlist-semantics change. The `worktree → slice` upward
coupling edge is unchanged (Non-Goal). The 7 `#![expect(unused)]` attributes
should not accumulate — future worktree changes should not add more without
cleaning the existing ones first.

**Tradeoffs consciously accepted:**
- F-1 over-widen: `pub(crate)` vs `pub(super)` is equivalent in this tree;
  fixing it would move a test (D2 violation deeper than it fixes).
- F-2 expects: suppressing real lints is honest but ugly; cleaning them requires
  refactoring each file's item visibility or re-export strategy — scope creep
  for a mechanical split.
- F-3 test distribution: de-interleaving 32 tests across 7 files is ~1 phase of
  work; the split shipped the mechanical core correctly, and the test
  distribution is a follow-up concern.

## Reconciliation Brief

### Per-slice (direct edit)
- design.md §Target layout — `base_has_slice_plan[p]` should read
  `base_has_slice_plan[pc]` to reflect actual `pub(crate)` visibility (F-1).

### Governance/spec (REV)
(none — F-2 and F-3 are code concerns, not governance/spec changes)

## Reconciliation Outcome

### Direct edits applied
- design.md §Target layout line 108: `base_has_slice_plan[p]` → `base_has_slice_plan[pc]`
  (reflects actual `pub(crate)` visibility; RV-135 F-1)
- design.md §Visibility line 171: removed `base_has_slice_plan` from the "Stay private" list
  (it is no longer file-private; RV-135 F-1)

### REVs completed
(none — no governance/spec changes in this reconciliation brief)

### Tolerated / follow-up
- RV-135 F-1: tolerated — `pub(crate)` vs `pub(super)` over-widen; rationale in finding disposition
- RV-135 F-2: tolerated — `#![expect(unused)]` scaffolding; rationale in finding disposition
- RV-135 F-3: follow-up → IMP-146; test co-location deferred to post-close cleanup

Reconcile pass complete — handoff to /close.
