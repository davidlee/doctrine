# Notes SL-249: Knowledge facet write seam

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-06 · stage `design` (pre-run) · f31b6187

### Produced

- `SL-249` — this slice; scope card carries four objectives, four settled
  questions, R1/R2/R2a/R4 and A1/A2.
- `CHR-056` — SL-222's unretired dead `facet_write` float writers + stale marker.
- Research artefact — `.doctrine/slice/249/research/` (runtime tier), three
  threads, baseline stamped at f31b6187.
- `ISS-318` — widened from one instance to the inert-key defect class; absorbed
  the `body`-on-checkpoint instance from SL-248's run.
- `IMP-403` — lead 2 corroborated with the SL-248 evidence; related to `ISS-318`.
- `ISS-316` — absorbed as objective 4, narrowed to its lifecycle-vocabulary half.

### Learned

- The facet write seam already exists and ships: `facet_write::set_facet_mixed` /
  `apply_set_mixed`, consumer at `src/commands/facet.rs:711` serving
  `doctrine risk set`. Objective 1 is wiring.
- SL-222's `deletes at SL-222 deletion phase` marker covers only three
  float-valued symbols; its reason string's premise is false (the migration
  scripts are Python). → `CHR-056`.
- Two write postures already exist in `dep_seq` and the choice is forced:
  `apply_status` refuses a missing key (scaffold-seeded, F-1), `apply_scalar`
  creates one. Facet fields are scaffold-seeded blank, so F-1 applies.
- The facet field inventory is 31 slots / 30 distinct names / 1 shared — the fact
  that decided OQ-1.
- `src/commands/knowledge.rs` does not exist; the knowledge CLI is in
  `src/knowledge.rs`.

### Open

- `OQ-4` (slice card) — is `ConceptFacet`'s emptiness designed or an omission?
  No governance answers it; code corroborates "designed". The REV must rule.
- `OQ-6` (slice card) — does the inert-key refusal extend to `validate_facet`'s
  read path, or is read-tolerance deliberate?
- `R1` — the SPEC-019 amendment is owed a REV (ADR-013), not an in-place edit.
- `R2` — `EVD`/`HYP`/`CPT` facet contracts need rulings, not transcription of
  current code.
- `R2a` — ordering dependency: SL-249's REV lands before `SL-246` derives its
  per-kind field lists.
- Unverified limit: whether `PRD-010` also carries the four-kind framing (if so
  the REV grows to two entities); whether ADR-013's apply path can auto-apply a
  prose-heavy amendment.
- `CHR-056` — open, not a blocker.
