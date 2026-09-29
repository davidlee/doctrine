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

## RV-415 proposed dispositions (2026-09-29, NOT yet user-approved)

Adversarial pass by codex (gpt-6-sol, high) raised F-1..F-17. Agent verified
each against the cited sites; all hold. None disposed on the ledger yet.

**Design changes — awaiting explicit user yes** (record as one DEC refining
DEC-358/DEC-359, then edit design + scope):

- F-4 accept: opaque Choice option keys (`o1`…); section prefix = title +
  heading path, no id; add headline over labels not already named in the query.
- F-6 accept: add arm B-point-matched (one Noul per entity over B-list's exact
  option text). B-list − B-point-matched = primitive; B-point −
  B-point-matched = text/max effect. C comparison reported as pipeline only.
- F-3 accept: BM25 pool N = 50 fixed, recorded per run (B-list cap ≈ 540 tok).
- F-10 accept: every list question run forward + reversed, probabilities
  averaged; order agreement (Kendall τ) reported as diagnostic.
- F-8 accept: DEC-352 agreement criterion — proceed iff Kendall τ ≥ 0.8
  (batched vs section-as-state point scores, tune slice's pool) and |Δ
  recall@10| ≤ 0.05; else DEC-352 fallback.

**Fixes — agent to apply unless user objects** (not yet stated either way):

- F-1 (blocker) accept: reserve worst case per attempt = 64k tokens
  (documented per-request max, ≈ $0.0027); never start an attempt whose worst
  case exceeds remaining cap; reported usage replaces the reservation.
- F-2 accept: pack to 80% of limits; calibrate estimator in probe; on
  context-limit 422 split request / shrink caps, counted; VT with dense ASCII
  (hash/base64) input.
- F-5 accept (modified): per-kind recall breakdown; VH adds a random 5 eval-
  slice label audit (labels only, no rubric exposure). No citation-context rule.
- F-7 accept: report BM25 pool coverage (labels in top-50) as the pool ceiling;
  drop "ceiling" wording for C.
- F-9 accept: arm×slice complete iff all candidates answered after one retry of
  the missing; incomplete excluded from paired headline, reported; VT.
- F-11 accept: separate memory-key matcher for the dropped-memory count.
- F-12 accept: Choice invariant — exact key set, finite p in [0,1], sum 1±0.01,
  choice ∈ keys; else Unanswered, not cached.
- F-13 accept: prefix walks rank order, first non-fitting entity ends it; cost
  min(est tokens, 1k); empty bodies excluded.
- F-14 accept: section provenance optional; score semantics labelled per ranker.
- F-15 accept: labels.rs pure; fixture I/O in the shell.
- F-16 accept: distinguish absent-at-commit (no numeric dir for that kind in the
  export) from present-but-unreadable; unreadable excluded from denominator,
  disclosed; headline also shown excluding slices with unreadable labels.
- F-17 accept: negative egress VTs (disallowed path, symlink escape, memory
  kind) refused before transport; send-log line written before each attempt.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-29 · design reviewing (run rev 31) · ea713441f

### Produced

- DEC-352, DEC-353, DEC-354, DEC-355, DEC-356, DEC-357, DEC-358, DEC-359
- `design.md` (draft, sec-1..9); RV-415 (17 findings, open)

### Learned

- `research/research.md` (runtime tier): census 39 research slices; Hindsight
  listwise evidence (`research/raw/hindsight-jev-reranker.md`).

### Open

- RV-415 F-1..F-17 — proposed dispositions above.
