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

## RV-415 dispositions

User approved all proposals 2026-09-29 ("ok, no objection"). F-3/F-4/F-6/F-8/F-10
recorded as DEC-360; all 17 disposed fix-now on RV-415 (responses there);
F-17 routed `control`, its criterion sketch stays on the ledger for `/plan`.

### Further review pass

Next pass: the RV-415 raiser verifies the 16 prose-repaired dispositions against
the revised design.md and DEC-360, then concludes. It should probe (a) whether
the worst-case reservation and 80% headroom actually close F-1/F-2, (b) whether
B-point-matched really isolates the ranker (same rubric, same truncated text),
(c) the unreadable/absent classification's reliance on scan-learned prefix
directories, and (d) new stale claims in the cost estimate. No fresh full
adversarial pass is proposed beyond that verification.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-29 · design locked (run rev 51)

### Produced

- DEC-352, DEC-353, DEC-354, DEC-355, DEC-356, DEC-357, DEC-358, DEC-359, DEC-360
- `design.md` locked (sec-1..9 user-reviewed); RV-415 concluded (F-1..F-18; F-17 control-routed, verify after `slice phases`)

### Learned

- `research/research.md` (runtime tier): census 39 research slices; Hindsight
  listwise evidence (`research/raw/hindsight-jev-reranker.md`).

### Open

- `/plan`: carry RV-415 F-17's criterion sketch (ledger) onto the transport/send-log phase.
- Tooling: top-level `acceptance` in reviewing records a spurious `design-accepted` (observations 44/…, 30/…).
