# Review RV-250 — code-review of SL-197

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Lines of attack:
- Dispatch integration: are the implementation branches merged into main?
  Does the HEAD contain CPT code or was the slice closed against stale base?
- Code quality: record-kind surface integrity, ConceptFacet consistency with
  other 6 facets, test coverage of the empty-facet pattern, guard canary
  presence (P2 single-entry, P3 target/sources, P4 vocab-derivation).
- Process: did the close transition verify implementation presence?

## Synthesis

**Overall**: acceptable (with caveat)

**Synopsis**: The slice's authored design and plan are sound — a scoped-DRY
two-phase approach adding a seventh knowledge record kind with an empty
facet, status vocabulary `[draft, active, retired]`, and the right relation
membership (Concerns + RECORD-ride for Shapes/Spawns, D4 supersede gate).

The implementation on `dispatch/197` compiles clean and the approach of
behaviour-preserving PHASE-01 followed by mechanical PHASE-02 append is
well-executed. The P2/P3/P4 canaries are properly placed, the ConceptFacet
empty-facet pattern correctly handles seed scaffold vs display suppression,
and the three golden tests are properly re-pinned.

**Correction (post-review):** The original F-1 claimed the implementation
was never merged. It WAS — `main` carries the full CPT implementation
via `bcd93294` (candidate merge of `review/197`). The actual state:
`main` has all 12 source/test/template files with CPT changes; `edge`
(the primary worktree, where agents operate) has zero CPT code — it
diverged from the common ancestor (`8a14b94d`) with 4 metadata-only
commits (audit → reconcile → close). The implementation was integrated
correctly via the dispatch funnel; edge was simply never updated from
main afterward.

**Fix (revised):** merge `main` → `edge` to bring the primary worktree
up to date. The implementation is already on `main`. One minor code
quality finding (F-2, spurious Ser/De derives on ConceptFacet) should be
addressed on `main` before merging back to `edge`.

**Standing risks**: the close-process gap (F-3) will recur until a gate
refuses `done` when unmerged dispatch branches exist. Minted as IMP-267.

**Haiku**:
```
concept born, but lost —
the branches hold the code still;
merge what was undone.
```
