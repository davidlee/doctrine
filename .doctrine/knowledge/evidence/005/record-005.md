# EVD-005: Criterion histories support opt-in lineage across modes, not a VT-only repair

## Observation

A bounded RFC-027 H4 trial reconstructed six criterion histories across SL-233,
SL-241, and SL-224:

- SL-233 VT-2 split into narrowed VT-2 plus VT-9;
- SL-233 VA-4 split measurement from judgement into VA-4 plus VH-1, then changed
  again;
- SL-233 EX-8 relocated across PHASE-06, PHASE-13, and PHASE-14 with a partial
  contribution and eventual merge;
- SL-241 EX-7 and EX-8 each replaced an earlier claim wholesale in place;
- stable SL-224 PHASE-02 EX-1 served as a negative control.

The current plan read model is flat. `Criterion { id, text }` and
`VerificationCriterion` carry no revision, disposition, predecessor, or
successor fact (`src/plan.rs`). Only VT rows have machine consumers:
`check_vt_shape` and `vtgate` inspect the current structured row. EN, EX, VA, and
VH semantics are consumed by human/agent plan, phase, audit, and reconciliation
work rather than interpreted by Rust.

All positive cases preserve their history through some combination of in-place
rewriting, amendment vocabulary embedded in the current normative text, repeated
successor prose, and Git. That kept false semantics from governing in the sample,
but produced three concrete deficits:

1. a live-file reader sometimes needs Git commit order to determine which meaning
   is current;
2. split, relocation, and merge facts are restated across rows and phases rather
   than owned once;
3. no derived read can distinguish an active criterion from a tombstone or an
   absorbed predecessor.

The unchanged control needs no evolution record. This supports an opt-in
mechanism triggered only when criterion meaning changes, not a lineage burden on
every criterion.

## Adjudication

The evidence worker returned `narrow and retry` and proposed a VT-only
coverage-reconciliation check. The underlying evidence is accepted; that
narrowing is not.

Its own clearest relocation/merge case is an EX criterion, not a VT criterion.
Human and agent readers are real consumers of EN/EX/VA/VH even though Rust does
not interpret their prose. H4 also need not mechanically prove that successor
prose is semantically complete: its first job is to own succession and
disposition once, derive which leaves govern, and render predecessors as history.
Semantic completeness remains review work. Structured VT pattern-set
conservation may later support an additional machine check, but it is not the
general lineage boundary.

The trial therefore supports retaining H4 for specification coverage with this
minimum scope:

- criterion evolution is authored only when meaning changes;
- criterion references are phase-qualified or otherwise unambiguous;
- replacement, withdrawal, one-to-many split, many-to-one merge, and cross-phase
  relocation can identify predecessors and successors without restating their
  normative text;
- active leaves are derived structurally and historical predecessors cannot be
  mistaken for governing truth;
- unchanged criteria incur no additional record;
- criterion prose remains the normative semantic statement; lineage does not
  claim language-neutral proof that a split or merge preserved all meaning.

This is evidence for a spec-coverage assessment, not for a final schema or an
implementation slice.

## Limits

- The sample contains no observed production failure where a stale criterion
  caused a wrong gate result; it demonstrates structural ambiguity and repeated
  maintenance instead.
- The cases cover three recent slices and may reflect shared owner conventions.
- Current VT evidence is recomputed from the live row; the trial found no
  persisted plan-criterion evidence identity to migrate or stale.
- The evidence does not settle whether immutability belongs to the criterion id,
  each revision, or authored Git history, nor whether a separate lineage id is
  required.
- Product ownership, technical ownership, rendering, migration, and validation
  rules remain for spec coverage and design.

## Sources

- RFC-027 H4 and Stage 1.
- SL-233 plan histories: `7c4eb95b1`, `43986ffeb`, `7e8f78aa6`.
- SL-241 plan histories: `ba9f4a8a3`, `cbce7876a`.
- SL-224 PHASE-02 EX-1 negative control.
- `src/plan.rs` — plan criterion read model and `check_vt_shape`.
- `src/vtgate.rs` — live VT verification consumer.
- `src/state.rs` — runtime phase state/materialisation surface.
