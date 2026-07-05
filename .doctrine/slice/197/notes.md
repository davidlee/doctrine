# SL-197 implementation notes

## Approach

Two-phase (PHASE-01 DRY, PHASE-02 add) paid off. PHASE-01's canaries and
vocab-derived messages made PHASE-02 a clean mechanical append — append a
variant + fill compiler-forced arms; no debugging. External review (codex/GPT-5.5)
caught the seed `[facet]` header requirement (scaffold-order invariant) and
golden re-pin scope — integrated into design before the plan locked.

## Design fidelity

All D1–D4 implemented as specified:
- **D1**: status vocab [draft, active, retired]; seed draft; hidden/terminal {retired}
- **D2**: empty ConceptFacet; seed emits empty `[facet]` header; `show` suppresses it
- **D3**: CPT rides RECORD for Shapes/Spawns/Supersedes; concerns added explicitly
- **D4**: zero `supersede.rs` edits — CPT rejected by `_ => None` + validate_matrix absence
- **D0** (scope): PHASE-01 DRY (P2/P3/P4) + PHASE-02 CPT append

## Patterns worth reusing

- **Empty facet pattern**: a unit-like `ConceptFacet` with no fields. Seed emits the
  `[facet]` header for scaffold-order invariance; `show` suppresses the empty block
  at display. The pattern also works for EvidenceFacet (no fields in seed, but
  evidence fields are generated at `knowledge new`).
- **Canary-forced manual adds**: the four combined constants (SEARCH_DEFAULT,
  ALL_KINDS, TAGGABLE, ADMISSIBLE_DEP_TARGETS) are hand-spelled — the design
  explicitly rejected auto-derive (design §2/D0). A canary
  `combined_constants_cover_record` catches omissions.
- **Non-supersedable kind with zero edits**: CPT has no supersede policy (D4).
  The existing `_ => None` wildcard arm + `validate_matrix` absence gates it
  automatically — identical to HYP. Zero `supersede.rs` edits.

## Gotchas

- The concept kind's "draft" status joined knowledge's union vocab, which broke
  `tests/e2e_list_conformance.rs` (`status_flag_is_recognised_grammar_on_every_kind`
  assumed knowledge had no draft-bearing kind). Fixed during PHASE-02 funnel;
  selector added for conformance completeness (F-1).
- Residual record-kind scatter (stale doc comments in supersede.rs, integrity.rs,
  etc.) is out of scope — deferred to the backlog DRY-the-scatter item.

## Backlog disposition

- IMP-244 part 1: completed by SL-197
- IMP-244 part 2 (per-edge relational descriptors): separate item, NOT addressed here
- IMP-244 itself stays open (part 2 pending) — note added
