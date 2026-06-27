# REV REV-014 — reconcile SL-165

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

Reconciles SL-165 (close-projection candidate-source gate). Audit RV-177 F-2
confirmed a normative contradiction inside SPEC-022: REQ-316's **Source
provenance** clause forbids any `candidate create --source` that is not a
`Verified` journal row, while REQ-317 (Repair-to-integrate propagation contract)
mandates sourcing a `close_target` from `--source refs/heads/candidate/<N>/<label>`
(the repaired candidate). The landed gate (`src/dispatch.rs`
`check_provenance`/`trace_candidate_provenance`) conforms to REQ-317 — the
controlling intent — and thereby widens REQ-316. Per design D4/Q3-A + ADR-013, a
normative FR gate-widening routes through a Revision + external review, not a quiet
direct spec edit. This REV narrows REQ-316 to admit the traced candidate-source
exception the gate implements. Settles design OQ-2 (exact wording).

## Reconcile narrative (SL-165)

- **[RV-177 F-2] → REQ-316 modify (primary).** Narrow the **Source provenance**
  clause of REQ-316 to admit one exception: a `close_target` create may source a
  recorded `candidate/<N>/<label>` whose role is an `audit` `review_surface` or a
  chained `close_target`, when the chain traces (bounded recursion, depth ≤ 16) to
  a `Verified` journaled-evidence root; `scratch`/`experiment` refused; the
  exception binds by lineage (resolved `source_oid` descends from the source row's
  recorded `merge_oid`, INV-6), never by ref name. Surfaced-for-manual at apply;
  before/after below.

- **[RV-177 F-2] → REQ-317 conformance.** REQ-317 is already `active` and is now
  satisfied by the substrate (no status row needed). Conformance recorded via
  `coverage record` SL-165 → REQ-316 + REQ-317 (VA attestation), per design R3.

- **OQ-3 (assessed, no change).** REQ-317's process-owner note already scopes the
  operator obligation to SPEC-021 and states it "fixes only the substrate fact."
  SL-165 implemented exactly that substrate path; the audit-time guard is IMP-130's
  mandate, a non-goal here. No companion tweak to REQ-317 or SPEC-021.

- **RFC-005 (noted, not rewritten).** Close-projection of a candidate-sourced
  repair is an H2-adjacent hazard for RFC-005 placement (slice OQ-4, deferred).
  Recorded here for the RFC author; the RFC is not edited in this pass.

### REQ-316 §"Source provenance" — before/after (manual landing)

**Before:**

> **Source provenance.** `candidate create` refuses a `--source` that is not a
> `Verified` stage-1 prepare-review journal row — a candidate may only be built
> from verified evidence. A `phase/<N>-NN` (code) close-target additionally refuses
> when an **earlier** non-empty phase row failed (an unresolved hole below the
> selected phase).

**After:**

> **Source provenance.** `candidate create` refuses a `--source` that is not a
> `Verified` stage-1 prepare-review journal row — a candidate may only be built
> from verified evidence — **with one exception for close-target repair propagation
> (REQ-317):** a `close_target` create may source a recorded `candidate/<N>/<label>`
> row, provided that row has `kind = audit` and `role ∈ {review_surface,
> close_target}` (`scratch` and `experiment` sources are refused) and `status =
> Created` (a clean-merge row; a hand-resolved `Conflicted` row is refused — a
> documented v1 limitation). The source `target_ref` must resolve to **exactly one**
> recorded candidate row — a count-exact, fail-closed match (duplicates refused).
> The candidate chain is then traced by bounded recursion (depth ≤ 16); each hop
> applies the same row gate, and the chain must terminate at a `Verified`
> journaled-evidence root (`review/<N>` or `phase/<N>-NN`) validated through the
> *full* journaled gate — including the phase-hole refusal below when that root is a
> `phase/<N>-NN` ref. The exception binds by lineage, not name: the resolved
> `source_oid` must descend from the source row's recorded `merge_oid`, so a moved
> ref cannot launder unrelated history (INV-6).
>
> A `phase/<N>-NN` (code) close-target — whether named directly as the source or
> reached as the traced chain root — additionally refuses when an **earlier**
> non-empty phase row failed (an unresolved hole below the selected phase).
