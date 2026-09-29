# Notes SL-277: Jev relevance trial

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design-surface triage (2026-09-29, explore.triage)

Evidence: `research/research.md` (runtime tier). Settled inputs: IDE-061
§ Settled (not reopened here).

### Constraining governance

- Binding: STD-001, STD-003 (by `governed_by`, beyond their `src/` scope),
  STD-002, POL-001, AGENTS.md storage rule, SPEC-019 / DEC-174 (EVD shape),
  ADR-023 (EVD is evidence), ADR-001 cross-crate half (only if the crate
  path-depends on the root lib).
- Not applicable (reasons in research.md Thread 1): POL-002 (not shipped),
  POL-003, ADR-005, ADR-008, ADR-019, ADR-021 (proposed; lint covers it),
  ADR-022 (proposed; no peer corpus), ADR-024, SPEC-007, PRD-017/SPEC-026,
  SPEC-016.

### Open questions (→ design-run inquiry map)

1. Arm C request shape and feasibility: section-as-state (cookbook shape,
   request-bound: ~25k req/slice) vs query-as-state with candidate sections
   batched as structured questions (`raw/jev-primitives-noul.md:448`; cuts
   requests ~N×, accuracy effect unknown — jaggedness as state/questions grow).
   Or subsample slices / bounded wide pool.
2. Label set: research.md only (39 slices, SL-229..275) vs + design.md (256,
   post-research, leakier); RV-kind citations; tune/spot-check vs eval split
   inside the small pool.
3. Snapshot commit rule: parent of first-add of `slice-NNN.toml`; long
   scope→research gaps (SL-245).
4. Query text and rubric: what is the "query" (scope md at snapshot?);
   applicability (Noul) + contribution (Score) wording.
5. Sectioner: heading split, 32k state+question cap, tiny-section merge;
   entity rank = best section.
6. Recall-at-budget token budget.
7. Crate name, CLI shape of rank/eval; path dep on root lib (reuse
   `integrity::KINDS`) vs standalone.
8. Cache/log home (`.doctrine/state/jev/`) and nix hermeticity (`rustls`, since
   crane builds `--workspace`).

### Risks

- Pool too small for tune/eval separation → overfit rubric.
- Free-tier rate limit unknown and dynamic; arm C may be infeasible at scale.
- Silver labels undercount relevance (found-and-used ≠ relevant).
- Nix release-check breaks if the crate's dep closure is not hermetic.

### Assumptions

- `JEV_API_KEY` is available in the jail (verified present 2026-09-29).
- Research.md files on disk are the originals as used (they are gitignored;
  no history to check against).

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open
